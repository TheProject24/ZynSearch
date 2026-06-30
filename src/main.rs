use std::path::Path;
use clap::Parser;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::Serialize;

// 1. Import your items from the library target.
// NOTE: If your package name in Cargo.toml is different, replace 'zynsearch' with that name.
use zynsearch::engine::SearchEngineCore;
use zynsearch::storage::StorageManager;
use zynsearch::config::Config;
use zynsearch::query_pipeline::{format_results, parse_query, QueryCoordinator};
use zynsearch::multi_protocol::ZynQuery;

#[derive(Serialize)]
struct SearchResponse {
    query: String,
    status: String,
    match_count: usize,
    documents: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Parse configuration from CLI arguments or .env variables
    let config = Config::parse();

    println!("========================================");
    println!("      ZynSearch TCP Daemon v1.0        ");
    println!("========================================");

    let engine_core = SearchEngineCore::new();
    let shared_engine = std::sync::Arc::new(engine_core);
    let coordinator = QueryCoordinator::new(shared_engine.clone(), 4);

    // 2. BOOT SEQUENCE
    if Path::new(&config.db_path).exists() {
        println!("[BOOT] Hydrating database from: {}", config.db_path);
        match StorageManager::deserialize(&config.db_path) {
            Ok(loaded_index) => {
                *shared_engine.index.write().unwrap() = loaded_index;
                println!("[BOOT] Hydration successful.");
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to load DB: {}", e);
                run_ingestion_pipeline(&shared_engine, &config)?;
            }
        }
    } else {
        println!("[BOOT] No database found. Initiating corpus crawl...");
        run_ingestion_pipeline(&shared_engine, &config)?;
    }

    if let Some(query) = config.query.clone() {
        // 2. FIXED: Changed `crate::multi_protocol::ZynQuery` to `ZynQuery` (imported above)
        let hits = coordinator.execute(ZynQuery { query_string: query.clone(), limit: 10 });
        println!("{}", String::from_utf8_lossy(&format_results(&hits, config.format)));
        return Ok(());
    }

    // 3. START TCP SERVER
    let bind_addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&bind_addr).await?;
    println!("\n[NETWORK] Server listening on TCP {}", bind_addr);
    println!("[NETWORK] Ready for incoming connections...\n");

    // 4. ASYNC CONNECTION LOOP
    loop {
        // Wait for a new client to connect
        let (mut socket, addr) = listener.accept().await?;
        println!("[TCP] Client connected from: {}", addr);

        // Spawn a concurrent Tokio task.
        let coordinator = coordinator.clone();
        tokio::spawn(async move {
            let mut buffer = [0; 1024];

            // Greet the client
            let _ = socket.write_all(b"Connected to ZynSearch. Enter query:\n> ").await;

            loop {
                // Read client input
                let bytes_read = match socket.read(&mut buffer).await {
                    Ok(n) if n == 0 => break, // Client disconnected cleanly
                    Ok(n) => n,
                    Err(_) => break, // Connection dropped
                };

                let payload = &buffer[..bytes_read];
                let parsed_query = match parse_query(payload) {
                    Ok(query) => query,
                    Err(err) => {
                        let _ = socket.write_all(format!("Query parse error: {}\n", err).as_bytes()).await;
                        continue;
                    }
                };

                if parsed_query.query_string.trim().is_empty() {
                    let _ = socket.write_all(b"> ").await;
                    continue;
                }

                if parsed_query.query_string == "exit" || parsed_query.query_string == "quit" {
                    let _ = socket.write_all(b"Goodbye!\n").await;
                    break;
                }

                let hits = coordinator.execute(parsed_query.clone());
                let wire_bytes = format_results(&hits, config.format);

                // Shoot the customized byte envelope out across the TCP wire!
                if socket.write_all(&wire_bytes).await.is_err() {
                    break;
                }
            }
            println!("[TCP] Client disconnected: {}", addr);
        });
    }
}

fn run_ingestion_pipeline(engine_core: &std::sync::Arc<SearchEngineCore>, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let target_dir = Path::new(&config.corpus_dir);
    let indexed = engine_core.ingest_corpus_dir(target_dir)?;
    println!("[BOOT] Indexed {} documents from corpus.", indexed.len());
    
    let current_state = engine_core.index.read().unwrap();
    let _ = StorageManager::serialize(&current_state, &config.db_path);
    Ok(())
}