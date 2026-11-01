use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;
use zip::ZipArchive;
use quick_xml::events::Event;
use quick_xml::Reader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to your PowerPoint file on Android storage via Termux
    let path = Path::new("/sdcard/Documents/presentation.pptx");
    
    let file = File::open(&path)?;
    let mut archive = ZipArchive::new(file)?;

    println!("Successfully opened: {}", path.display());
    println!("Total archive entries: {}\n", archive.len());

    // Iterate through zip entries to find slide XML files
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        // PowerPoint slides are typically stored under ppt/slides/slide*.xml
        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            println!("--- Analyzing {} ---", name);
            
            let mut xml_content = String::new();
            entry.read_to_string(&mut xml_content)?;

            extract_text_from_slide(&xml_content)?;
        }
    }

    Ok(())
}

fn extract_text_from_slide(xml: &str) -> quick-xml::Result<()> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    let mut buf = Vec::new();

    let mut in_text_node = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                // OpenXML text elements use the DrawingML tag name `a:t`
                if e.name().local_name().as_ref() == b"t" {
                    in_text_node = true;
                }
            }
            Ok(Event::Text(e)) if in_text_node => {
                let text = e.unescape()?;
                print!("  [Slide Text]: {}", text);
            }
            Ok(Event::End(ref e)) => {
                if e.name().local_name().as_ref() == b"t" {
                    in_text_node = false;
                    println!();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("Error parsing XML at position {}: {:?}", reader.buffer_position(), e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(())
}
