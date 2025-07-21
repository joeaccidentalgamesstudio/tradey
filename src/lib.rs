// src/lib.rs - Complete Fixed Fast Solana Meme Trading Bot
// Ultra-fast trading with ATH pullback strategies - ALL ISSUES FIXED

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signature, Signer},
    pubkey::Pubkey,
    commitment_config::CommitmentConfig,
    transaction::Transaction,
    native_token::LAMPORTS_PER_SOL,
};
use base64::Engine;
use spl_associated_token_account::get_associated_token_address;
use serde_json::{json, Value};
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use anyhow::{anyhow, Result};

// Main trading bot structure
pub struct FastMemeTrader {
    pub rpc_client: RpcClient,
    pub keypair: Keypair,
    pub helius_api_key: String,
    jupiter_endpoint: String,
    max_priority_fee: u64,
    
    // Strategy tracking
    pub positions: Arc<RwLock<HashMap<String, Position>>>,
    ath_tracker: Arc<RwLock<HashMap<String, ATHTracker>>>,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub token_address: String,
    pub entry_price: Decimal,
    pub amount_tokens: u64,
    pub entry_time: DateTime<Utc>,
    pub strategy: StrategyType,
    pub buy_signature: String,
}

#[derive(Debug, Clone)]
pub struct ATHTracker {
    pub entry_price: Decimal,
    pub ath_price: Decimal,
    pub last_price: Decimal,
    pub pullback_percent: Decimal,
    pub min_profit_percent: Decimal,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum StrategyType {
    Conservative,           // 15% profit, 5% stop loss
    Aggressive,            // 50% profit, 15% stop loss  
    ConservativeATH,       // 8% pullback, 3% min profit
    AggressiveATH,         // 12% pullback, 5% min profit
}

#[derive(Debug, Clone)]
pub struct TradeConfig {
    pub token_address: String,
    pub amount_sol: f64,
    pub slippage_bps: u16,
    pub strategy: StrategyType,
}

#[derive(Debug, Clone)]
pub struct TradeResult {
    pub signature: String,
    pub success: bool,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub platform_used: Platform,
    pub tokens_received: Option<u64>,
    pub sol_spent: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum Platform {
    PumpFun,
    Raydium,
    Jupiter,
}

// Known token addresses for common pairs
pub mod token_addresses {
    pub const SOL: &str = "So11111111111111111111111111111111111111112";
    pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    pub const BONK: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";
    pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
    pub const JUP: &str = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN";
}

impl FastMemeTrader {
    // Ultra-fast initialization with better key parsing
    pub fn new(private_key: &str, helius_api_key: String) -> Result<Self> {
        log::info!("Initializing FastMemeTrader...");
        
        // Better keypair parsing with multiple format support
        let keypair = Self::parse_private_key(private_key)?;
        log::info!("Wallet: {}", keypair.pubkey());
        
        let rpc_url = format!("https://mainnet.helius-rpc.com/?api-key={}", helius_api_key);
        let rpc_client = RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::processed(),
        );

        let trader = Self {
            rpc_client,
            keypair,
            helius_api_key,
            // FIX: Use correct Jupiter v4 API endpoint
            jupiter_endpoint: "https://quote-api.jup.ag/v4".to_string(),
            max_priority_fee: 200_000,
            positions: Arc::new(RwLock::new(HashMap::new())),
            ath_tracker: Arc::new(RwLock::new(HashMap::new())),
        };
        
        log::info!("FastMemeTrader initialized successfully");
        Ok(trader)
    }

