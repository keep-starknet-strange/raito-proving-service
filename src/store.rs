use crate::{
    error::{AppError, Result},
    model::{BlockDetail, BlockSummary, BlocksResponse, HeaderStatus, TransactionStatus},
};
use once_cell::sync::Lazy;
use std::{collections::HashMap, fs, io::Read, path::Path};

#[derive(Debug)]
pub struct MockStore {
    blocks_by_height: HashMap<u32, BlockDetail>,
    blocks_by_hash: HashMap<String, BlockDetail>,
    tx_index: HashMap<String, u32>,
    header_index: HashMap<String, u32>,
}

static STORE: Lazy<MockStore> =
    Lazy::new(|| MockStore::load_from_files().expect("Failed to load mock store"));

impl MockStore {
    pub fn global() -> &'static Self {
        &STORE
    }

    fn load_from_files() -> Result<Self> {
        let blocks_data = fs::read_to_string("data/mock_blocks.json")
            .map_err(|e| AppError::Store(anyhow::anyhow!("Failed to read blocks file: {}", e)))?;

        let raw_blocks: Vec<serde_json::Value> = serde_json::from_str(&blocks_data)?;

        let mut blocks_by_height = HashMap::new();
        let mut blocks_by_hash = HashMap::new();
        let mut tx_index = HashMap::new();
        let mut header_index = HashMap::new();

        for block_data in raw_blocks {
            let summary = BlockSummary {
                height: block_data["height"].as_u64().unwrap() as u32,
                hash: block_data["hash"].as_str().unwrap().to_string(),
                tx_count: block_data["tx_count"].as_u64().unwrap() as u32,
                total_fees: block_data["total_fees"].as_f64().unwrap(),
                timestamp: block_data["timestamp"].as_i64().unwrap(),
                verified: block_data["verified"].as_bool().unwrap(),
            };

            let txids: Vec<String> = block_data["txids"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect();

            let block_detail = BlockDetail {
                prev_hash: block_data["prev_hash"].as_str().unwrap().to_string(),
                merkle_root: block_data["merkle_root"].as_str().unwrap().to_string(),
                bits: block_data["bits"].as_u64().unwrap() as u32,
                nonce: block_data["nonce"].as_u64().unwrap() as u32,
                proof_url: format!("/v1/blocks/{}/proof", summary.height),
                txids: txids.clone(),
                summary: summary.clone(),
            };

            for txid in &txids {
                tx_index.insert(txid.clone(), summary.height);
            }

            header_index.insert(summary.hash.clone(), summary.height);

            blocks_by_height.insert(summary.height, block_detail.clone());
            blocks_by_hash.insert(summary.hash.clone(), block_detail);
        }

        Ok(Self {
            blocks_by_height,
            blocks_by_hash,
            tx_index,
            header_index,
        })
    }

    pub fn get_blocks(&self, limit: u32, cursor: Option<u32>) -> BlocksResponse {
        let mut blocks: Vec<_> = self.blocks_by_height.values().collect();
        blocks.sort_by(|a, b| b.summary.height.cmp(&a.summary.height));

        let start_idx = if let Some(cursor) = cursor {
            blocks
                .iter()
                .position(|b| b.summary.height < cursor)
                .unwrap_or(blocks.len())
        } else {
            0
        };

        let end_idx = std::cmp::min(start_idx + limit as usize, blocks.len());
        let selected_blocks: Vec<BlockSummary> = blocks[start_idx..end_idx]
            .iter()
            .map(|b| b.summary.clone())
            .collect();

        let has_next = end_idx < blocks.len();
        let next_cursor = if has_next {
            selected_blocks.last().map(|b| b.height)
        } else {
            None
        };

        BlocksResponse {
            blocks: selected_blocks,
            total: self.blocks_by_height.len() as u32,
            has_next,
            next_cursor,
        }
    }

    pub fn get_block_by_height(&self, height: u32) -> Result<&BlockDetail> {
        self.blocks_by_height
            .get(&height)
            .ok_or_else(|| AppError::BlockNotFound(height.to_string()))
    }

    pub fn get_block_by_hash(&self, hash: &str) -> Result<&BlockDetail> {
        self.blocks_by_hash
            .get(hash)
            .ok_or_else(|| AppError::BlockNotFound(hash.to_string()))
    }

    pub fn get_proof_file(&self, height: u32) -> Result<Vec<u8>> {
        if !self.blocks_by_height.contains_key(&height) {
            return Err(AppError::BlockNotFound(height.to_string()));
        }

        let proof_path = format!("data/proofs/{}.json", height);
        if !Path::new(&proof_path).exists() {
            return Err(AppError::ProofNotFound(height.to_string()));
        }

        let mut file = fs::File::open(&proof_path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        Ok(contents)
    }

    pub fn get_transaction_status(&self, txid: &str) -> Result<TransactionStatus> {
        if let Some(&block_height) = self.tx_index.get(txid) {
            Ok(TransactionStatus {
                included: true,
                block_height: Some(block_height),
            })
        } else {
            Ok(TransactionStatus {
                included: false,
                block_height: None,
            })
        }
    }

    pub fn get_header_status(&self, hash: &str) -> Result<HeaderStatus> {
        if let Some(&block_height) = self.header_index.get(hash) {
            Ok(HeaderStatus {
                in_chain: true,
                block_height: Some(block_height),
            })
        } else {
            Ok(HeaderStatus {
                in_chain: false,
                block_height: None,
            })
        }
    }

    pub fn block_exists_by_identifier(&self, identifier: &str) -> bool {
        if let Ok(height) = identifier.parse::<u32>() {
            self.blocks_by_height.contains_key(&height)
        } else {
            self.blocks_by_hash.contains_key(identifier)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_loading() {
        let store = MockStore::global();
        assert!(!store.blocks_by_height.is_empty());
        assert!(!store.blocks_by_hash.is_empty());
    }

    #[test]
    fn test_get_blocks_pagination() {
        let store = MockStore::global();
        let response = store.get_blocks(2, None);

        assert_eq!(response.blocks.len(), 2);
        assert!(response.total > 0);
    }

    #[test]
    fn test_get_block_by_height() {
        let store = MockStore::global();
        if let Some((&height, _)) = store.blocks_by_height.iter().next() {
            let result = store.get_block_by_height(height);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_transaction_status() {
        let store = MockStore::global();
        if let Some((txid, _)) = store.tx_index.iter().next() {
            let status = store.get_transaction_status(txid).unwrap();
            assert!(status.included);
            assert!(status.block_height.is_some());
        }
    }
}
