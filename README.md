# Fast Solana Meme Trading Bot v0.3.1

Ultra-fast Solana meme coin trading bot with ATH (All-Time High) pullback strategies for automated profit-taking.

## 🚀 Features

- **Multi-Platform Trading**: Automatically detects and uses PumpFun, Raydium, or Jupiter
- **ATH Strategies**: Advanced pullback strategies that sell on drops from ATH
- **Real-time Monitoring**: Continuous position monitoring with auto-execution
- **Risk Management**: Built-in stop losses and profit targets
- **Emergency Controls**: Quick sell-all functionality for rapid exits
- **Performance Tracking**: Track win rates and P&L across all trades

## 📋 Prerequisites

1. **Rust**: Install from [rustup.rs](https://rustup.rs/)
2. **Solana Wallet**: A funded Solana wallet with private key
3. **Helius API Key**: Free tier available at [helius.xyz](https://helius.xyz)

## 🛠️ Setup

1. **Clone and build**:
   ```bash
   git clone <your-repo>
   cd fast-meme-trader
   cargo build --release
   ```

2. **Environment setup**:
   ```bash
   cp .env.template .env
   # Edit .env with your actual keys
   ```

3. **Configure .env**:
   ```bash
   WALLET_PRIVATE_KEY=your_base58_private_key_here
   HELIUS_API_KEY=your_helius_api_key_here
   ```

## 🎯 Trading Strategies

### Conservative Strategy
- **Profit Target**: 15%
- **Stop Loss**: 5%
- **Best For**: Safer, steady gains

### Aggressive Strategy  
- **Profit Target**: 50%
- **Stop Loss**: 15%
- **Best For**: High-risk, high-reward

### Conservative ATH Strategy
- **Pullback Trigger**: 8% from ATH
- **Minimum Profit**: 3%
- **Best For**: Riding pumps with safe exits

### Aggressive ATH Strategy
- **Pullback Trigger**: 12% from ATH  
- **Minimum Profit**: 5%
- **Best For**: Maximum pump exposure

## 🚀 Usage

### Interactive CLI Mode
```bash
cargo run --bin cli
```

### Quick Commands
```bash
# Health check
cargo run health

# Example trading session  
cargo run example

# Help
cargo run --help
```

## 🔧 CLI Menu Options

1. **🚀 Quick Buy**: Purchase tokens with strategy selection
2. **💰 Quick Sell**: Manually sell positions
3. **📊 View Positions**: See all active trades and P&L
4. **🎯 Monitor Positions**: Auto-trading with strategy execution
5. **📈 Check ATH Status**: View ATH tracking for positions
6. **🚨 Emergency Sell All**: Immediately liquidate all positions
7. **📋 Platform Test**: Test platform detection for tokens
8. **📊 Performance Stats**: View trading performance metrics
9. **🔧 Settings**: View configuration and help

## 💡 Quick Start Guide

1. **Start Small**: Begin with 0.001-0.01 SOL trades
2. **Choose Strategy**: Conservative ATH recommended for beginners
3. **Monitor Actively**: Use option 4 for automatic strategy execution
4. **Set Alerts**: Keep emergency sell option ready
5. **Track Performance**: Check stats regularly

## 🎯 Platform Detection

The bot automatically selects the best platform:

- **PumpFun**: For new meme coins on pump.fun
- **Raydium**: For established tokens with AMM pools
- **Jupiter**: DEX aggregator for best prices (fallback)

## ⚠️ Risk Warnings

- **High Risk**: Meme coin trading is extremely volatile
- **Start Small**: Never risk more than you can afford to lose
- **Monitor Positions**: Keep an eye on your trades
- **Emergency Exit**: Always have exit strategy ready
- **Private Keys**: Keep your wallet credentials secure

## 🔒 Security

- Never share your private keys
- Use a dedicated trading wallet
- Keep .env file secure and never commit it
- Consider using hardware wallets for large amounts

## 📊 Example Trading Session

```bash
# 1. Start CLI
cargo run --bin cli

# 2. Quick Buy
# Select option 1
# Enter token address or 'bonk'
# Amount: 0.01 (SOL)
# Strategy: Conservative ATH
# Slippage: 1.0%

# 3. Monitor Positions  
# Select option 4 to enable auto-trading
# Bot will execute strategy when conditions met

# 4. Check Performance
# Select option 8 for stats
```

## 🛠️ Configuration

### Environment Variables
- `WALLET_PRIVATE_KEY`: Your Solana wallet private key (base58)
- `HELIUS_API_KEY`: Your Helius RPC API key
- `BIRDEYE_API_KEY`: (Optional) Backup price feed API key

### Default Settings
- Max trade size: 10.0 SOL
- Min trade size: 0.001 SOL  
- Default slippage: 1.0%
- Priority fee: Dynamic (High priority)

## 🐛 Troubleshooting

### Build Errors
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

### Connection Issues
- Check Helius API key validity
- Verify internet connection
- Try different RPC endpoint

### Transaction Failures
- Increase slippage tolerance
- Check SOL balance for fees
- Retry with fresh transaction

## 📝 Token Shortcuts

- `bonk` → BONK token
- `usdc` → USDC token  
- `usdt` → USDT token
- `jup` → Jupiter token

## ⚡ Performance Tips

- Use release build for better performance: `cargo build --release`
- Monitor system resources during active trading
- Consider running on dedicated VPS for 24/7 monitoring
- Keep SOL balance sufficient for transaction fees

## 🤝 Contributing

1. Fork the repository
2. Create feature branch
3. Submit pull request
4. Follow Rust best practices

## 📄 License

[Add your license here]

## ⚠️ Disclaimer

This software is for educational purposes. Trading cryptocurrencies involves substantial risk. The authors are not responsible for any financial losses. Use at your own risk.