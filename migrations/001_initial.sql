-- Initial database schema for Raito Proving Service
-- SQLite database with tables for blocks, transactions, and proof files

-- Blocks table - stores Bitcoin block information
CREATE TABLE blocks (
    height INTEGER PRIMARY KEY,
    hash TEXT NOT NULL UNIQUE,
    prev_hash TEXT NOT NULL,
    merkle_root TEXT NOT NULL,
    bits INTEGER NOT NULL,
    nonce INTEGER NOT NULL,
    tx_count INTEGER NOT NULL,
    total_fees REAL NOT NULL,
    timestamp INTEGER NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Index for hash lookups
CREATE INDEX idx_blocks_hash ON blocks(hash);
CREATE INDEX idx_blocks_timestamp ON blocks(timestamp);

-- Transactions table - stores transaction IDs and their block associations
CREATE TABLE transactions (
    txid TEXT PRIMARY KEY,
    block_height INTEGER NOT NULL,
    position_in_block INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (block_height) REFERENCES blocks(height) ON DELETE CASCADE
);

-- Index for block height lookups
CREATE INDEX idx_transactions_block_height ON transactions(block_height);

-- Proof files table - tracks which STARK proof files are available
CREATE TABLE proof_files (
    block_height INTEGER PRIMARY KEY,
    file_path TEXT NOT NULL,
    file_size INTEGER,
    proof_version TEXT NOT NULL DEFAULT 'v1.0',
    generated_at INTEGER NOT NULL,
    execution_time_ms INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (block_height) REFERENCES blocks(height) ON DELETE CASCADE
);

-- Headers table for quick header hash lookups (duplicate of blocks.hash but optimized for lookups)
CREATE TABLE block_headers (
    hash TEXT PRIMARY KEY,
    block_height INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (block_height) REFERENCES blocks(height) ON DELETE CASCADE
);

-- Trigger to automatically update timestamps
CREATE TRIGGER update_blocks_timestamp 
    AFTER UPDATE ON blocks
    BEGIN
        UPDATE blocks SET updated_at = datetime('now') WHERE height = NEW.height;
    END;

-- Trigger to automatically insert header record when block is inserted
CREATE TRIGGER insert_block_header
    AFTER INSERT ON blocks
    BEGIN
        INSERT INTO block_headers (hash, block_height) VALUES (NEW.hash, NEW.height);
    END; 