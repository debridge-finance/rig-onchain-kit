use anyhow::Result;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;

use crate::{
    common::{claude_agent_builder, PREAMBLE_COMMON},
    data::{FetchCandlesticks, FetchTopTokens},
    debridge::tools::{GetDebridgeQuoteTool, GetDebridgeSupportedChainsTool, SearchDebridgeTokensTool},
    dexscreener::tools::SearchOnDexScreener,
};

/// Create a deBridge agent for cross-chain operations
pub async fn create_debridge_agent(
    preamble: Option<String>,
) -> Result<Agent<AnthropicCompletionModel>> {
    let preamble = preamble.unwrap_or(format!(
        "{} {}",
        "You are a cross-chain trading agent using deBridge protocol", PREAMBLE_COMMON,
    ));
    
    Ok(claude_agent_builder()
        .preamble(&preamble)
        .tool(SearchOnDexScreener)
        .tool(GetDebridgeQuoteTool)
        .tool(GetDebridgeSupportedChainsTool)
        .tool(SearchDebridgeTokensTool)
        .tool(FetchCandlesticks)
        .tool(FetchTopTokens)
        .build())
}
