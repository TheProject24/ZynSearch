use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "AuraSearch Daemon")]
#[command(version = "1.0.0")]
#[command(about = "A high-performance zero-copy TCP search engine", long_about = None)]
pub struct Config {
    /// The host IP to bind the TCP server to.
    #[arg(short = 'H', long, env = "AURA_HOST", default_value = "127.0.0.1")]
    pub host: String,

    /// The port to listen on for incoming TCP queries.
    #[arg(short = 'P', long, env = "AURA_PORT", default_value = "7777")]
    pub port: u16,

    /// Path to the serialized binary database file.
    #[arg(short = 'D', long, env = "AURA_DB_PATH", default_value = "index.bin")]
    pub db_path: String,

    /// Directory to crawl if no existing database is found.
    #[arg(short = 'C', long, env = "AURA_CORPUS_DIR", default_value = "./")]
    pub corpus_dir: String,
}

#[derive(ValueEnum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Binary,
}

#[derive(Parser, Debug, Clone)]
pub struct Config {
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    pub host: String,

    #[arg(short = 'P', long, default_value = "7777")]
    pub port: u16,

    // NEW: Allow the user to specify the serialization format!
    #[arg(short = 'F', long, env = "AURA_FORMAT", value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
    
    #[arg(short = 'D', long, default_value = "index.bin")]
    pub db_path: String,
}