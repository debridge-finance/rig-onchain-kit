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
        user_address: &str,
    ) -> String {
        let params = [
            ("chainId", chain_id),
            ("tokenIn", token_in),
            ("tokenInAmount", token_in_amount),
            ("tokenOut", token_out),
            ("tokenOutRecipient", user_address),
            ("slippage", "auto"),
            ("affiliateFeePercent", "0"),
        ];

        self.build_query_string(&params)
    }

    // Helper method to build query parameters for cross-chain transactions
    fn build_cross_chain_params(
        &self,
        src_chain_id: &str,
        src_chain_token_in: &str,
        src_chain_token_in_amount: &str,
        dst_chain_id: &str,
        dst_chain_token_out: &str,
    ) -> String {
        let params = [
            ("srcChainId", src_chain_id),
            ("srcChainTokenIn", src_chain_token_in),
            ("srcChainTokenInAmount", src_chain_token_in_amount),
            ("dstChainId", dst_chain_id),
            ("dstChainTokenOut", dst_chain_token_out),
            ("dstChainTokenOutAmount", "auto"),
            ("prependOperatingExpenses", "true"),
            ("additionalTakerRewardBps", "0"),
        ];

        self.build_query_string(&params)
    }

    // Helper method to build a query string from parameters
    fn build_query_string(&self, params: &[(&str, &str)]) -> String {
        params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>()
            .join("&")
    }
}