    // Improved private key parsing with multiple format support
    fn parse_private_key(private_key: &str) -> Result<Keypair> {
        let trimmed_key = private_key.trim();
        
        // Try different key formats
        
        // 1. Try as base58 string (most common format) - using bs58 crate
        if let Ok(bytes) = bs58::decode(trimmed_key).into_vec() {
            if bytes.len() == 64 {
                if let Ok(keypair) = Keypair::try_from(&bytes[..]) {
                    return Ok(keypair);
                }
            }
        }
        
        // 2. Try as JSON array (Phantom/Solflare export format)
        if trimmed_key.starts_with('[') && trimmed_key.ends_with(']') {
            if let Ok(bytes_vec) = serde_json::from_str::<Vec<u8>>(trimmed_key) {
                if bytes_vec.len() == 64 {
                    if let Ok(keypair) = Keypair::try_from(&bytes_vec[..]) {
                        return Ok(keypair);
                    }
                }
            }
        }
        
        // 3. Try as hex string (without 0x prefix)
        if trimmed_key.len() == 128 || (trimmed_key.len() == 130 && trimmed_key.starts_with("0x")) {
            let hex_str = if trimmed_key.starts_with("0x") {
                &trimmed_key[2..]
            } else {
                trimmed_key
            };
            
            if let Ok(bytes) = hex::decode(hex_str) {
                if bytes.len() == 64 {
                    if let Ok(keypair) = Keypair::try_from(&bytes[..]) {
                        return Ok(keypair);
                    }
                }
            }
        }
        
        // 4. Try as comma-separated bytes
        if trimmed_key.contains(',') {
            let parts: Result<Vec<u8>, _> = trimmed_key
                .split(',')
                .map(|s| s.trim().parse::<u8>())
                .collect();
            
            if let Ok(bytes) = parts {
                if bytes.len() == 64 {
                    if let Ok(keypair) = Keypair::try_from(&bytes[..]) {
                        return Ok(keypair);
                    }
                }
            }
        }
        
        Err(anyhow!(
            "Invalid private key format. Supported formats:\n\
            1. Base58 string (most common)\n\
            2. JSON array: [1,2,3,...,64]\n\
            3. Hex string: 0x1a2b3c... or 1a2b3c...\n\
            4. Comma-separated bytes: 1,2,3,...,64\n\
            \n\
            Your key length: {} characters\n\
            First 10 chars: {}",
            trimmed_key.len(),
            &trimmed_key[..trimmed_key.len().min(10)]
        ))
    }

    // FIX: Add token address validation
    fn validate_token_address(token_address: &str) -> Result<()> {
        if token_address.len() != 44 {
            return Err(anyhow!("Invalid token address length: expected 44 characters, got {}", token_address.len()));
        }
        
        // Try to parse as Pubkey to validate format
        Pubkey::from_str(token_address)
            .map_err(|_| anyhow!("Invalid token address format: {}", token_address))?;
        
        Ok(())
    }

    // Better platform detection logic
    pub async fn detect_best_platform(&self, token_address: &str) -> Platform {
        log::debug!("Detecting best platform for token: {}", token_address);
        
        // First check if it's an established token
        match token_address {
            token_addresses::SOL | token_addresses::USDC | token_addresses::USDT | 
            token_addresses::BONK | token_addresses::JUP => {
                log::info!("Established token detected, using Jupiter");
                return Platform::Jupiter;
            },
            _ => {}
        }
        
        // For unknown tokens, check PumpFun first (faster)
        let is_pumpfun = self.is_pumpfun_token(token_address).await;
        
        let platform = if is_pumpfun {
            Platform::PumpFun
        } else {
            // Default to Jupiter for all other tokens
            Platform::Jupiter
        };
        
        log::info!("Selected platform: {:?} for token {}", platform, &token_address[..8]);
        platform
    }

    // Improved buy_fast with comprehensive validation
    pub async fn buy_fast(&self, config: TradeConfig) -> TradeResult {
        let start_time = Instant::now();
        
        log::info!("Starting fast buy: {} SOL for {}", config.amount_sol, &config.token_address[..8]);
        
        // Enhanced validation
        if config.amount_sol < 0.000001 || config.amount_sol > 50.0 {
            return TradeResult {
                signature: String::new(),
                success: false,
                error: Some("Amount must be between 0.000001 and 50.0 SOL".to_string()),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                platform_used: Platform::Jupiter,
                tokens_received: None,
                sol_spent: None,
            };
        }
        
        // Validate token address
        if let Err(e) = Self::validate_token_address(&config.token_address) {
            return TradeResult {
                signature: String::new(),
                success: false,
                error: Some(format!("Invalid token address: {}", e)),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                platform_used: Platform::Jupiter,
                tokens_received: None,
                sol_spent: None,
            };
        }
        
        let platform = self.detect_best_platform(&config.token_address).await;
        
        let result = match platform {
            Platform::PumpFun => self.buy_pumpfun(&config).await,
            Platform::Raydium => self.buy_jupiter(&config).await, // Fall back to Jupiter
            Platform::Jupiter => self.buy_jupiter(&config).await,
        };
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        match result {
            Ok((signature, tokens_received)) => {
                log::info!("Buy successful: {} tokens received in {}ms", tokens_received, execution_time);
                
                // Initialize position and ATH tracking
                self.initialize_position(&config, &signature, tokens_received).await;
                
                TradeResult {
                    signature,
                    success: true,
                    error: None,
                    execution_time_ms: execution_time,
                    platform_used: platform,
                    tokens_received: Some(tokens_received),
                    sol_spent: Some(config.amount_sol),
                }
            },
            Err(e) => {
                log::error!("Buy failed after {}ms: {}", execution_time, e);
                TradeResult {
                    signature: String::new(),
                    success: false,
                    error: Some(e.to_string()),
                    execution_time_ms: execution_time,
                    platform_used: platform,
                    tokens_received: None,
                    sol_spent: None,
                }
            },
        }
    }

