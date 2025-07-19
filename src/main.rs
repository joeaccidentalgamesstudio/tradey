// src/bin/cli.rs - Command Line Interface for the trading bot

use fast_meme_trader::{FastMemeTrader, TradeConfig, StrategyType, token_addresses};
use anyhow::Result;
use std::io::{self, Write};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();
    
    println!("🚀 Fast Solana Meme Coin Trading Bot v0.3.0");
    println!("===============================================");
    
    // Initialize trader
    let private_key = std::env::var("WALLET_PRIVATE_KEY")
        .expect("❌ WALLET_PRIVATE_KEY not set in .env file");
    let helius_api_key = std::env::var("HELIUS_API_KEY")
        .expect("❌ HELIUS_API_KEY not set in .env file");
    
    println!("🔄 Initializing trader...");
    let trader = FastMemeTrader::new(&private_key, helius_api_key)?;
    println!("✅ Trader initialized successfully!");
    println!("🔑 Wallet: {}...{}", 
        &trader.keypair.pubkey().to_string()[..8],
        &trader.keypair.pubkey().to_string()[trader.keypair.pubkey().to_string().len()-8..]
    );
    
    loop {
        println!("\n📋 Main Menu:");
        println!("1. 🚀 Quick Buy (with strategy)");
        println!("2. 💰 Quick Sell");
        println!("3. 📊 View Positions");
        println!("4. 🎯 Monitor Positions (auto-trading)");
        println!("5. 📈 Check ATH Status");
        println!("6. 🚨 Emergency Sell All");
        println!("7. 📋 Platform Test");
        println!("8. 🔧 Settings");
        println!("9. ❌ Exit");
        
        print!("\nSelect option (1-9): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => quick_buy(&trader).await?,
            "2" => quick_sell(&trader).await?,
            "3" => view_positions(&trader).await?,
            "4" => monitor_positions(&trader).await?,
            "5" => check_ath_status(&trader).await?,
            "6" => emergency_sell_all(&trader).await?,
            "7" => platform_test(&trader).await?,
            "8" => show_settings(),
            "9" => {
                println!("👋 Goodbye!");
                break;
            },
            _ => println!("❌ Invalid option, please try again."),
        }
    }
    
    Ok(())
}

