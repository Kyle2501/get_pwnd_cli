use std::fs::{self, File};
use std::io::{Read, Write, Cursor, BufReader};
use std::path::PathBuf;
use std::collections::HashMap;
use clap::Parser;
use zip::{ZipArchive, ZipWriter};
use zip::write::FileOptions;
use quick_xml::events::{BytesText, Event};
use quick_xml::{Reader, Writer};

#[derive(Parser, Debug)]
#[command(name = "pptx_json_batch")]
#[command(about = "Batch update PowerPoint files using a JSON dictionary of replacement rules", long_about = None)]
struct Args {
    /// Directory containing input .pptx files
    #[arg(short, long)]
    input_dir: PathBuf,

    /// Directory to save processed .pptx files
    #[arg(short, long)]
    output_dir: PathBuf,

    /// Path to the JSON rules dictionary file
    #[arg(short, long)]
    rules: PathBuf,

    /// Optional filename pattern filter
    #[arg(short, long)]
    pattern: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load and parse the JSON rules dictionary
    let rules_file = File::open(&args.rules)?;
    let reader = BufReader::new(rules_file);
    let rules: HashMap<String, String> = serde_json::from_reader(reader)?;
    
    println!("Loaded {} replacement rule(s) from {}", rules.len(), args.rules.display());

    // Ensure output directory exists
    fs::create_dir_all(&args.output_dir)?;

    let entries = fs::read_dir(&args.input_dir)?;
    let mut processed_count = 0;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let is_pptx = path.extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("pptx"))
                .unwrap_or(false);

            if !is_pptx {
                continue;
            }

            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown.pptx");

            if let Some(ref pat) = args.pattern {
                if !file_name.contains(pat) {
                    continue;
                }
            }

            let dest_path = args.output_dir.join(file_name);
            println!("Processing: {} -> {}", file_name, dest_path.display());

            if let Err(e) = process_presentation(&path, &dest_path, &rules) {
                eprintln!("  [Error] Failed to process {}: {}", file_name, e);
            } else {
                processed_count += 1;
            }
        }
    }

    println!("\nBatch complete. Successfully updated {} presentation(s).", processed_count);
    Ok(())
}

fn process_presentation(src: &PathBuf, dest: &PathBuf, rules: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(src)?;
    let mut archive = ZipArchive::new(file)?;

    let out_file = File::create(dest)?;
    let mut zip_writer = ZipWriter::new(out_file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        zip_writer.start_file(name.clone(), options)?;

        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            let mut xml_content = String::new();
            entry.read_to_string(&mut xml_content)?;

            let updated_xml = update_slide_text(&xml_content, rules)?;
            zip_writer.write_all(updated_xml.as_bytes())?;
        } else {
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;
            zip_writer.write_all(&buffer)?;
        }
    }

    zip_writer.finish()?;
    Ok(())
}

fn update_slide_text(xml: &str, rules: &HashMap<String, String>) -> Result<String, Box<dyn std::error::Error>> {
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
                let updated = if let Some(replacement) = rules.get(&text.to_string()) {
                    replacement.clone()
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
