use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

use calamine::{open_workbook_auto_from_rs, Reader};
use csv::ReaderBuilder;
use lopdf::Document as PdfDocument;
use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;
use zip::ZipArchive;

use crate::parser::{DocumentParser, MarkdownParser, PlainTextParser};

pub fn allowed_extensions() -> Vec<String> {
    vec![
        "txt".to_string(),
        "md".to_string(),
        "csv".to_string(),
        "pdf".to_string(),
        "docx".to_string(),
        "xlsx".to_string(),
    ]
}

pub fn extract_text(path: &Path) -> Result<String, String> {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("").to_ascii_lowercase();
    match extension.as_str() {
        "md" => extract_plain_or_markdown(path, true),
        "txt" => extract_plain_or_markdown(path, false),
        "csv" => extract_csv(path),
        "docx" => extract_docx(path),
        "xlsx" => extract_xlsx(path),
        "pdf" => extract_pdf(path),
        other => Err(format!("Unsupported file format: {other}")),
    }
}

pub fn normalize_for_indexing(path: &Path) -> Result<String, String> {
    let raw_text = extract_text(path)?;
    let parser: &dyn DocumentParser = match path.extension().and_then(|ext| ext.to_str()) {
        Some("md") => &MarkdownParser,
        _ => &PlainTextParser,
    };

    Ok(parser.parse(&raw_text))
}

fn extract_plain_or_markdown(path: &Path, markdown: bool) -> Result<String, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    if markdown {
        Ok(raw)
    } else {
        Ok(raw)
    }
}

fn extract_csv(path: &Path) -> Result<String, String> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .map_err(|e| format!("Failed to open CSV {}: {e}", path.display()))?;

    let mut text = String::new();
    for record in reader.records() {
        let record = record.map_err(|e| format!("Failed to parse CSV {}: {e}", path.display()))?;
        for field in record.iter() {
            if !field.trim().is_empty() {
                text.push_str(field);
                text.push(' ');
            }
        }
    }
    Ok(text)
}

fn extract_docx(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open DOCX {}: {e}", path.display()))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Invalid DOCX archive {}: {e}", path.display()))?;
    let mut xml = String::new();
    archive
        .by_name("word/document.xml")
        .map_err(|e| format!("Missing document.xml in {}: {e}", path.display()))?
        .read_to_string(&mut xml)
        .map_err(|e| format!("Failed to read DOCX XML {}: {e}", path.display()))?;

    let mut reader = XmlReader::from_str(&xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut text = String::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Text(event)) => {
                let segment = event.unescape().map_err(|e| format!("Failed to decode DOCX text {}: {e}", path.display()))?;
                if !segment.trim().is_empty() {
                    text.push_str(&segment);
                    text.push(' ');
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(format!("Failed to parse DOCX XML {}: {e}", path.display())),
        }
        buffer.clear();
    }

    Ok(text)
}

fn extract_xlsx(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open XLSX {}: {e}", path.display()))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| format!("Failed to read XLSX {}: {e}", path.display()))?;
    let cursor = std::io::Cursor::new(bytes);
    let mut workbook = open_workbook_auto_from_rs(cursor)
        .map_err(|e| format!("Failed to parse XLSX {}: {e}", path.display()))?;

    let mut text = String::new();
    for sheet in workbook.sheet_names().to_owned() {
        if let Ok(range) = workbook.worksheet_range(&sheet) {
            for row in range.rows() {
                for cell in row {
                    let value = cell.to_string();
                    if !value.trim().is_empty() {
                        text.push_str(&value);
                        text.push(' ');
                    }
                }
            }
        }
    }
    Ok(text)
}

fn extract_pdf(path: &Path) -> Result<String, String> {
    let doc = PdfDocument::load(path).map_err(|e| format!("Failed to load PDF {}: {e}", path.display()))?;
    let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    let text = doc.extract_text(&pages).map_err(|e| format!("Failed to extract PDF text {}: {e}", path.display()))?;
    Ok(text)
}

#[allow(dead_code)]
fn _read_all<R: Read + Seek>(mut reader: R) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}