async fn quick_buy(trader: &FastMemeTrader) -> Result<()> {
    println!("\n🚀 Quick Buy Setup");
    
    // Get token address
    print!("Enter token address (or 'bonk' for BONK): ");
    io::stdout().flush()?;
    let mut token_input = String::new();
    io::stdin().read_line(&mut token_input)?;
    let token_address = match token_input.trim().to_lowercase().as_str() {
        "bonk" => token_addresses::BONK.to_string(),
        "usdc" => token_addresses::USDC.to_string(),
        addr => addr.to_string(),
    };
    
    // Get amount
    print!("Enter SOL amount (0.001 - 10.0): ");
    io::stdout().flush()?;
    let mut amount_input = String::new();
    io::stdin().read_line(&mut amount_input)?;
    let amount_sol: f64 = amount_input.trim().parse()
        .map_err(|_| anyhow::anyhow!("Invalid amount"))?;
    
    if amount_sol < 0.001 || amount_sol > 10.0 {
        println!("❌ Amount must be between 0.001 and 10.0 SOL");
        return Ok(());
    }
    
    // Get strategy
    println!("\nSelect strategy:");
    println!("1. 💚 Conservative (15% profit, 5% stop loss)");
    println!("2. 🔥 Aggressive (50% profit, 15% stop loss)");
    println!("3. 🎯 Conservative ATH (8% pullback, 3% min profit)");
    println!("4. ⚡ Aggressive ATH (12% pullback, 5% min profit)");
    print!("Choice (1-4): ");
    io::stdout().flush()?;
    
    let mut strategy_input = String::new();
    io::stdin().read_line(&mut strategy_input)?;
    let strategy = match strategy_input.trim() {
        "1" => StrategyType::Conservative,
        "2" => StrategyType::Aggressive,
        "3" => StrategyType::ConservativeATH,
        "4" => StrategyType::AggressiveATH,
        _ => {
            println!("❌ Invalid choice, using Conservative ATH");
            StrategyType::ConservativeATH
        }
    };
    
    // Get slippage
    print!("Enter slippage % (default 1.0): ");
    io::stdout().flush()?;
    let mut slippage_input = String::new();
    io::stdin().read_line(&mut slippage_input)?;
    let slippage_percent: f64 = if slippage_input.trim().is_empty() {
        1.0
    } else {
        slippage_input.trim().parse().unwrap_or(1.0)
    };
    let slippage_bps = (slippage_percent * 100.0) as u16;
    
    println!("\n🔄 Executing buy...");
    println!("💰 Amount: {} SOL", amount_sol);
    println!("🎯 Token: {}", token_address);
    println!("📊 Strategy: {:?}", strategy);
    println!("📈 Slippage: {}%", slippage_percent);
    
    let config = TradeConfig {
        token_address: token_address.clone(),
        amount_sol,
        slippage_bps,
        strategy,
    };
    
    let result = trader.buy_fast(config).await;
    
    if result.success {
        println!("✅ Buy successful!");
        println!("📝 Signature: {}", result.signature);
        println!("⚡ Platform: {:?}", result.platform_used);
        println!("⏱️  Execution time: {}ms", result.execution_time_ms);
        if let Some(tokens) = result.tokens_received {
            println!("🪙 Tokens received: {}", tokens);
        }
        println!("\n🎯 Position created with {:?} strategy", strategy);
        println!("💡 Use option 4 to start monitoring for auto-exit");
    } else {
        println!("❌ Buy failed: {}", result.error.unwrap_or("Unknown error".to_string()));
    }
    
    Ok(())
}

async fn quick_sell(trader: &FastMemeTrader) -> Result<()> {
    println!("\n💰 Quick Sell");
    
    // Show current positions
    let positions = trader.list_positions().await;
    if positions.is_empty() {
        println!("📭 No active positions to sell");
        return Ok(());
    }
    
    println!("Current positions:");
    for (i, position) in positions.iter().enumerate() {
        println!("{}. {}", i + 1, position);
    }
    
    print!("\nEnter token address to sell: ");
    io::stdout().flush()?;
    let mut token_input = String::new();
    io::stdin().read_line(&mut token_input)?;
    let token_address = token_input.trim();
    
    println!("🔄 Executing sell...");
    let result = trader.sell_position(token_address).await?;
    
    if result.success {
        println!("✅ Sell successful!");
        println!("📝 Signature: {}", result.signature);
        println!("⏱️  Execution time: {}ms", result.execution_time_ms);
    } else {
        println!("❌ Sell failed: {}", result.error.unwrap_or("Unknown error".to_string()));
    }
    
    Ok(())
}

async fn view_positions(trader: &FastMemeTrader) -> Result<()> {
    println!("\n📊 Current Positions");
    
    let positions = trader.list_positions().await;
    if positions.is_empty() {
        println!("📭 No active positions");
        return Ok(());
    }
    
    for (i, position) in positions.iter().enumerate() {
        println!("{}. {}", i + 1, position);
    }
    
    Ok(())
}