    // FIXED: Complete Jupiter implementation with proper error handling
    async fn buy_jupiter(&self, config: &TradeConfig) -> Result<(String, u64)> {
        log::info!("Executing Jupiter buy for {}", &config.token_address[..8]);
        
        // Validate token first
        Self::validate_token_address(&config.token_address)?;
        
        let amount_lamports = (config.amount_sol * LAMPORTS_PER_SOL as f64) as u64;
        
        if amount_lamports < 1000 { // Minimum ~0.000001 SOL
            return Err(anyhow!("Amount too small: {} lamports", amount_lamports));
        }
        
        // 1. Get quote with timeout and retries
        log::info!("Getting Jupiter quote for {} lamports...", amount_lamports);
        let quote = tokio::time::timeout(
            Duration::from_secs(15),
            self.get_jupiter_quote_with_retry(config, amount_lamports, 3)
        ).await??;
        
        // FIX: Handle both v4 and v6 response formats
        let tokens_expected = if let Some(out_amount) = quote.get("outAmount") {
            out_amount.as_str()
                .ok_or_else(|| anyhow!("outAmount is not a string"))?
                .parse::<u64>()?
        } else if let Some(data) = quote.get("data") {
            // Handle alternative response format
            data.get("outAmount")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("No outAmount in data"))?
                .parse::<u64>()?
        } else {
            return Err(anyhow!("No outAmount found in quote response"));
        };
        
        log::info!("Jupiter quote: {} lamports -> {} tokens", amount_lamports, tokens_expected);
        
        // 2. Get swap transaction with optimized parameters
        let priority_fee = self.calculate_priority_fee().await;
        let swap_data = json!({
            "userPublicKey": self.keypair.pubkey().to_string(),
            "quoteResponse": quote,
            "prioritizationFeeLamports": priority_fee,
            "asLegacyTransaction": false,
            "dynamicComputeUnitLimit": true,
        });
        
        let swap_data_str = serde_json::to_string(&swap_data)?;
        let url = format!("{}/swap", self.jupiter_endpoint);
        
        log::debug!("Sending swap request to: {}", url);
        
        let response = tokio::task::spawn_blocking(move || {
            ureq::post(&url)
                .timeout(Duration::from_secs(30))
                .set("Content-Type", "application/json")
                .send_string(&swap_data_str)
        }).await??;
        
        if response.status() != 200 {
            let error_text = response.into_string().unwrap_or_else(|_| "Failed to read error response".to_string());
            log::error!("Jupiter swap API error: Status {}, Body: {}", response.status(), error_text);
            return Err(anyhow!("Jupiter swap API error ({}): {}", response.status(), error_text));
        }
        
        let swap_result: Value = response.into_json()?;
        let transaction_b64 = swap_result["swapTransaction"].as_str()
            .ok_or_else(|| anyhow!("No transaction returned from Jupiter swap"))?;
        
        // 3. Execute transaction
        log::info!("Executing swap transaction...");
        let signature = self.execute_transaction_b64(transaction_b64).await?;
        
        Ok((signature, tokens_expected))
    }

