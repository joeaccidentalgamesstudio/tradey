// src/bin/cli.rs - FIXED Command Line Interface for the trading bot

use fast_meme_trader::{FastMemeTrader, TradeConfig, StrategyType, token_addresses};
use anyhow::Result;
use std::io::{self, Write};
use std::time::Duration;
use solana_sdk::{signature::Signer, pubkey::Pubkey};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();
    
    println!("🚀 Fast Solana Meme Coin Trading Bot v0.3.1 - FIXED");
    println!("====================================================");
    
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
    
    // Show initial health check
    if let Ok(health) = trader.health_check().await {
        println!("📊 {}", health);
    }
    
    loop {
        println!("\n📋 Main Menu:");
        println!("1. 🚀 Quick Buy (with strategy)");
        println!("2. 💰 Quick Sell");
        println!("3. 📊 View Positions");
        println!("4. 🎯 Monitor Positions (auto-trading)");
        println!("5. 📈 Check ATH Status");
        println!("6. 🚨 Emergency Sell All");
        println!("7. 📋 Platform Test");
        println!("8. 📊 Performance Stats");
        println!("9. 🔧 Settings");
        println!("0. ❌ Exit");
        
        print!("\nSelect option (0-9): ");
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
            "8" => performance_stats(&trader).await?,
            "9" => show_settings(),
            "0" => {
                println!("👋 Goodbye!");
                break;
            },
            _ => println!("❌ Invalid option, please try again."),
        }
    }
    
    Ok(())
}

