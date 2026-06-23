mod crawler;
mod parser;
mod index;
mod engine;
mod storage;
mod analyzer;
mod searcher;

use std::path::Path;
use crawler::DirectoryCrawler;
use parser::{PlainTextParser, DocumentParser, MarkdownParser};
// use analyzer::TextAnalyzer;
// use index::InvertedIndex;
use engine::SearchEngineCore;

// use crate::searcher::SearchEngine;

fn main() {
    println!("=== Initializing AuraSearch Engine ===");

    let target_dir = Path::new("./");
    let allowed_extensions = vec!["txt".to_string(), "md".to_string()];

    let crawler = DirectoryCrawler::new(target_dir, allowed_extensions);
    let engine_core = SearchEngineCore::new();

    // let analyzer = TextAnalyzer::new();
    // let mut master_index = InvertedIndex::new();

    println!("Scanning directory for indexable documents . . .");
    let discovered_files = crawler.run();
    println!("Found {} candidate files.", discovered_files.len());

    for path_buf in discovered_files {
        let path_str = path_buf.to_string_lossy().into_owned();
        println!("Processing: {}", path_str);

        let raw_content = match std::fs::read_to_string(&path_buf) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to read file {}", e);
                continue;
            }
        };

        let clean_text = match path_buf.extension().and_then(|ext| ext.to_str()) {
            Some("md") => {
                let parser = MarkdownParser;
                parser.parse(&raw_content)
            }
            Some("txt") | _ => {
                let parser = PlainTextParser;
                parser.parse(&raw_content)
            }
        };

        let tokens = engine_core.analyzer.analyze(&clean_text);
        engine_core.ingest_document(&path_str, tokens);
        // let doc_id = master_index.register_document(&path_str);
        // let tokens = analyzer.analyze(&clean_text);
        // master_index.add_document(doc_id, tokens);
    }

    println!("=== Ingestion Complete ===");
    // println!("Master database contains index entries for {} unique terms.", master_index.index.len());

    // let searcher = SearchEngine::new(&master_index, &analyzer);

    use std::io::{self, Write};
    loop {
        print!("\nAURASEARCH > ");
        io::stdout().flush().unwrap();

        let mut query = String::new();
        io::stdin().read_line(&mut query).unwrap();
        let query = query.trim();

        if query == "exit" || query == "quit" {
            break;
        }

        let hits = engine_core.execute_search(query);
        println!("Found {} matching documents:", hits.len());
        for path in hits {
            println!(" -> {}", path);
        }
    }
}
