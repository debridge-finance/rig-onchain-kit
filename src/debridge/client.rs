use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;

use crate::debridge::DEBRIDGE_API_URL;

/// Client for interacting with the deBridge API
pub struct DeBridgeClient {
    client: Client,
    base_url: String,
}

impl DeBridgeClient {
    /// Create a new deBridge client
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: DEBRIDGE_API_URL.to_string(),
        }
    }

    /// Get a list of supported chains and their information
    pub async fn get_supported_chains(&self) -> Result<Value> {
        let url = format!("{}/supported-chains-info", self.base_url);
        
        // Make the request
        let response = self.client.get(&url).send().await?;
        
        // Handle the response
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            return Err(anyhow!("HTTP error! status: {}, body: {}", status, text));
        }
        
        // Parse the response
        let data: Value = response.json().await?;
        
        // Check for errors in the response
        if let Some(error) = data.get("error").and_then(|e| e.as_str()) {
            return Err(anyhow!("API error: {}", error));
        }
        
        Ok(data)
    }

    /// Get a list of tokens for a specific chain
    pub async fn get_token_list(&self, chain_id: &str) -> Result<Value> {
        let url = format!("{}/token-list?chainId={}", self.base_url, chain_id);
        
        // Make the request
        let response = self.client.get(&url).send().await?;
        
        // Handle the response
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            return Err(anyhow!("HTTP error! status: {}, body: {}", status, text));
        }
        
        // Parse the response
        let data: Value = response.json().await?;
        
        // Check for errors in the response
        if let Some(error) = data.get("error").and_then(|e| e.as_str()) {
            return Err(anyhow!("API error: {}", error));
        }
        
        Ok(data)
    }

    /// Search for tokens on a specific chain by name or symbol
    pub async fn search_tokens(&self, chain_id: &str, search_term: &str) -> Result<Value> {
        // First, get the complete token list for the chain
        let token_list = self.get_token_list(chain_id).await?;
        
        // Extract the tokens object from the response
        let tokens = token_list.get("tokens")
            .ok_or_else(|| anyhow!("Invalid token list response format"))?;
        
        // Convert the search term to lowercase for case-insensitive comparison
        let search_term = search_term.to_lowercase();
        
        // Filter the tokens by the search term (matching name or symbol)
        let mut filtered_tokens = serde_json::Map::new();
        
        if let Some(tokens_obj) = tokens.as_object() {
            for (address, token_data) in tokens_obj {
                let token_symbol = token_data.get("symbol")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                
                let token_name = token_data.get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                
                // Check if the search term matches either the symbol or name
                if token_symbol.contains(&search_term) || token_name.contains(&search_term) {
                    filtered_tokens.insert(address.clone(), token_data.clone());
                }
            }
        }
        
        // Create a new response with only the filtered tokens
        let mut filtered_response = serde_json::Map::new();
        filtered_response.insert("tokens".to_string(), Value::Object(filtered_tokens));
        
        Ok(Value::Object(filtered_response))
    }

    /// Get a quote for bridging tokens between chains
    pub async fn get_bridge_quote(
        &self,
        src_chain_id: &str,
        src_chain_token_in: &str,
        src_chain_token_in_amount: &str,
        dst_chain_id: &str,
        dst_chain_token_out: &str,
        user_address: &str,
        slippage: Option<&str>,
        dst_chain_token_out_amount: Option<&str>,
        prepend_operating_expenses: Option<&str>,
    ) -> Result<Value> {
        // Check if source and destination chains are the same
        let is_same_chain = src_chain_id == dst_chain_id;

        // Build the URL with query parameters
        let url = if is_same_chain {
            format!(
                "{}/chain/transaction?{}",
                self.base_url,
                self.build_same_chain_params(
                    src_chain_id,
                    src_chain_token_in,
                    src_chain_token_in_amount,
                    dst_chain_token_out,
                    user_address,
                    slippage,
                )
            )
        } else {
            format!(
                "{}/dln/order/create-tx?{}",
                self.base_url,
                self.build_cross_chain_params(
                    src_chain_id,
                    src_chain_token_in,
                    src_chain_token_in_amount,
                    dst_chain_id,
                    dst_chain_token_out,
                    user_address,
                    dst_chain_token_out_amount,
                    prepend_operating_expenses,
                )
            )
        };

        // Make the request
        let response = self.client.get(&url).send().await?;

        // Handle the response
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            return Err(anyhow!("HTTP error! status: {}, body: {}", status, text));
        }

        // Parse the response
        let data: Value = response.json().await?;

        // Check for errors in the response
        if let Some(error) = data.get("error").and_then(|e| e.as_str()) {
            return Err(anyhow!("API error: {}", error));
        }

        Ok(data)
    }

    // Helper method to build query parameters for same-chain transactions
    fn build_same_chain_params(
        &self,
        chain_id: &str,
        token_in: &str,
        token_in_amount: &str,
        token_out: &str,
        token_out_recipient: &str,
        slippage: Option<&str>,
    ) -> String {
        let params = vec![
            ("chainId", chain_id.to_string()),
            ("tokenIn", token_in.to_string()),
            ("tokenInAmount", token_in_amount.to_string()),
            ("tokenOut", token_out.to_string()),
            ("tokenOutRecipient", token_out_recipient.to_string()),
            (
                "slippage",
                slippage.unwrap_or("auto").to_string(),
            ),
            ("affiliateFeePercent", "0".to_string()),
        ];

        params
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>()
            .join("&")
    }

    // Helper method to build query parameters for cross-chain transactions
    fn build_cross_chain_params(
        &self,
        src_chain_id: &str,
        src_chain_token_in: &str,
        src_chain_token_in_amount: &str,
        dst_chain_id: &str,
        dst_chain_token_out: &str,
        user_address: &str,
        dst_chain_token_out_amount: Option<&str>,
        prepend_operating_expenses: Option<&str>,
    ) -> String {
        // Determine if destination chain is Solana (7565164)
        let is_dst_solana = dst_chain_id == "7565164";
        
        // For Solana, we need a valid Solana address. For simplicity in this example,
        // we'll use a common example Solana address when the destination is Solana
        let dst_recipient = if is_dst_solana {
            // Example Solana address - in a real implementation, you would derive this properly
            "DRiMMbLc8Tmxozpamk3a6JwRvyYH6vc72Md7xd4jEyxw"
        } else {
            user_address
        };
        
        let params = vec![
            ("srcChainId", src_chain_id.to_string()),
            ("srcChainTokenIn", src_chain_token_in.to_string()),
            ("srcChainTokenInAmount", src_chain_token_in_amount.to_string()),
            ("dstChainId", dst_chain_id.to_string()),
            ("dstChainTokenOut", dst_chain_token_out.to_string()),
            ("dstChainTokenOutAmount", dst_chain_token_out_amount.unwrap_or("auto").to_string()),
            // Use appropriate address format based on destination chain
            ("dstChainTokenOutRecipient", dst_recipient.to_string()),
            // Add sender address (same as user_address)
            ("senderAddress", user_address.to_string()),
            // Source chain authorities
            ("srcChainOrderAuthorityAddress", user_address.to_string()),
            ("srcChainRefundAddress", user_address.to_string()),
            // Destination chain authority
            ("dstChainOrderAuthorityAddress", dst_recipient.to_string()),
            // Analytics
            ("referralCode", "21064".to_string()),
            ("deBridgeApp", "GOAT".to_string()),  // Using allowed value
            // Operating expenses
            (
                "prependOperatingExpenses",
                prepend_operating_expenses.unwrap_or("true").to_string(),
            ),
        ];

        params
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>()
            .join("&")
    }
}
