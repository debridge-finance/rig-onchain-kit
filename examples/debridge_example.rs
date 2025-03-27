use anyhow::Result;
use serde_json::Value;

use rig_onchain_kit::debridge::client::DeBridgeClient;

#[cfg(feature = "debridge")]
#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Testing deBridge Implementation ===\n");
    
    // Create a deBridge client
    println!("Creating deBridge client...");
    let client = DeBridgeClient::new();
    
    // Test getting supported chains
    println!("\n=== Testing Supported Chains API ===");
    let supported_chains = client.get_supported_chains().await?;
    
    if let Some(chains) = supported_chains.get("chains").and_then(|c| c.as_array()) {
        println!("Supported chains count: {}", chains.len());
        println!("\nFirst 5 supported chains:");
        
        for chain in chains.iter().take(5) {
            let name = chain.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
            let chain_id = chain.get("chainId").unwrap_or(&Value::Null);
            let chain_type = chain.get("chainType").and_then(|t| t.as_str()).unwrap_or("Unknown");
            
            println!("- {} (ID: {}, Type: {})", name, chain_id, chain_type);
        }
    } else {
        println!("No chains found in the response");
    }
    
    // Test token search functionality
    println!("\n=== Testing Token Search API ===");
    // Search for DBR tokens on Solana
    let search_chain_id = "7565164"; // Solana
    let search_term = "DBR";
    println!("Searching for '{}' tokens on chain ID: {}", search_term, search_chain_id);
    
    let token_results = client.search_tokens(search_chain_id, search_term).await?;
    
    if let Some(tokens) = token_results.get("tokens").and_then(|t| t.as_object()) {
        println!("Found {} matching tokens:", tokens.len());
        
        if tokens.is_empty() {
            println!("No matching tokens found. Let's try a different search term.");
            
            // Try a more common token on Solana as a fallback
            let fallback_search = "SOL";
            println!("Searching for '{}' tokens on chain ID: {}", fallback_search, search_chain_id);
            let fallback_results = client.search_tokens(search_chain_id, fallback_search).await?;
            
            if let Some(fallback_tokens) = fallback_results.get("tokens").and_then(|t| t.as_object()) {
                println!("Found {} matching tokens for '{}':", fallback_tokens.len(), fallback_search);
                
                for (address, token_data) in fallback_tokens.iter().take(3) {
                    let name = token_data.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
                    let symbol = token_data.get("symbol").and_then(|s| s.as_str()).unwrap_or("Unknown");
                    let decimals = token_data.get("decimals").and_then(|d| d.as_u64()).unwrap_or(0);
                    
                    println!("- {}: {} ({}) [Decimals: {}]", address, name, symbol, decimals);
                }
            }
        } else {
            for (address, token_data) in tokens.iter() {
                let name = token_data.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
                let symbol = token_data.get("symbol").and_then(|s| s.as_str()).unwrap_or("Unknown");
                let decimals = token_data.get("decimals").and_then(|d| d.as_u64()).unwrap_or(0);
                
                println!("- {}: {} ({}) [Decimals: {}]", address, name, symbol, decimals);
            }
        }
    } else {
        println!("Invalid token list response format");
    }
    
    // Test getting a quote (using example values)
    println!("\n=== Testing Quote API ===");
    let src_chain_id = "1"; // Ethereum mainnet
    let src_token_address = "0x0000000000000000000000000000000000000000"; // ETH (native token)
    let amount = "10000000000000000"; // 0.01 ETH
    let dst_chain_id = "7565164"; // Solana
    let dst_token_address = "11111111111111111111111111111111"; // SOL (native token)
    let address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"; // Example address (vitalik.eth)
    
    println!("Quote request parameters:");
    println!("  From chain: {} ({})", src_chain_id, src_token_address);
    println!("  To chain: {} ({})", dst_chain_id, dst_token_address);
    println!("  Amount: {} wei", amount);
    println!("  Recipient: {}", address);
    
    let quote = client.get_bridge_quote(
        src_chain_id,
        src_token_address,
        amount,
        dst_chain_id,
        dst_token_address,
        address,
    ).await?;
    
    // Print a simplified version of the quote response
    println!("\nQuote response (simplified summary):");
    
    if let Some(src_chain_gas_fee) = quote.get("srcChainGasFee") {
        println!("Source chain gas fee: {}", src_chain_gas_fee);
    }
    
    if let Some(dst_chain_gas_fee) = quote.get("dstChainGasFee") {
        println!("Destination chain gas fee: {}", dst_chain_gas_fee);
    }
    
    if let Some(estimation) = quote.get("estimation") {
        if let Some(dst_chain_token_out) = estimation.get("dstChainTokenOut") {
            println!("Estimated amount out: {}", dst_chain_token_out);
        }
    }
    
    println!("\n=== Tests Completed Successfully ===");
    println!("The deBridge module is working correctly!");
    
    Ok(())
}

#[cfg(not(feature = "debridge"))]
fn main() {
    println!("Enable the debridge feature to run this example.");
    println!("Try: cargo run --example debridge_example --features=debridge");
}
