// src/bin/key_checker.rs - Utility to check and convert private key formats

use solana_sdk::signature::{Keypair, Signer};
use anyhow::Result;
use std::io::{self, Write};
use bs58;

fn main() -> Result<()> {
    println!("🔑 Solana Private Key Format Checker");
    println!("===================================");
    
    print!("Enter your private key: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let private_key = input.trim();
    
    println!("\n🔍 Analyzing key format...");
    println!("Length: {} characters", private_key.len());
    println!("First 10 chars: {}", &private_key[..private_key.len().min(10)]);
    
    // Try different formats
    match try_parse_key(private_key) {
        Ok(keypair) => {
            println!("✅ SUCCESS! Key is valid");
            println!("🔑 Public Key: {}", keypair.pubkey());
            println!("📝 Base58 format: {}", keypair.to_base58_string());
        },
        Err(e) => {
            println!("❌ FAILED to parse key: {}", e);
            print_format_help();
        }
    }
    
    Ok(())
}

fn try_parse_key(private_key: &str) -> Result<Keypair> {
    let trimmed_key = private_key.trim();
    
    println!("🔄 Trying different formats...");
    
    // 1. Try as base58 string (most common format)
    println!("   → Trying base58 format...");
    match bs58::decode(trimmed_key).into_vec() {
        Ok(bytes) => {
            if bytes.len() == 64 {
                match Keypair::try_from(bytes.as_slice()) {
                    Ok(keypair) => {
                        println!("   ✅ Base58 format works!");
                        return Ok(keypair);
                    }
                    Err(_) => {
                        println!("   ❌ Failed to create keypair from base58 bytes");
                    }
                }
            } else {
                println!("   ❌ Expected 64 bytes from base58, got {}", bytes.len());
            }
        }
        Err(_) => {
            println!("   ❌ Invalid base58 string");
        }
    }
    
    // 2. Try as JSON array (Phantom/Solflare export format)
    println!("   → Trying JSON array format...");
    if trimmed_key.starts_with('[') && trimmed_key.ends_with(']') {
        if let Ok(bytes_vec) = serde_json::from_str::<Vec<u8>>(trimmed_key) {
            println!("     JSON array has {} bytes", bytes_vec.len());
            if bytes_vec.len() == 64 {
                match Keypair::try_from(bytes_vec.as_slice()) {
                    Ok(keypair) => {
                        println!("   ✅ JSON array format works!");
                        return Ok(keypair);
                    }
                    Err(_) => {
                        println!("   ❌ Failed to create keypair from JSON array bytes");
                    }
                }
            } else {
                println!("     ❌ Expected 64 bytes, got {}", bytes_vec.len());
            }
        }
    }
    
    // 3. Try as hex string (without 0x prefix)
    println!("   → Trying hex format...");
    if trimmed_key.len() == 128 || (trimmed_key.len() == 130 && trimmed_key.starts_with("0x")) {
        let hex_str = if trimmed_key.starts_with("0x") {
            &trimmed_key[2..]
        } else {
            trimmed_key
        };
        
        if let Ok(bytes) = hex::decode(hex_str) {
            println!("     Hex decoded to {} bytes", bytes.len());
            if bytes.len() == 64 {
                match Keypair::try_from(bytes.as_slice()) {
                    Ok(keypair) => {
                        println!("   ✅ Hex format works!");
                        return Ok(keypair);
                    }
                    Err(_) => {
                        println!("   ❌ Failed to create keypair from hex bytes");
                    }
                }
            } else {
                println!("     ❌ Expected 64 bytes, got {}", bytes.len());
            }
        } else {
            println!("     ❌ Invalid hex string");
        }
    }
    
    // 4. Try as comma-separated bytes
    println!("   → Trying comma-separated bytes...");
    if trimmed_key.contains(',') {
        let parts: Result<Vec<u8>, _> = trimmed_key
            .split(',')
            .map(|s| s.trim().parse::<u8>())
            .collect();
        
        if let Ok(bytes) = parts {
            println!("     Parsed {} bytes from comma-separated format", bytes.len());
            if bytes.len() == 64 {
                match Keypair::try_from(bytes.as_slice()) {
                    Ok(keypair) => {
                        println!("   ✅ Comma-separated format works!");
                        return Ok(keypair);
                    }
                    Err(_) => {
                        println!("   ❌ Failed to create keypair from comma-separated bytes");
                    }
                }
            } else {
                println!("     ❌ Expected 64 bytes, got {}", bytes.len());
            }
        } else {
            println!("     ❌ Failed to parse comma-separated bytes");
        }
    }
    
    Err(anyhow::anyhow!("All format attempts failed"))
}

fn print_format_help() {
    println!("\n📖 PRIVATE KEY FORMAT GUIDE");
    println!("===========================");
    
    println!("\n1. 🎯 BASE58 FORMAT (Most Common)");
    println!("   • Used by: Solana CLI, most wallets");
    println!("   • Length: ~88 characters");
    println!("   • Example: 5Kd3NBUAdUnhyzenEwVLy9pBKxSwXvE9FMPyR4UKZvpe6E6VuvYvQHn5VDkNBhECGNRtSFLhXMkgVdmRjXFKMMY");
    
    println!("\n2. 📱 JSON ARRAY FORMAT (Phantom/Solflare Export)");
    println!("   • Used by: Phantom, Solflare wallets");
    println!("   • Format: [1,2,3,...,64] (array of 64 numbers)");
    println!("   • Example: [123,45,67,...,255] (64 numbers total)");
    
    println!("\n3. 🔢 HEX FORMAT");
    println!("   • Length: 128 characters (or 130 with 0x prefix)");
    println!("   • Example: 1a2b3c4d5e6f... (128 hex characters)");
    println!("   • Example: 0x1a2b3c4d5e6f... (with 0x prefix)");
    
    println!("\n4. 📝 COMMA-SEPARATED BYTES");
    println!("   • Format: 1,2,3,...,64 (64 numbers separated by commas)");
    println!("   • Example: 123,45,67,89,12,34,56,78...");
    
    println!("\n🚀 HOW TO GET YOUR PRIVATE KEY:");
    println!("===============================");
    
    println!("\n📱 FROM PHANTOM WALLET:");
    println!("   1. Settings → Security & Privacy → Export Private Key");
    println!("   2. Copy the JSON array format: [1,2,3,...,64]");
    
    println!("\n📱 FROM SOLFLARE WALLET:");
    println!("   1. Settings → Export Private Key");
    println!("   2. Copy the array of numbers");
    
    println!("\n💻 FROM SOLANA CLI:");
    println!("   1. Run: solana-keygen pubkey ~/.config/solana/id.json");
    println!("   2. The file contains the private key in JSON array format");
    
    println!("\n🔒 SECURITY WARNING:");
    println!("   • NEVER share your private key");
    println!("   • Store it securely in your .env file");
    println!("   • Use a test wallet with small amounts first");
    
    println!("\n📝 NEXT STEPS:");
    println!("   1. Get your private key in one of the formats above");
    println!("   2. Add it to your .env file as WALLET_PRIVATE_KEY=your_key_here");
    println!("   3. Test with this tool again to verify it works");
}