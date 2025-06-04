use crate::{
    error::{AppError, Result},
    model::{BlockDetail, BlockSummary, BlocksResponse, HeaderStatus, TransactionStatus},
};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::{path::Path, str::FromStr};
use tracing::info;

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub run_migrations: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite:raito_data/raito.db".to_string(),
            max_connections: 10,
            run_migrations: true,
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:raito_data/raito.db".to_string()),
            max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            run_migrations: std::env::var("DATABASE_RUN_MIGRATIONS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }

    pub fn test_config() -> Self {
        Self {
            database_url: "sqlite::memory:".to_string(),
            max_connections: 5,
            run_migrations: true,
        }
    }
}

impl Database {
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        // Ensure data directory exists for file-based databases
        if config.database_url.starts_with("sqlite:") && !config.database_url.contains(":memory:") {
            if let Some(db_path) = config.database_url.strip_prefix("sqlite:") {
                if let Some(parent) = Path::new(db_path).parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        AppError::Store(anyhow::anyhow!(
                            "Failed to create database directory: {}",
                            e
                        ))
                    })?;
                }
            }
        }

        let options = SqliteConnectOptions::from_str(&config.database_url)
            .map_err(|e| AppError::Store(anyhow::anyhow!("Invalid database URL: {}", e)))?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .pragma("cache_size", "1000")
            .pragma("temp_store", "memory");

        let pool = SqlitePool::connect_with(options).await.map_err(|e| {
            AppError::Store(anyhow::anyhow!("Failed to connect to database: {}", e))
        })?;

        let db = Self { pool };

        if config.run_migrations {
            db.run_migrations().await?;
        }

        info!("Database initialized successfully");
        Ok(db)
    }

    pub async fn run_migrations(&self) -> Result<()> {
        info!("Running database migrations...");

        let migration_sql = include_str!("../migrations/001_initial.sql");

        sqlx::query(migration_sql)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Store(anyhow::anyhow!("Migration failed: {}", e)))?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    pub async fn seed_data(&self) -> Result<()> {
        info!("Seeding database with mock data...");

        let mock_data = include_str!("../data/mock_blocks.json");
        let blocks: Vec<serde_json::Value> = serde_json::from_str(mock_data)
            .map_err(|e| AppError::Store(anyhow::anyhow!("Failed to parse mock data: {}", e)))?;

        for block_data in blocks {
            self.insert_block(&block_data).await?;
        }

        info!("Database seeding completed successfully");
        Ok(())
    }

    async fn insert_block(&self, block_data: &serde_json::Value) -> Result<()> {
        let height = block_data["height"].as_u64().unwrap() as i64;
        let hash = block_data["hash"].as_str().unwrap();
        let prev_hash = block_data["prev_hash"].as_str().unwrap();
        let merkle_root = block_data["merkle_root"].as_str().unwrap();
        let bits = block_data["bits"].as_u64().unwrap() as i64;
        let nonce = block_data["nonce"].as_u64().unwrap() as i64;
        let tx_count = block_data["tx_count"].as_u64().unwrap() as i64;
        let total_fees = block_data["total_fees"].as_f64().unwrap();
        let timestamp = block_data["timestamp"].as_i64().unwrap();
        let verified = block_data["verified"].as_bool().unwrap();

        // Insert block
        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO blocks 
            (height, hash, prev_hash, merkle_root, bits, nonce, tx_count, total_fees, timestamp, verified)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            height, hash, prev_hash, merkle_root, bits, nonce, tx_count, total_fees, timestamp, verified
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Store(anyhow::anyhow!("Failed to insert block: {}", e)))?;

        // Insert transactions
        if let Some(txids) = block_data["txids"].as_array() {
            for (position, txid_value) in txids.iter().enumerate() {
                if let Some(txid) = txid_value.as_str() {
                    let position_i64 = position as i64;
                    sqlx::query!(
                        "INSERT OR REPLACE INTO transactions (txid, block_height, position_in_block) VALUES (?, ?, ?)",
                        txid, height, position_i64
                    )
                    .execute(&self.pool)
                    .await
                    .map_err(|e| AppError::Store(anyhow::anyhow!("Failed to insert transaction: {}", e)))?;
                }
            }
        }

        // Insert proof file record if it exists
        let proof_path = format!("data/proofs/{}.json", height);
        if Path::new(&proof_path).exists() {
            sqlx::query!(
                r#"
                INSERT OR REPLACE INTO proof_files 
                (block_height, file_path, proof_version, generated_at, execution_time_ms)
                VALUES (?, ?, 'v1.0', ?, 45000)
                "#,
                height,
                proof_path,
                timestamp
            )
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Store(anyhow::anyhow!("Failed to insert proof file: {}", e)))?;
        }

        Ok(())
    }

    pub async fn get_blocks(&self, limit: u32, cursor: Option<u32>) -> Result<BlocksResponse> {
        let limit = limit.min(50) as i64;

        let blocks = if let Some(cursor_height) = cursor {
            let cursor_i64 = cursor_height as i64;
            sqlx::query_as!(
                BlockSummary,
                r#"
                SELECT height as "height: u32", hash, tx_count as "tx_count: u32", 
                       total_fees, timestamp, verified as "verified: bool"
                FROM blocks 
                WHERE height < ?
                ORDER BY height DESC 
                LIMIT ?
                "#,
                cursor_i64,
                limit
            )
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as!(
                BlockSummary,
                r#"
                SELECT height as "height: u32", hash, tx_count as "tx_count: u32", 
                       total_fees, timestamp, verified as "verified: bool"
                FROM blocks 
                ORDER BY height DESC 
                LIMIT ?
                "#,
                limit
            )
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| AppError::Store(anyhow::anyhow!("Failed to fetch blocks: {}", e)))?;

        let total = sqlx::query_scalar!("SELECT COUNT(*) FROM blocks")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::Store(anyhow::anyhow!("Failed to count blocks: {}", e)))?;

        let has_next = blocks.len() as i64 == limit;
        let next_cursor = if has_next {
            blocks.last().map(|b| b.height)
        } else {
            None
        };

        Ok(BlocksResponse {
            blocks,
            total: total as u32,
            has_next,
            next_cursor,
        })
    }

    pub async fn get_block_by_height(&self, height: u32) -> Result<BlockDetail> {
        let height_i64 = height as i64;
        let block_row = sqlx::query!(
            r#"
            SELECT height, hash, prev_hash, merkle_root, bits, nonce, 
                   tx_count, total_fees, timestamp, verified
            FROM blocks 
            WHERE height = ?
            "#,
            height_i64
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Store(anyhow::anyhow!("Failed to fetch block: {}", e)))?
        .ok_or_else(|| AppError::BlockNotFound(height.to_string()))?;

        let txids: Vec<String> = sqlx::query_scalar!(
            "SELECT txid FROM transactions WHERE block_height = ? ORDER BY position_in_block",
            height_i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Store(anyhow::anyhow!("Failed to fetch transactions: {}", e)))?
        .into_iter()
        .filter_map(|txid| txid)
        .collect();

        Ok(BlockDetail {
            summary: BlockSummary {
                height: block_row.height as u32,
                hash: block_row.hash,
                tx_count: block_row.tx_count as u32,
                total_fees: block_row.total_fees,
                timestamp: block_row.timestamp,
                verified: block_row.verified,
            },
            prev_hash: block_row.prev_hash,
            merkle_root: block_row.merkle_root,
            bits: block_row.bits as u32,
            nonce: block_row.nonce as u32,
            proof_url: format!("/v1/blocks/{}/proof", height),
            txids,
        })
    }

    pub async fn get_block_by_hash(&self, hash: &str) -> Result<BlockDetail> {
        let height = sqlx::query_scalar!(
            "SELECT block_height FROM block_headers WHERE hash = ?",
            hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Store(anyhow::anyhow!("Failed to fetch block by hash: {}", e)))?
        .ok_or_else(|| AppError::BlockNotFound(hash.to_string()))?;

        self.get_block_by_height(height as u32).await
    }

    pub async fn get_transaction_status(&self, txid: &str) -> Result<TransactionStatus> {
        let result = sqlx::query!("SELECT block_height FROM transactions WHERE txid = ?", txid)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Store(anyhow::anyhow!("Failed to fetch transaction: {}", e)))?;

        Ok(match result {
            Some(row) => TransactionStatus {
                included: true,
                block_height: Some(row.block_height as u32),
            },
            None => TransactionStatus {
                included: false,
                block_height: None,
            },
        })
    }

    pub async fn get_header_status(&self, hash: &str) -> Result<HeaderStatus> {
        let result = sqlx::query!(
            "SELECT block_height FROM block_headers WHERE hash = ?",
            hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Store(anyhow::anyhow!("Failed to fetch header: {}", e)))?;

        Ok(match result {
            Some(row) => HeaderStatus {
                in_chain: true,
                block_height: Some(row.block_height as u32),
            },
            None => HeaderStatus {
                in_chain: false,
                block_height: None,
            },
        })
    }

    pub async fn proof_file_exists(&self, height: u32) -> Result<bool> {
        let height_i64 = height as i64;
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM proof_files WHERE block_height = ?)",
            height_i64
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Store(anyhow::anyhow!("Failed to check proof file: {}", e)))?;

        Ok(exists == Some(1))
    }

    pub async fn block_exists_by_identifier(&self, identifier: &str) -> Result<bool> {
        if let Ok(height) = identifier.parse::<u32>() {
            let height_i64 = height as i64;
            let exists = sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM blocks WHERE height = ?)",
                height_i64
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                AppError::Store(anyhow::anyhow!("Failed to check block existence: {}", e))
            })?;
            Ok(exists == Some(1))
        } else {
            let exists = sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM block_headers WHERE hash = ?)",
                identifier
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                AppError::Store(anyhow::anyhow!("Failed to check block existence: {}", e))
            })?;
            Ok(exists == Some(1))
        }
    }

    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Store(anyhow::anyhow!("Database health check failed: {}", e)))?;
        Ok(())
    }
}
