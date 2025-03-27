use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use crate::common::wrap_unsafe;
use crate::signer::SignerContext;

/// Execute a deBridge transaction using the current signer context
pub async fn execute_debridge_transaction(tx_data: &Value) -> Result<String> {
    let signer = SignerContext::current().await;
    
    // Validate transaction data
    let to = tx_data.get("to")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'to' field in transaction data"))?;
    
    let data = tx_data.get("data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'data' field in transaction data"))?;
    
    if !data.starts_with("0x") {
        return Err(anyhow!("Invalid transaction data: 'data' field must start with '0x'"));
    }
    
    let value = tx_data.get("value")
        .and_then(|v| v.as_str())
        .unwrap_or("0x0");

    // Get chain ID - privy implementation requires this as a number
    let chain_id_value = tx_data.get("chainId")
        .or_else(|| tx_data.get("chain_id"))
        .ok_or_else(|| anyhow!("Missing 'chainId' or 'chain_id' field in transaction data"))?;

    // Convert chain ID to u64 if it's a string
    let chain_id = match chain_id_value {
        Value::String(s) => {
            if s.starts_with("0x") {
                // Parse hex string
                u64::from_str_radix(&s[2..], 16)
                    .map_err(|_| anyhow!("Invalid chainId hex format: {}", s))?
            } else {
                // Parse decimal string
                s.parse::<u64>()
                    .map_err(|_| anyhow!("Invalid chainId format: {}", s))?
            }
        },
        Value::Number(n) => n.as_u64()
            .ok_or_else(|| anyhow!("Chain ID is not a valid u64 number"))?,
        _ => return Err(anyhow!("Chain ID must be a string or number")),
    };
    
    tracing::info!(
        to = to,
        data_selector = &data[..std::cmp::min(10, data.len())],
        value = value,
        chain_id = chain_id,
        "Executing deBridge transaction"
    );
    
    // For Solana, we need a different approach
    if is_solana_chain(chain_id) {
        return Err(anyhow!("Solana transactions are not supported yet through this method"));
    }
    
    // Construct the transaction JSON that matches what the signer expects
    wrap_unsafe(move || async move {
        signer.sign_and_send_json_evm_transaction(serde_json::json!({
            "to": to,
            "data": data,
            "value": value,
            "chain_id": chain_id,  // Must be a number
        })).await
    }).await
}

/// Check if a chain ID represents a Solana chain
fn is_solana_chain(chain_id: u64) -> bool {
    chain_id == 7565164  // Solana mainnet CAIP-2 chain ID
}

/// Get the chain type from a chain ID string
pub fn get_chain_type(chain_id: &str) -> Result<ChainType> {
    match chain_id {
        // EVM chains
        "1" | "10" | "56" | "137" | "250" | "42161" | "43114" => Ok(ChainType::Evm),
        
        // Solana
        "7565164" => Ok(ChainType::Solana),
        
        // Unknown chain ID
        _ => Err(anyhow!("Unsupported chain ID: {}", chain_id)),
    }
}

/// Enum representing different blockchain types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainType {
    Evm,
    Solana,
}
