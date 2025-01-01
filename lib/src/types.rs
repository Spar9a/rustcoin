use crate::U256;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct Blockchain {
    pub blocks: Vec<Block>,
}

pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

pub struct BlockHeader {
    /// Timestamp of the block
    pub timestamp: DateTime<Utc>,
    // Nonce used to mine the block
    pub nonce: u64,
    /// Hash of the prev block
    pub prev_block_hash: [u8; 32],
    /// Merle root of the block's transactions
    pub merkle_root: [u8; 32],
    /// target
    pub target: U256,
}

pub struct Transaction{
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>
}

pub struct TransactionInput {
    pub prev_transaction_output_hash: [u8; 32],
    pub signature: [u8; 64],
}

pub struct TransactionOutput {
    pub value: u64,
    pub unique_id: Uuid,
    pub pubkey: [u8; 32],
}


impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain { blocks: vec![] }
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

impl Block {
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Block {
        Block { header, transactions }
    }

    pub fn hash(&self) -> ! {
        unimplemented!()
    }
}

impl BlockHeader {
    pub fn new(
        timestamp: DateTime<Utc>,
        nonce: u64,
        prev_block_hash: [u8; 32],
        merkle_root: [u8; 32],
        target: U256,
    ) -> BlockHeader {
        BlockHeader {
            timestamp,
            nonce,
            prev_block_hash,
            merkle_root,
            target,
        }
    }

    pub fn hash(&self) -> ! {
        unimplemented!()
    }
}

impl Transaction {
    pub fn new(
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
    ) -> Transaction {
        Transaction { inputs, outputs }
    }

    pub fn hash(&self) -> ! {
        unimplemented!()
    }
}