// FIXED: Enhanced quick_buy with comprehensive validation
async fn quick_buy(trader: &FastMemeTrader) -> Result<()> {
    println!("\n🚀 Quick Buy Setup - ENHANCED VALIDATION");
    
    // Get token address with better validation
    print!("Enter token address (or 'bonk' for BONK, 'usdc' for USDC): ");
    io::stdout().flush()?;
    let mut token_input = String::new();
    io::stdin().read_line(&mut token_input)?;
    let token_address = match token_input.trim().to_lowercase().as_str() {
        "bonk" => token_addresses::BONK.to_string(),
        "usdc" => token_addresses::USDC.to_string(),
        "usdt" => token_addresses::USDT.to_string(),
        "jup" => token_addresses::JUP.to_string(),
        addr => addr.to_string(),
    };
    
    // FIXED: Enhanced token address validation
    if token_address.len() != 44 {
        println!("❌ Invalid token address length: expected 44 characters, got {}", token_address.len());
        println!("💡 Example valid address: DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263");
        return Ok(());
    }
    
    // Test if we can parse the address as a valid Pubkey
    if let Err(e) = Pubkey::from_str(&token_address) {
        println!("❌ Invalid token address format: {}", e);
        println!("💡 Make sure it's a valid base58 Solana address");
        return Ok(());
    }
    
    println!("✅ Token address validated: {}", &token_address[..8]);
    
    // FIXED: Enhanced amount validation with better limits
    print!("Enter SOL amount (0.000001 - 10.0): ");
    io::stdout().flush()?;
    let mut amount_input = String::new();
    io::stdin().read_line(&mut amount_input)?;
    let amount_sol: f64 = amount_input.trim().parse()
        .map_err(|_| anyhow::anyhow!("Invalid amount - please enter a number"))?;
    
    if amount_sol < 0.000001 || amount_sol > 10.0 {
        println!("❌ Amount must be between 0.000001 and 10.0 SOL");
        println!("💡 Minimum: 0.000001 SOL (~$0.0002), Maximum: 10.0 SOL");
        return Ok(());
    }
    
    println!("✅ Amount validated: {} SOL", amount_sol);
    
    // Get strategy
    println!("\nSelect strategy:");
    println!("1. 💚 Conservative (15% profit, 5% stop loss)");
    println!("2. 🔥 Aggressive (50% profit, 15% stop loss)");
    println!("3. 🎯 Conservative ATH (8% pullback, 3% min profit) - RECOMMENDED");
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
            println!("❌ Invalid choice, using Conservative ATH (recommended)");
            StrategyType::ConservativeATH
        }
    };
    
    println!("✅ Strategy selected: {:?}", strategy);
    
    // FIXED: Enhanced slippage validation
    print!("Enter slippage % (default 1.0, range 0.1-50.0): ");
    io::stdout().flush()?;
    let mut slippage_input = String::new();
    io::stdin().read_line(&mut slippage_input)?;
    let slippage_percent: f64 = if slippage_input.trim().is_empty() {
        1.0
    } else {
        match slippage_input.trim().parse::<f64>() {
            Ok(val) => {
                if val < 0.1 {
                    println!("⚠️  Slippage too low, using minimum 0.1%");
                    0.1
                } else if val > 50.0 {
                    println!("⚠️  Slippage too high, using maximum 50.0%");
                    50.0
                } else {
                    val
                }
            },
            Err(_) => {
                println!("⚠️  Invalid slippage, using default 1.0%");
                1.0
            }
        }
    };
    let slippage_bps = (slippage_percent * 100.0) as u16;
    
    println!("✅ Slippage set: {}% ({} bps)", slippage_percent, slippage_bps);
    
    // Show confirmation
    println!("\n🔄 EXECUTING BUY ORDER");
    println!("=====================================");
    println!("💰 Amount: {} SOL", amount_sol);
    println!("🎯 Token: {}", token_address);
    println!("📊 Strategy: {:?}", strategy);
    println!("📈 Slippage: {}%", slippage_percent);
    println!("=====================================");
    
    let config = TradeConfig {
        token_address: token_address.clone(),
        amount_sol,
        slippage_bps,
        strategy: strategy.clone(),
    };
    
    println!("⏳ Processing... (this may take 10-30 seconds)");
    let result = trader.buy_fast(config).await;
    
    if result.success {
        println!("\n🎉 BUY SUCCESSFUL!");
        println!("=====================================");
        println!("📝 Signature: {}", result.signature);
        println!("⚡ Platform: {:?}", result.platform_used);
        println!("⏱️  Execution time: {}ms", result.execution_time_ms);
        if let Some(tokens) = result.tokens_received {
            println!("🪙 Tokens received: {}", tokens);
        }
        if let Some(sol_spent) = result.sol_spent {
            println!("💸 SOL spent: {}", sol_spent);
        }
        println!("=====================================");
        println!("🎯 Position created with {:?} strategy", strategy);
        println!("💡 Use option 4 to start monitoring for auto-exit");
        println!("💡 Use option 3 to view your position details");
    } else {
        println!("\n❌ BUY FAILED!");
        println!("=====================================");
        println!("Error: {}", result.error.unwrap_or("Unknown error".to_string()));
        println!("⏱️  Execution time: {}ms", result.execution_time_ms);
        println!("=====================================");
        
        println!("\n🔧 TROUBLESHOOTING GUIDE:");
        println!("1. ✅ Token address validation: Check if address is exactly 44 characters");
        println!("2. 💰 Insufficient balance: Ensure you have enough SOL + gas fees");
        println!("3. 📊 No liquidity: Token might not have enough liquidity for your trade size");
        println!("4. ⚡ Network issues: Try again in a few seconds");
        println!("5. 📈 Slippage too low: Try increasing slippage to 2-5%");
        println!("6. 🎯 Token not tradeable: Some tokens may not be available on Jupiter/PumpFun");
        
        // Additional specific troubleshooting based on error
        if let Some(error_msg) = &result.error {
            if error_msg.contains("400") {
                println!("\n🚨 API Error 400 - Specific fixes:");
                println!("   • Token address might be invalid or not tradeable");
                println!("   • Try with a known token like BONK first");
                println!("   • Increase slippage tolerance");
            } else if error_msg.contains("insufficient") {
                println!("\n🚨 Insufficient Balance - Check:");
                println!("   • Your SOL balance (need {} + ~0.01 SOL for fees)", amount_sol);
                println!("   • Try with a smaller amount");
            }
        }
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
    
    print!("\nEnter token address to sell (or position number): ");
    io::stdout().flush()?;
    let mut token_input = String::new();
    io::stdin().read_line(&mut token_input)?;
    let input = token_input.trim();
    
    let token_address = if let Ok(pos_num) = input.parse::<usize>() {
        if pos_num > 0 && pos_num <= positions.len() {
            // Extract token address from position string
            let position_str = &positions[pos_num - 1];
            let parts: Vec<&str> = position_str.split(':').collect();
            if !parts.is_empty() {
                parts[0].to_string()
            } else {
                input.to_string()
            }
        } else {
            println!("❌ Invalid position number");
            return Ok(());
        }
    } else {
        input.to_string()
    };
    
    println!("🔄 Executing sell for {}...", &token_address[..8]);
    let result = trader.sell_position(&token_address).await?;
    
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
    
    // Show performance stats
    let stats = trader.get_performance_stats().await;
    println!("\n📈 {}", stats);
    
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
        
        // Show performance stats
        let stats = trader.get_performance_stats().await;
        println!("📈 {}", stats);
        
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

async fn check_ath_status(trader: &FastMemeTrader) -> Result<()> {
    println!("\n📈 ATH Status Check");
    
    // Show all positions first
    let position_tokens: Vec<String> = {
        let positions = trader.positions.read().await;
        positions.keys().cloned().collect()
    };
    
    if position_tokens.is_empty() {
        println!("📭 No active positions to check");
        return Ok(());
    }
    
    println!("Current positions:");
    for (i, token) in position_tokens.iter().enumerate() {
        println!("{}. {}", i + 1, &token[..8]);
    }
    
    print!("\nEnter token address (or position number, or 'all' for all): ");
    io::stdout().flush()?;
    let mut token_input = String::new();
    io::stdin().read_line(&mut token_input)?;
    let input = token_input.trim();
    
    if input.to_lowercase() == "all" {
        for token in position_tokens {
            if let Some(status) = trader.get_ath_status(&token).await {
                println!("{}: {}", &token[..8], status);
            }
        }
    } else if let Ok(pos_num) = input.parse::<usize>() {
        if pos_num > 0 && pos_num <= position_tokens.len() {
            let token = &position_tokens[pos_num - 1];
            if let Some(status) = trader.get_ath_status(token).await {
                println!("📊 {}: {}", &token[..8], status);
            }
        } else {
            println!("❌ Invalid position number");
        }
    } else {
        let token_address = input;
        if let Some(status) = trader.get_ath_status(token_address).await {
            println!("📊 {}: {}", &token_address[..8], status);
        } else {
            println!("❌ No ATH data found for this token");
        }
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

// FIXED: Enhanced platform test with better error handling
async fn platform_test(trader: &FastMemeTrader) -> Result<()> {
    println!("\n🔧 Platform Detection Test - ENHANCED");
    
    let test_tokens = vec![
        ("BONK", token_addresses::BONK),
        ("SOL", token_addresses::SOL),
        ("USDC", token_addresses::USDC),
        ("USDT", token_addresses::USDT),
        ("JUP", token_addresses::JUP),
    ];
    
    println!("Testing known tokens:");
    for (name, address) in test_tokens {
        print!("Testing {}... ", name);
        io::stdout().flush()?;
        
        let platform = trader.detect_best_platform(address).await;
        println!("Platform: {:?}", platform);
    }
    
    println!("\n🔍 Custom Token Test");
    print!("Enter token address to test (or press Enter to skip): ");
    io::stdout().flush()?;
    let mut token_input = String::new();
    io::stdin().read_line(&mut token_input)?;
    let token_address = token_input.trim();
    
    if !token_address.is_empty() {
        // Validate first
        if token_address.len() != 44 {
            println!("❌ Invalid token address length: expected 44 characters");
            return Ok(());
        }
        
        if let Err(e) = Pubkey::from_str(token_address) {
            println!("❌ Invalid token address format: {}", e);
            return Ok(());
        }
        
        println!("🔄 Testing platform detection...");
        let platform = trader.detect_best_platform(token_address).await;
        println!("✅ Best platform for {}: {:?}", &token_address[..8], platform);
        
        // Test Jupiter quote capability
        println!("🔄 Testing Jupiter quote capability...");
        let config = TradeConfig {
            token_address: token_address.to_string(),
            amount_sol: 0.001,
            slippage_bps: 100,
            strategy: StrategyType::Conservative,
        };
        
        // This is just a test - don't actually execute
        match trader.get_jupiter_quote(&config, 1000000).await {
            Ok(_) => println!("✅ Jupiter quote test: SUCCESS"),
            Err(e) => println!("❌ Jupiter quote test: FAILED - {}", e),
        }
    }
    
    Ok(())
}

async fn performance_stats(trader: &FastMemeTrader) -> Result<()> {
    println!("\n📊 Performance Statistics");
    
    let stats = trader.get_performance_stats().await;
    println!("📈 {}", stats);
    
    if let Ok(health) = trader.health_check().await {
        println!("🔧 {}", health);
    }
    
    // Show detailed position breakdown
    let positions = trader.list_positions().await;
    if !positions.is_empty() {
        println!("\n📋 Detailed Position Breakdown:");
        for (i, position) in positions.iter().enumerate() {
            println!("{}. {}", i + 1, position);
        }
    }
    
    Ok(())
}

// FIXED: Enhanced settings display
fn show_settings() {
    println!("\n🔧 Current Settings - ENHANCED");
    println!("Environment variables from .env file:");
    
    if let Ok(key) = std::env::var("WALLET_PRIVATE_KEY") {
        println!("🔑 Wallet: {}...{}", &key[..8], &key[key.len()-8..]);
    } else {
        println!("❌ WALLET_PRIVATE_KEY not set");
    }
    
    if let Ok(key) = std::env::var("HELIUS_API_KEY") {
        println!("🌐 Helius API: {}...{}", &key[..8], &key[key.len()-8..]);
    } else {
        println!("❌ HELIUS_API_KEY not set");
    }
    
    println!("\n📊 Available Strategies:");
    println!("   • Conservative: 15% profit target, 5% stop loss");
    println!("   • Aggressive: 50% profit target, 15% stop loss");
    println!("   • Conservative ATH: 8% pullback from ATH, 3% minimum profit ⭐ RECOMMENDED");
    println!("   • Aggressive ATH: 12% pullback from ATH, 5% minimum profit");
    
    println!("\n🎯 Supported Platforms (AUTO-DETECTED):");
    println!("   • PumpFun: For new meme coins on pump.fun");
    println!("   • Raydium: For established tokens with liquidity pools");
    println!("   • Jupiter: DEX aggregator for best prices (PRIMARY)");
    
    println!("\n🪙 Known Token Shortcuts:");
    println!("   • 'bonk' → {} (BONK)", token_addresses::BONK);
    println!("   • 'usdc' → {} (USDC)", token_addresses::USDC);
    println!("   • 'usdt' → {} (USDT)", token_addresses::USDT);
    println!("   • 'jup' → {} (JUP)", token_addresses::JUP);
    
    println!("\n⚙️ Configuration:");
    println!("   • Max trade size: 10.0 SOL");
    println!("   • Min trade size: 0.000001 SOL");
    println!("   • Default slippage: 1.0%");
    println!("   • Slippage range: 0.1% - 50.0%");
    println!("   • Priority fee: Dynamic (High priority)");
    println!("   • Jupiter API: v4 (Fixed)");
    
    println!("\n🔧 RECENT FIXES:");
    println!("   ✅ Fixed Jupiter API endpoint (v6 → v4)");
    println!("   ✅ Enhanced token address validation");
    println!("   ✅ Improved slippage parameter handling");
    println!("   ✅ Better error messages and troubleshooting");
    println!("   ✅ Enhanced platform detection logic");
    println!("   ✅ Comprehensive input validation");
    
    println!("\n💡 Usage Tips:");
    println!("   • Start with small amounts (0.001-0.01 SOL)");
    println!("   • Use Conservative ATH strategy for safer trading");
    println!("   • Monitor positions regularly with option 4");
    println!("   • Keep emergency sell (option 6) in mind for quick exits");
    println!("   • Test with known tokens (BONK, USDC) first");
    println!("   • Use slippage 1-3% for most trades");
    
    println!("\n🚨 Common Issues & Solutions:");
    println!("   • 400 API Error: Check token address validity, increase slippage");
    println!("   • Insufficient balance: Ensure enough SOL + 0.01 SOL for fees");
    println!("   • No liquidity: Try smaller amounts or different tokens");
    println!("   • Timeout errors: Network congestion, try again");
    
    println!("\n📝 Setup Instructions:");
    println!("   1. Create .env file in project root");
    println!("   2. Add: WALLET_PRIVATE_KEY=your_private_key_here");
    println!("   3. Add: HELIUS_API_KEY=your_helius_api_key_here");
    println!("   4. Use 'cargo run --bin key_checker' to verify key format");
    println!("   5. Start with 'cargo run --bin cli' for interactive trading");
}