async fn monitor_positions(trader: &FastMemeTrader) -> Result<()> {
    println!("\n🎯 Starting Position Monitoring");
    println!("💡 Strategies will auto-execute when conditions are met");
    println!("⚠️  Press Ctrl+C to stop monitoring");
    
    let mut iteration = 0;
    loop {
        iteration += 1;
        println!("\n🔄 Monitoring cycle #{}", iteration);
        
        // Check for sells
        let sells = trader.monitor_positions().await;
        for sell in sells {
            println!("💰 EXECUTED SELL: {}", sell);
        }
        
        // Show current positions
        let positions = trader.list_positions().await;
        if positions.is_empty() {
            println!("📭 No active positions - monitoring will continue");
        } else {
            println!("📊 Active positions:");
            for position in positions {
                println!("   {}", position);
            }
        }
        
        // Show ATH status for all positions
        println!("📈 ATH Status:");
        let position_tokens: Vec<String> = {
            let positions = trader.positions.read().await;
            positions.keys().cloned().collect()
        };
        
        for token in position_tokens {
            if let Some(status) = trader.get_ath_status(&token).await {
                println!("   {}: {}", &token[..8], status);
            }
        }
        
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

async fn check_ath_status(trader: &FastMemeTrader) -> Result<()> {
    println!("\n📈 ATH Status Check");
    
    print!("Enter token address: ");
    io::stdout().flush()?;
    let mut token_input = String::new();
    io::stdin().read_line(&mut token_input)?;
    let token_address = token_input.trim();
    
    if let Some(status) = trader.get_ath_status(token_address).await {
        println!("📊 {}", status);
    } else {
        println!("❌ No ATH data found for this token");
    }
    
    Ok(())
}

async fn emergency_sell_all(trader: &FastMemeTrader) -> Result<()> {
    println!("\n🚨 EMERGENCY SELL ALL POSITIONS");
    print!("⚠️  Are you sure? This will sell ALL positions immediately! (y/N): ");
    io::stdout().flush()?;
    
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;
    
    if confirm.trim().to_lowercase() == "y" {
        println!("🔄 Executing emergency sell for all positions...");
        let results = trader.emergency_sell_all().await;
        
        println!("📊 Emergency sell results:");
        for (i, result) in results.iter().enumerate() {
            if result.success {
                println!("{}. ✅ Success: {} ({}ms)", i + 1, result.signature, result.execution_time_ms);
            } else {
                println!("{}. ❌ Failed: {}", i + 1, result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
            }
        }
        
        println!("🧹 All positions cleared");
    } else {
        println!("❌ Emergency sell cancelled");
    }
    
    Ok(())
}

async fn platform_test(trader: &FastMemeTrader) -> Result<()> {
    println!("\n🔧 Platform Detection Test");
    
    let test_tokens = vec![
        ("BONK", token_addresses::BONK),
        ("SOL", token_addresses::SOL),
        ("USDC", token_addresses::USDC),
    ];
    
    for (name, address) in test_tokens {
        print!("Testing {}... ", name);
        io::stdout().flush()?;
        
        let platform = trader.detect_best_platform(address).await;
        println!("Platform: {:?}", platform);
    }
    
    println!("\n🔍 Custom Token Test");
    print!("Enter token address to test: ");
    io::stdout().flush()?;
    let mut token_input = String::new();
    io::stdin().read_line(&mut token_input)?;
    let token_address = token_input.trim();
    
    if !token_address.is_empty() {
        println!("🔄 Testing platform detection...");
        let platform = trader.detect_best_platform(token_address).await;
        println!("✅ Best platform for {}: {:?}", &token_address[..8], platform);
    }
    
    Ok(())
}

fn show_settings() {
    println!("\n🔧 Current Settings");
    println!("Environment variables from .env file:");
    
    if let Ok(key) = std::env::var("WALLET_PRIVATE_KEY") {
        println!("🔑 Wallet: {}...{}", &key[..8], &key[key.len()-8..]);
    }
    
    if let Ok(key) = std::env::var("HELIUS_API_KEY") {
        println!("🌐 Helius API: {}...{}", &key[..8], &key[key.len()-8..]);
    }
    
    println!("📊 Strategies Available:");
    println!("   • Conservative: 15% profit, 5% stop loss");
    println!("   • Aggressive: 50% profit, 15% stop loss");
    println!("   • Conservative ATH: 8% pullback, 3% min profit");
    println!("   • Aggressive ATH: 12% pullback, 5% min profit");
    
    println!("🎯 Supported Platforms:");
    println!("   • PumpFun (new meme coins)");
    println!("   • Raydium (established tokens)");
    println!("   • Jupiter (DEX aggregator)");
    
    println!("\n💡 Edit .env file to change settings");
}