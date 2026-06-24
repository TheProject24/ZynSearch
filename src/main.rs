mod crawler;
mod parser;
mod analyzer;
mod index;
mod searcher;
mod engine;
mod storage;

use std::path::Path;
use std::io::{self, Write};
use crawler::DirectoryCrawler;
use parser::{PlainTextParser, MarkdownParser, DocumentParser};
use engine::SearchEngineCore;
// use storage::StorageManager;
use storage::{StorageManager, ZeroCopyReader};

fn main() {
    println!("========================================");
    println!("      AuraSearch Engine v1.0 Core       ");
    println!("========================================");

    // Initialize our thread-safe concurrency engine
    let engine_core = SearchEngineCore::new();
    let db_filename = "index.bin";

    // BOOT SEQUENCE: Attempt to hydrate an existing database
    if Path::new(db_filename).exists() {
        println!("[BOOT] Found existing database: '{}'", db_filename);
        print!("[BOOT] Hydrating index into memory... ");
        io::stdout().flush().unwrap();

        match StorageManager::deserialize(db_filename) {
            Ok(loaded_index) => {
                // Acquire an exclusive write lock and swap out the empty index 
                // for the fully hydrated one we just pulled from disk!
                *engine_core.index.write().unwrap() = loaded_index;
                println!("Done.");
            }
            Err(e) => {
                eprintln!("\n[ERROR] Failed to load database: {}", e);
                println!("[BOOT] Falling back to fresh ingestion...");
                run_ingestion_pipeline(&engine_core, db_filename);
            }
        }
    } else {
        println!("[BOOT] No existing database found. Initiating full system crawl...");
        run_ingestion_pipeline(&engine_core, db_filename);
    }

    // INTERACTIVE QUERY SHELL
    println!("\nSystem Ready. Type 'exit' or 'quit' to shutdown.");
    loop {
        print!("AuraSearch > ");
        io::stdout().flush().unwrap();

        let mut query = String::new();
        io::stdin().read_line(&mut query).unwrap();
        let query = query.trim();

        if query == "exit" || query == "quit" {
            println!("Shutting down AuraSearch. Goodbye!");
            break;
        }

        if query.is_empty() {
            continue;
        }

        // ==========================================
        // SECRET DEV COMMAND: Zero-Copy Disk Lookup
        // ==========================================
        if query.starts_with("mmap ") {
            // Extract the word they want to search
            let target_term = query.trim_start_matches("mmap ").trim();
            
            // Read the raw binary file and use our zero-copy engine
            if let Ok(raw_bytes) = std::fs::read(db_filename) {
                let reader = ZeroCopyReader::new(&raw_bytes);
                
                if let Some(doc_ids) = reader.lookup_term_postings(target_term) {
                    println!("  [MMAP DIRECT DISK READ] Found term in {} document IDs: {:?}", doc_ids.len(), doc_ids);
                } else {
                    println!("  [MMAP DIRECT DISK READ] Term not found on disk.");
                }
            } else {
                eprintln!("  [ERROR] Could not read index.bin from disk.");
            }
            continue; // Skip the standard RAM search and ask for the next prompt
        }
        // ==========================================

        // Standard RAM search execution
        let hits = engine_core.execute_search(query);
        
        if hits.is_empty() {
            println!("  No documents matched your query.");
        } else {
            println!("  Found {} matching document(s):", hits.len());
            for path in hits {
                println!("    -> {}", path);
            }
        }
    }
}

/// A dedicated helper function to handle the ingestion logic cleanly
fn run_ingestion_pipeline(engine_core: &SearchEngineCore, db_filename: &str) {
    let target_dir = Path::new("./"); 
    let allowed_extensions = vec!["txt".to_string(), "md".to_string()];
    
    let crawler = DirectoryCrawler::new(target_dir, allowed_extensions);

    println!("[INGEST] Scanning directory for indexable documents...");
    let discovered_files = crawler.run();
    println!("[INGEST] Found {} candidate files.", discovered_files.len());

    for path_buf in discovered_files {
        let path_str = path_buf.to_string_lossy().into_owned();

        let raw_content = match std::fs::read_to_string(&path_buf) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let clean_text = match path_buf.extension().and_then(|ext| ext.to_str()) {
            Some("md") => MarkdownParser.parse(&raw_content),
            _ => PlainTextParser.parse(&raw_content),
        };

        let tokens = engine_core.analyzer.analyze(&clean_text);
        engine_core.ingest_document(&path_str, tokens);
    }

    println!("[INGEST] Ingestion Complete.");

    // Serialize snapshot out to disk
    let current_state = engine_core.index.read().unwrap();
    if let Err(e) = StorageManager::serialize(&current_state, db_filename) {
        eprintln!("[ERROR] Failed to persist database index: {}", e);
    } else {
        println!("[STORAGE] Database snapshot safely written to '{}'", db_filename);
    }
}