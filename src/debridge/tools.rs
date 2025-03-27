use anyhow::Result;
use rig_tool_macro::tool;
use serde_json::{Value, json};

use crate::debridge::client::DeBridgeClient;

/// Native token address for EVM chains (ETH/BNB/etc)
const EVM_NATIVE_TOKEN: &str = "0x0000000000000000000000000000000000000000";

/// Native token address for Solana (SOL) in base58 format
const SOLANA_NATIVE_TOKEN: &str = "11111111111111111111111111111111";

#[tool(description = "
Retrieve the list of blockchains supported by deBridge protocol.

Returns information about each supported chain including chain ID, name, type, 
and other relevant details that can be used for cross-chain operations.
")]
pub async fn get_debridge_supported_chains() -> Result<Value> {
    let client = DeBridgeClient::new();
    let supported_chains = client.get_supported_chains().await?;
    Ok(supported_chains)
}

#[tool(description = "
Search for tokens on a specific blockchain by name or symbol using deBridge protocol.

Parameters:
- chain_id: The ID of the chain to search on (e.g., '1' for Ethereum, '7565164' for Solana)
- search_term: The token name or symbol to search for (e.g., 'DBR', 'Ethereum')

Returns a list of matching tokens with their details such as address, name, symbol, and decimals.

Example:
```
search_debridge_tokens(chain_id: '7565164', search_term: 'DBR')
```
")]
pub async fn search_debridge_tokens(chain_id: String, search_term: String) -> Result<Value> {
    let client = DeBridgeClient::new();
    let tokens = client.search_tokens(&chain_id, &search_term).await?;
    Ok(tokens)
}

#[tool(description = "
Get a quote for a cross-chain transfer using deBridge protocol.

Required Parameters:
- from_token_address: The address of the token to send. Use the native token address for ETH/BNB/etc.
- to_token_address: The address of the token to receive. For Solana tokens, use the base58 address.
- amount: The amount of tokens to send, in the smallest unit (e.g., wei for ETH).
- from_chain: The ID of the source chain (e.g., '1' for Ethereum, '56' for BSC).
- to_chain: The ID of the destination chain (e.g., '7565164' for Solana).
- user_address: The address of the user who will receive the tokens. This is also used as the sender address, source and destination chain authorities.

Optional Parameters:
- slippage: Maximum allowed slippage in percentage. Default is 'auto'.
- dst_token_amount: The desired amount of tokens to receive. Default is 'auto'.
- prepend_operating_expenses: Whether to include operating expenses in the quote. Default is 'true'.

Examples:
```
// Simple quote with required parameters only
get_debridge_quote(
  from_token_address: '0x0000000000000000000000000000000000000000',
  to_token_address: 'DBRiDgJAMsM95moTzJs7M9LnkGErpbv9v6CUR1DXnUu5',
  amount: '1000000000000000000',
  from_chain: '1',
  to_chain: '7565164',
  user_address: '0xYourAddressHere'
)

// Quote with optional parameters
get_debridge_quote(
  from_token_address: '0x0000000000000000000000000000000000000000',
  to_token_address: 'DBRiDgJAMsM95moTzJs7M9LnkGErpbv9v6CUR1DXnUu5',
  amount: '1000000000000000000',
  from_chain: '1',
  to_chain: '7565164',
  user_address: '0xYourAddressHere',
  slippage: '0.5',
  prepend_operating_expenses: 'true'
)
```
")]
pub async fn get_debridge_quote(
    from_token_address: String,
    to_token_address: String,
    amount: String,
    from_chain: String,
    to_chain: String,
    user_address: String,
    slippage: Option<String>,
    dst_token_amount: Option<String>,
    prepend_operating_expenses: Option<String>,
) -> Result<Value> {
    // Create a new deBridge client
    let client = DeBridgeClient::new();
    
    // Get a quote for the cross-chain transfer
    let quote = client.get_bridge_quote(
        &from_chain,
        &from_token_address,
        &amount,
        &to_chain,
        &to_token_address,
        &user_address,
        slippage.as_deref(),
        dst_token_amount.as_deref(),
        prepend_operating_expenses.as_deref(),
    ).await?;
    
    Ok(quote)
}

#[tool(description = "
Execute a deBridge transaction for cross-chain transfers. This function takes the transaction data 
obtained from a quote and executes it on the source chain to initiate the cross-chain transfer.

Required Parameters:
- tx_data: The transaction data from the quote response. This must include 'to', 'data', 'value' (optional), and 'chainId' fields.

The function validates the transaction data format and submits it to the blockchain using the current wallet context.
It returns the transaction hash that can be used to track the transaction status.

Example:
```
execute_debridge_transaction(
  tx_data: quote.tx
)
```

Always ask for confirmation before executing a transaction, as it can move real assets on the blockchain.
")]
pub async fn execute_debridge_transaction(
    tx_data: Value,
) -> Result<Value> {
    use crate::debridge::transaction;
    
    // Execute the transaction using the current signer context
    let tx_hash = transaction::execute_debridge_transaction(&tx_data).await?;
    
    Ok(json!({ "hash": tx_hash }))
}

/// Helper function to determine if a token address is a native token
pub fn is_native_token(token_address: &str) -> bool {
    token_address == EVM_NATIVE_TOKEN || token_address == SOLANA_NATIVE_TOKEN
}