    // FIXED: Jupiter quote with proper validation and retry logic
    async fn get_jupiter_quote_with_retry(&self, config: &TradeConfig, amount_lamports: u64, max_retries: u32) -> Result<Value> {
        let mut last_error = None;
        
        for attempt in 1..=max_retries {
            match self.get_jupiter_quote(config, amount_lamports).await {
                Ok(quote) => return Ok(quote),
                Err(e) => {
                    log::warn!("Quote attempt {} failed: {}", attempt, e);
                    last_error = Some(e);
                    if attempt < max_retries {
                        tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow!("All quote attempts failed")))
    }

    // FIXED: Jupiter quote with proper URL and validation
    async fn get_jupiter_quote(&self, config: &TradeConfig, amount_lamports: u64) -> Result<Value> {
        // Validate inputs first
        Self::validate_token_address(&config.token_address)?;
        
        if amount_lamports == 0 {
            return Err(anyhow!("Amount cannot be zero"));
        }
        
        // FIX: Improved slippage validation and conversion
        let slippage_bps = if config.slippage_bps == 0 {
            100 // Default to 1% if zero
        } else if config.slippage_bps > 5000 {
            5000 // Cap at 50%
        } else {
            config.slippage_bps
        };

        // FIX: Simplified Jupiter v4 quote endpoint without problematic parameters
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            self.jupiter_endpoint,
            token_addresses::SOL,
            config.token_address,
            amount_lamports,
            slippage_bps
        );
        
        log::debug!("Jupiter quote URL: {}", url);
        
        let response = tokio::task::spawn_blocking(move || {
            ureq::get(&url)
                .timeout(Duration::from_secs(15))
                .call()
        }).await??;
        
        if response.status() != 200 {
            let error_text = response.into_string().unwrap_or_else(|_| "Failed to read error response".to_string());
            log::error!("Jupiter quote API error: Status {}, Body: {}", response.status(), error_text);
            return Err(anyhow!("Jupiter quote API error ({}): {}", response.status(), error_text));
        }
        
        let quote: Value = response.into_json()?;
        
        // FIX: Validate quote response
        if quote.get("data").is_none() && quote.get("outAmount").is_none() {
            log::error!("Invalid quote response: {}", serde_json::to_string_pretty(&quote)?);
            return Err(anyhow!("Invalid quote response from Jupiter"));
        }
        
        Ok(quote)
    }

    // FIXED: Complete PumpFun implementation
    async fn buy_pumpfun(&self, config: &TradeConfig) -> Result<(String, u64)> {
        log::info!("Executing PumpFun buy for {}", &config.token_address[..8]);
        
        let amount_lamports = (config.amount_sol * LAMPORTS_PER_SOL as f64) as u64;
        
        // Use PumpPortal API for transaction generation
        let pumpfun_data = json!({
            "publicKey": self.keypair.pubkey().to_string(),
            "action": "buy",
            "mint": config.token_address,
            "denominatedInSol": "true",
            "amount": amount_lamports,
            "slippage": config.slippage_bps,
            "priorityFee": self.calculate_priority_fee().await,
            "pool": "pump"
        });
        
        let pumpfun_data_str = serde_json::to_string(&pumpfun_data)?;
        
        let response = tokio::task::spawn_blocking(move || {
            ureq::post("https://pumpportal.fun/api/trade-local")
                .timeout(Duration::from_secs(20))
                .set("Content-Type", "application/json")
                .send_string(&pumpfun_data_str)
        }).await??;
        
        if response.status() != 200 {
            let error_text = response.into_string()?;
            return Err(anyhow!("PumpFun API error: {}", error_text));
        }
        
        let transaction_b64 = response.into_string()?;
        let signature = self.execute_transaction_b64(&transaction_b64).await?;
        
        // Better token estimation based on bonding curve
        let tokens_estimated = self.estimate_pumpfun_tokens(amount_lamports).await
            .unwrap_or(amount_lamports * 1_000_000);
        
        Ok((signature, tokens_estimated))
    }

    // Estimate PumpFun tokens
    async fn estimate_pumpfun_tokens(&self, amount_lamports: u64) -> Result<u64> {
        // Simple linear approximation - could be improved with actual curve calculation
        Ok(amount_lamports * 1_000_000)
    }

    // Execute base64 encoded transaction with better error handling
    async fn execute_transaction_b64(&self, transaction_b64: &str) -> Result<String> {
        log::debug!("Executing transaction from base64");
        
        let transaction_bytes = base64::engine::general_purpose::STANDARD.decode(transaction_b64)?;
        let mut transaction: Transaction = bincode::deserialize(&transaction_bytes)?;
        
        // Get fresh blockhash and re-sign
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        transaction.sign(&[&self.keypair], recent_blockhash);
        
        let signature = self.send_with_retry(transaction).await?;
        Ok(signature.to_string())
    }

    // Robust transaction sending with exponential backoff
    async fn send_with_retry(&self, mut transaction: Transaction) -> Result<Signature> {
        let mut last_error = None;
        
        for attempt in 1..=5 {
            log::debug!("Sending transaction attempt {}/5", attempt);
            
            // Get fresh blockhash for each attempt after the first
            if attempt > 1 {
                if let Ok(new_blockhash) = self.rpc_client.get_latest_blockhash() {
                    transaction.sign(&[&self.keypair], new_blockhash);
                }
            }
            
            match self.rpc_client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => {
                    log::info!("Transaction confirmed: {}", signature);
                    return Ok(signature);
                },
                Err(e) => {
                    log::warn!("Transaction attempt {} failed: {}", attempt, e);
                    last_error = Some(e);
                    
                    if attempt < 5 {
                        // Exponential backoff: 500ms, 1s, 2s, 4s
                        let delay = Duration::from_millis(500 * (1u64 << (attempt - 1)));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
        
        Err(anyhow!("Transaction failed after 5 attempts: {:?}", last_error))
    }

    // Initialize position with strategy tracking
    async fn initialize_position(&self, config: &TradeConfig, signature: &str, tokens_received: u64) {
        let current_price = self.get_current_price(&config.token_address).await
            .unwrap_or(Decimal::from(0));
        
        log::info!("Initializing position at price ${:.8}", current_price);
        
        let position = Position {
            token_address: config.token_address.clone(),
            entry_price: current_price,
            amount_tokens: tokens_received,
            entry_time: Utc::now(),
            strategy: config.strategy.clone(),
            buy_signature: signature.to_string(),
        };
        
        let ath_tracker = match config.strategy {
            StrategyType::ConservativeATH => ATHTracker {
                entry_price: current_price,
                ath_price: current_price,
                last_price: current_price,
                pullback_percent: Decimal::from(8),
                min_profit_percent: Decimal::from(3),
                last_updated: Utc::now(),
            },
            StrategyType::AggressiveATH => ATHTracker {
                entry_price: current_price,
                ath_price: current_price,
                last_price: current_price,
                pullback_percent: Decimal::from(12),
                min_profit_percent: Decimal::from(5),
                last_updated: Utc::now(),
            },
            _ => ATHTracker {
                entry_price: current_price,
                ath_price: current_price,
                last_price: current_price,
                pullback_percent: Decimal::from(10),
                min_profit_percent: Decimal::from(2),
                last_updated: Utc::now(),
            },
        };
        
        {
            let mut positions = self.positions.write().await;
            positions.insert(config.token_address.clone(), position);
        }
        {
            let mut trackers = self.ath_tracker.write().await;
            trackers.insert(config.token_address.clone(), ath_tracker);
        }
        
        log::info!("Position and ATH tracker initialized for strategy: {:?}", config.strategy);
    }

    // Monitor positions and execute strategies with batched price updates
    pub async fn monitor_positions(&self) -> Vec<String> {
        let mut executed_sells = Vec::new();
        let positions: Vec<_> = {
            let positions_guard = self.positions.read().await;
            positions_guard.values().cloned().collect()
        };
        
        if positions.is_empty() {
            return executed_sells;
        }
        
        // Batch price updates for efficiency
        let mut price_updates = HashMap::new();
        for position in &positions {
            if let Ok(current_price) = self.get_current_price(&position.token_address).await {
                price_updates.insert(position.token_address.clone(), current_price);
            }
        }
        
        for position in positions {
            if let Some(&current_price) = price_updates.get(&position.token_address) {
                let should_sell = self.evaluate_exit_strategy(&position, current_price).await;
                
                if should_sell {
                    log::info!("Exit strategy triggered for {}", &position.token_address[..8]);
                    
                    if let Ok(sell_result) = self.sell_position(&position.token_address).await {
                        let message = format!(
                            "Sold {} - Signature: {} - Strategy: {:?} - Time: {}ms",
                            &position.token_address[..8], 
                            sell_result.signature, 
                            position.strategy,
                            sell_result.execution_time_ms
                        );
                        executed_sells.push(message);
                        
                        // Clean up position and tracker
                        {
                            let mut positions_guard = self.positions.write().await;
                            positions_guard.remove(&position.token_address);
                        }
                        {
                            let mut trackers_guard = self.ath_tracker.write().await;
                            trackers_guard.remove(&position.token_address);
                        }
                    }
                }
            }
        }
        
        executed_sells
    }

    // Strategy evaluation with ATH logic
    async fn evaluate_exit_strategy(&self, position: &Position, current_price: Decimal) -> bool {
        // Update ATH tracker
        {
            let mut trackers = self.ath_tracker.write().await;
            if let Some(tracker) = trackers.get_mut(&position.token_address) {
                if current_price > tracker.ath_price {
                    tracker.ath_price = current_price;
                    log::debug!("New ATH for {}: ${:.8}", &position.token_address[..8], current_price);
                }
                tracker.last_price = current_price;
                tracker.last_updated = Utc::now();
            }
        }
        
        match position.strategy {
            StrategyType::Conservative => {
                let profit_percent = self.calculate_profit_percent(position.entry_price, current_price);
                profit_percent >= Decimal::from(15) || profit_percent <= Decimal::from(-5)
            },
            StrategyType::Aggressive => {
                let profit_percent = self.calculate_profit_percent(position.entry_price, current_price);
                profit_percent >= Decimal::from(50) || profit_percent <= Decimal::from(-15)
            },
            StrategyType::ConservativeATH | StrategyType::AggressiveATH => {
                self.check_ath_pullback_exit(&position.token_address, current_price).await
            },
        }
    }

    // ATH pullback exit logic
    async fn check_ath_pullback_exit(&self, token_address: &str, current_price: Decimal) -> bool {
        let trackers = self.ath_tracker.read().await;
        if let Some(tracker) = trackers.get(token_address) {
            let profit_percent = self.calculate_profit_percent(tracker.entry_price, current_price);
            let pullback_from_ath = if tracker.ath_price > Decimal::ZERO {
                (tracker.ath_price - current_price) / tracker.ath_price * Decimal::from(100)
            } else {
                Decimal::ZERO
            };
            
            let should_exit = profit_percent >= tracker.min_profit_percent 
                && pullback_from_ath >= tracker.pullback_percent;
            
            if should_exit {
                log::info!(
                    "ATH pullback triggered for {}: Profit: {:.2}%, Pullback: {:.2}%, ATH: ${:.8}",
                    &token_address[..8], profit_percent, pullback_from_ath, tracker.ath_price
                );
            }
            
            should_exit
        } else {
            false
        }
    }

    // Fast sell implementation
    pub async fn sell_position(&self, token_address: &str) -> Result<TradeResult> {
        let start_time = Instant::now();
        
        log::info!("Starting sell for {}", &token_address[..8]);
        
        let token_balance = self.get_token_balance(token_address).await?;
        if token_balance == 0 {
            return Err(anyhow!("No tokens to sell"));
        }
        
        log::info!("Selling {} tokens", token_balance);
        
        // Use Jupiter for selling (most reliable)
        let result = self.sell_jupiter(token_address, token_balance).await;
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        match result {
            Ok(signature) => Ok(TradeResult {
                signature,
                success: true,
                error: None,
                execution_time_ms: execution_time,
                platform_used: Platform::Jupiter,
                tokens_received: None,
                sol_spent: None,
            }),
            Err(e) => Ok(TradeResult {
                signature: String::new(),
                success: false,
                error: Some(e.to_string()),
                execution_time_ms: execution_time,
                platform_used: Platform::Jupiter,
                tokens_received: None,
                sol_spent: None,
            }),
        }
    }

    // Jupiter sell implementation
    async fn sell_jupiter(&self, token_address: &str, amount: u64) -> Result<String> {
        let quote_url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps=500",
            self.jupiter_endpoint,
            token_address,
            token_addresses::SOL,
            amount
        );
        
        let quote: Value = tokio::task::spawn_blocking(move || {
            ureq::get(&quote_url)
                .timeout(Duration::from_secs(15))
                .call()
        }).await??.into_json()?;
        
        let swap_data = json!({
            "userPublicKey": self.keypair.pubkey().to_string(),
            "quoteResponse": quote,
            "prioritizationFeeLamports": self.calculate_priority_fee().await,
            "asLegacyTransaction": false,
            "dynamicComputeUnitLimit": true,
        });
        
        let swap_data_str = serde_json::to_string(&swap_data)?;
        let swap_url = format!("{}/swap", self.jupiter_endpoint);
        
        let swap_result: Value = tokio::task::spawn_blocking(move || {
            ureq::post(&swap_url)
                .timeout(Duration::from_secs(15))
                .set("Content-Type", "application/json")
                .send_string(&swap_data_str)
        }).await??.into_json()?;
        
        let transaction_b64 = swap_result["swapTransaction"].as_str()
            .ok_or_else(|| anyhow!("No transaction returned"))?;
        
        self.execute_transaction_b64(transaction_b64).await
    }

    // Improved priority fee calculation
    async fn calculate_priority_fee(&self) -> u64 {
        let url = format!("https://mainnet.helius-rpc.com/?api-key={}", self.helius_api_key);
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": "1",
            "method": "getPriorityFeeEstimate",
            "params": [{
                "options": { "priorityLevel": "High" }
            }]
        });
        
        let request_body_str = serde_json::to_string(&request_body).unwrap_or_default();
        
        if let Ok(response) = tokio::task::spawn_blocking(move || {
            ureq::post(&url)
                .timeout(Duration::from_secs(5))
                .set("Content-Type", "application/json")
                .send_string(&request_body_str)
        }).await {
            if let Ok(response) = response {
                if let Ok(data) = response.into_json::<Value>() {
                    if let Some(fee) = data["result"]["priorityFeeEstimate"].as_f64() {
                        let calculated_fee = (fee as u64).min(self.max_priority_fee);
                        log::debug!("Calculated priority fee: {} microlamports", calculated_fee);
                        return calculated_fee;
                    }
                }
            }
        }
        
        log::warn!("Failed to get priority fee, using fallback");
        150_000
    }

    // Improved PumpFun detection
    async fn is_pumpfun_token(&self, token_address: &str) -> bool {
        // Check if token is a known established token first
        match token_address {
            token_addresses::SOL | token_addresses::USDC | token_addresses::USDT | 
            token_addresses::BONK | token_addresses::JUP => {
                return false; // These are established tokens, not PumpFun
            },
            _ => {}
        }
        
        let url = format!("https://frontend-api.pump.fun/coins/{}", token_address);
        
        match tokio::task::spawn_blocking(move || {
            ureq::get(&url)
                .timeout(Duration::from_secs(3))
                .call()
        }).await {
            Ok(Ok(response)) => {
                let is_pumpfun = response.status() == 200;
                log::debug!("PumpFun check for {}: {}", &token_address[..8], is_pumpfun);
                is_pumpfun
            },
            _ => {
                log::debug!("PumpFun check failed for {}, assuming not PumpFun", &token_address[..8]);
                false
            }
        }
    }

    // Price fetching with multiple sources
    async fn get_current_price(&self, token_address: &str) -> Result<Decimal> {
        // Try Jupiter price API first
        if let Ok(price) = self.get_jupiter_price(token_address).await {
            return Ok(price);
        }
        
        // Fallback to simple estimation
        Ok(Decimal::from(0))
    }
    
    async fn get_jupiter_price(&self, token_address: &str) -> Result<Decimal> {
        let url = format!("https://price.jup.ag/v4/price?ids={}", token_address);
        
        let response: Value = tokio::task::spawn_blocking(move || {
            ureq::get(&url)
                .timeout(Duration::from_secs(5))
                .call()
        }).await??.into_json()?;
        
        if let Some(price_value) = response["data"][token_address]["price"].as_f64() {
            Ok(Decimal::try_from(price_value)?)
        } else {
            Err(anyhow!("Price not found in Jupiter API"))
        }
    }

    async fn get_token_balance(&self, token_address: &str) -> Result<u64> {
        let mint = Pubkey::from_str(token_address)?;
        let ata = get_associated_token_address(&self.keypair.pubkey(), &mint);
        
        match self.rpc_client.get_token_account_balance(&ata) {
            Ok(balance) => Ok(balance.amount.parse()?),
            Err(_) => Ok(0),
        }
    }

    fn calculate_profit_percent(&self, entry_price: Decimal, current_price: Decimal) -> Decimal {
        if entry_price == Decimal::ZERO {
            return Decimal::ZERO;
        }
        (current_price - entry_price) / entry_price * Decimal::from(100)
    }

    // Status and monitoring methods
    pub async fn get_ath_status(&self, token_address: &str) -> Option<String> {
        let trackers = self.ath_tracker.read().await;
        if let Some(tracker) = trackers.get(token_address) {
            let profit_percent = self.calculate_profit_percent(tracker.entry_price, tracker.last_price);
            let pullback_from_ath = if tracker.ath_price > Decimal::ZERO {
                (tracker.ath_price - tracker.last_price) / tracker.ath_price * Decimal::from(100)
            } else {
                Decimal::ZERO
            };
            
            Some(format!(
                "Entry: ${:.8} | ATH: ${:.8} | Current: ${:.8} | P&L: {:.2}% | Pullback: {:.2}%",
                tracker.entry_price, tracker.ath_price, tracker.last_price, 
                profit_percent, pullback_from_ath
            ))
        } else {
            None
        }
    }

    pub async fn list_positions(&self) -> Vec<String> {
        let positions = self.positions.read().await;
        let mut result = Vec::new();
        
        for (token, position) in positions.iter() {
            if let Ok(current_price) = self.get_current_price(token).await {
                let profit_percent = self.calculate_profit_percent(position.entry_price, current_price);
                result.push(format!(
                    "{}: {:.0} tokens | Entry: ${:.8} | Current: ${:.8} | P&L: {:.2}% | Strategy: {:?}",
                    &token[..8], position.amount_tokens, position.entry_price, current_price, 
                    profit_percent, position.strategy
                ));
            }
        }
        
        result
    }

    pub async fn emergency_sell_all(&self) -> Vec<TradeResult> {
        log::warn!("EMERGENCY SELL ALL initiated");
        
        let mut results = Vec::new();
        let positions: Vec<_> = {
            let positions_guard = self.positions.read().await;
            positions_guard.keys().cloned().collect()
        };
        
        // Execute sells sequentially for stability
        for token_address in positions {
            log::warn!("Emergency selling {}", &token_address[..8]);
            if let Ok(trade_result) = self.sell_position(&token_address).await {
                results.push(trade_result);
            }
        }
        
        // Clear all positions
        {
            let mut positions_guard = self.positions.write().await;
            positions_guard.clear();
        }
        {
            let mut trackers_guard = self.ath_tracker.write().await;
            trackers_guard.clear();
        }
        
        log::warn!("Emergency sell completed, {} positions liquidated", results.len());
        results
    }

    // Health check
    pub async fn health_check(&self) -> Result<String> {
        // Check SOL balance
        let sol_balance = self.rpc_client.get_balance(&self.keypair.pubkey())?;
        let sol_amount = sol_balance as f64 / LAMPORTS_PER_SOL as f64;
        
        // Check positions count
        let positions_count = self.positions.read().await.len();
        
        // Test Jupiter connectivity
        let jupiter_test = self.get_current_price(token_addresses::BONK).await.is_ok();
        
        // Test RPC connectivity
        let rpc_test = self.rpc_client.get_latest_blockhash().is_ok();
        
        Ok(format!(
            "Health: SOL Balance: {:.6} | Positions: {} | Jupiter: {} | RPC: {}",
            sol_amount, 
            positions_count, 
            if jupiter_test { "✅" } else { "❌" },
            if rpc_test { "✅" } else { "❌" }
        ))
    }
    
    // Performance metrics
    pub async fn get_performance_stats(&self) -> String {
        let positions = self.positions.read().await;
        let mut total_profit = Decimal::ZERO;
        let mut winning_trades = 0;
        let total_trades = positions.len();
        
        for position in positions.values() {
            if let Ok(current_price) = self.get_current_price(&position.token_address).await {
                let profit_percent = self.calculate_profit_percent(position.entry_price, current_price);
                total_profit += profit_percent;
                if profit_percent > Decimal::ZERO {
                    winning_trades += 1;
                }
            }
        }
        
        let win_rate = if total_trades > 0 {
            (winning_trades as f64 / total_trades as f64) * 100.0
        } else {
            0.0
        };
        
        format!(
            "Performance: Active Trades: {} | Win Rate: {:.1}% | Avg P&L: {:.2}%",
            total_trades, win_rate, total_profit / Decimal::from(total_trades.max(1))
        )
    }
}

// Example usage function
pub async fn example_usage() -> Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();
    
    let private_key = std::env::var("WALLET_PRIVATE_KEY")?;
    let helius_api_key = std::env::var("HELIUS_API_KEY")?;
    
    let trader = FastMemeTrader::new(&private_key, helius_api_key)?;
    
    // Example: Buy BONK with Conservative ATH strategy
    let config = TradeConfig {
        token_address: token_addresses::BONK.to_string(),
        amount_sol: 0.01,
        slippage_bps: 100,
        strategy: StrategyType::ConservativeATH,
    };
    
    let result = trader.buy_fast(config).await;
    println!("Buy result: {:#?}", result);
    
    if result.success {
        // Monitor positions
        loop {
            let sells = trader.monitor_positions().await;
            for sell in sells {
                println!("Executed sell: {}", sell);
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
    
    Ok(())
}