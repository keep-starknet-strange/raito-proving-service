use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BlockSummary {
    pub height: u32,
    pub hash: String,
    pub tx_count: u32,
    pub total_fees: f64,
    pub timestamp: i64,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BlockDetail {
    #[serde(flatten)]
    pub summary: BlockSummary,
    pub prev_hash: String,
    pub merkle_root: String,
    pub bits: u32,
    pub nonce: u32,
    pub txids: Vec<String>,
    pub proof_url: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TransactionStatus {
    pub included: bool,
    pub block_height: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HeaderStatus {
    pub in_chain: bool,
    pub block_height: Option<u32>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct BlocksQuery {
    #[validate(range(min = 1, max = 50))]
    pub limit: Option<u32>,
    pub cursor: Option<u32>,
}

impl Default for BlocksQuery {
    fn default() -> Self {
        Self {
            limit: Some(20),
            cursor: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BlocksResponse {
    pub blocks: Vec<BlockSummary>,
    pub total: u32,
    pub has_next: bool,
    pub next_cursor: Option<u32>,
}
