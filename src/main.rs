// src/main.rs - Simplified entry point that delegates to CLI

use fast_meme_trader::{FastMemeTrader, example_usage};
use anyhow::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    // Load environment variables
    dotenv::dotenv().ok();
    
    let args: Vec<String> = env::args().collect();
    
    match args.get(1).map(|s| s.as_str()) {
        Some("example") => {
            println!("🚀 Running example usage...");
            example_usage().await
        },
        Some("health") => {
            println!("🔍 Running health check...");
            run_health_check().await
        },
        Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        },
        _ => {
            println!("🚀 Fast Solana Meme Coin Trading Bot v0.3.1");
            println!("Use 'cargo run --bin cli' for interactive mode");
            println!("Use 'cargo run example' for example usage");
            println!("Use 'cargo run health' for health check");
            println!("Use 'cargo run --help' for more options");
            Ok(())
        }
    }
}

async fn run_health_check() -> Result<()> {
    let private_key = std::env::var("WALLET_PRIVATE_KEY")
        .expect("❌ WALLET_PRIVATE_KEY not set in .env file");
    let helius_api_key = std::env::var("HELIUS_API_KEY")
        .expect("❌ HELIUS_API_KEY not set in .env file");
    
    let trader = FastMemeTrader::new(&private_key, helius_api_key)?;
    let health = trader.health_check().await?;
    println!("📊 {}", health);
    
    Ok(())
}

fn print_help() {
    println!("Fast Solana Meme Trading Bot v0.3.1");
    println!();
    println!("USAGE:");
    println!("    cargo run [COMMAND]");
    println!("    cargo run --bin cli          # Interactive CLI mode");
    println!();
    println!("COMMANDS:");
    println!("    example    Run example trading session");
    println!("    health     Check bot health and connectivity");
    println!("    --help     Show this help message");
    println!();
    println!("ENVIRONMENT:");
    println!("    WALLET_PRIVATE_KEY    Your Solana wallet private key (base58)");
    println!("    HELIUS_API_KEY       Your Helius RPC API key");
    println!();
    println!("For interactive trading, use: cargo run --bin cli");
}