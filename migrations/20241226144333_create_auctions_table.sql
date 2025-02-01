CREATE TABLE IF NOT EXISTS auctions (
    auction_id TEXT PRIMARY KEY,
    chain_id INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    seller_address TEXT NOT NULL,
    blockspace_size INTEGER NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER NOT NULL,
    seller_signature TEXT NOT NULL
);
