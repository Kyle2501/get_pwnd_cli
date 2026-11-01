use std::fs::File;
use std::io::{Read, Write, Cursor};
use std::path::PathBuf;
use clap::Parser;
use zip::{ZipArchive, ZipWriter};
use zip::write::FileOptions;
use quick_xml::events::{BytesText, Event};
use quick_xml::{Reader, Writer};

#[derive(Parser, Debug)]
#[command(name = "pptx_cli")]
#[command(about = "Search and replace text inside PowerPoint (.pptx) presentations via CLI", long_about = None)]
struct Args {
    /// Path to the input .pptx file
    #[arg(short, long)]
    input: PathBuf,

    /// Path to save the modified .pptx file
    #[arg(short, long)]
    output: PathBuf,

    /// Text string to search for within slide content
    #[arg(short, long)]
    target: String,

    /// Replacement text string
    #[arg(short, long)]
    replacement: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let file = File::open(&args.input)?;
    let mut archive = ZipArchive::new(file)?;

    let out_file = File::create(&args.output)?;
    let mut zip_writer = ZipWriter::new(out_file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    println!("Processing: {} -> {}", args.input.display(), args.output.display());

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        zip_writer.start_file(name.clone(), options)?;

        // Inspect and modify slide XML files
        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            let mut xml_content = String::new();
            entry.read_to_string(&mut xml_content)?;

            let updated_xml = update_slide_text(&xml_content, &args.target, &args.replacement)?;
            zip_writer.write_all(updated_xml.as_bytes())?;
            println!(" -> Updated text references in {}", name);
        } else {
            // Copy assets, relationships, and core metadata untouched
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;
            zip_writer.write_all(&buffer)?;
        }
    }

    zip_writer.finish()?;
    println!("Successfully saved modified presentation to: {}", args.output.display());

    Ok(())
}

fn update_slide_text(xml: &str, target_text: &str, new_text: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(false);
    
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();
    let mut in_t_tag = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().local_name().as_ref() == b"t" {
                    in_t_tag = true;
                }
                writer.write_event(Event::Start(e.clone()))?;
            }
            Ok(Event::Text(e)) if in_t_tag => {
                let text = e.unescape()?;
                let updated = if text == target_text {
                    new_text.to_string()
                } else {
                    text.into_owned()
                };
                writer.write_event(Event::Text(BytesText::new(&updated)))?;
            }
            Ok(Event::End(ref e)) => {
                if e.name().local_name().as_ref() == b"t" {
                    in_t_tag = false;
                }
                writer.write_event(Event::End(e.clone()))?;
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                writer.write_event(e)?;
            }
            Err(e) => return Err(e.into()),
        }
        buf.clear();
    }

    let result_bytes = writer.into_inner().into_inner();
    Ok(String::from_utf8(result_bytes)?)
}
