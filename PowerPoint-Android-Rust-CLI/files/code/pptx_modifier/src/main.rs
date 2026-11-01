use std::fs::File;
use std::io::{Read, Write, Cursor};
use std::path::Path;
use zip::{ZipArchive, ZipWriter};
use zip::write::FileOptions;
use quick_xml::events::{BytesText, Event};
use quick_xml::{Reader, Writer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src_path = Path::new("/sdcard/Documents/input.pptx");
    let dest_path = Path::new("/sdcard/Documents/output.pptx");

    let file = File::open(&src_path)?;
    let mut archive = ZipArchive::new(file)?;

    let out_file = File::create(&dest_path)?;
    let mut zip_writer = ZipWriter::new(out_file);

    // Standard compression options for OpenXML zip entries
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    println!("Repacking and modifying presentation...");

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        zip_writer.start_file(name.clone(), options)?;

        // Intercept slide 1 (or iterate over all slides as needed)
        if name == "ppt/slides/slide1.xml" {
            let mut xml_content = String::new();
            entry.read_to_string(&mut xml_content)?;

            // Replace target text inside the DrawingML <a:t> node
            let updated_xml = update_slide_text(&xml_content, "Placeholder Title", "Generated via Termux & Rust")?;
            zip_writer.write_all(updated_xml.as_bytes())?;
            println!(" -> Successfully updated text in {}", name);
        } else {
            // Copy all other files (media, relationships, presentation.xml, etc.) untouched
            let mut buffer = Vec::new();
            entry.read_to_end(&mut buffer)?;
            zip_writer.write_all(&buffer)?;
        }
    }

    zip_writer.finish()?;
    println!("Successfully wrote modified presentation to: {}", dest_path.display());

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
