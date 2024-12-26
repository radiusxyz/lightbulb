-- migrations/YYYYMMDDHHMMSS_create_auctions_table.sql

CREATE TABLE IF NOT EXISTS auctions (
    id TEXT PRIMARY KEY,
    block_height INTEGER NOT NULL,
    seller_addr TEXT NOT NULL,
    blockspace_size INTEGER NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER NOT NULL,
    seller_signature TEXT NOT NULL
);
