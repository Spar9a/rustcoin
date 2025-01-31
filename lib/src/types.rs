use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::crypto::{PublicKey, Signature};
use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::util::MerkleRoot;
use crate::U256;

#[derive(Serialize, Deserialize, Clone)]
pub struct Blockchain {
    pub utxos: HashMap<Hash, TransactionOutput>,
    pub target: U256,
    pub blocks: Vec<Block>,
    #[serde(default, skip_serializing)]
    pub mempool: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockHeader {
    /// Timestamp of the block
    pub timestamp: DateTime<Utc>,
    // Nonce used to mine the block
    pub nonce: u64,
    /// Hash of the prev block
    pub prev_block_hash: Hash,
    /// Merle root of the block's transactions
    pub merkle_root: MerkleRoot,
    /// target
    pub target: U256,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Transaction{
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransactionInput {
    pub prev_transaction_output_hash: Hash,
    pub signature: Signature,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionOutput {
    pub value: u64,
    pub unique_id: Uuid,
    pub pubkey: PublicKey,
}
impl TransactionOutput {
    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}


impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            utxos: HashMap::new(),
            blocks: vec![],
            target: crate::MIN_TARGET,
            mempool: vec![],
        }
    }

    pub fn add_block(
        &mut self,
        block: Block) -> Result<()> {
        if self.blocks.is_empty() {
            if block.header.prev_block_hash != Hash::zero() {
                println!("Block 0 is not the genesis block");
                return Err(BtcError::InvalidBlock)
            }
        } else {
            let last_block = self.blocks.last().unwrap();
            if block.header.prev_block_hash != last_block.hash() {
                println!("Block is not linked to the last block");
                return Err(BtcError::InvalidBlock)
            }
            // check if the block's hash is less than the target
            if !block
                .header
                .hash()
                .matches_target(last_block.header.target){
                println!("Block hash does not match target");
                return Err(BtcError::InvalidBlock)
            }
            // check if the block's merkle root is correct
            let calulated_merkle_root =
                MerkleRoot::calculate(&block.transactions);
            if calulated_merkle_root
                != block.header.merkle_root {
                println!("Block merkle root does not match");
                return Err(BtcError::InvalidMerkleRoot)
            }
            // check if the block's timestamp is after the
            // last block's timestamp
            if block.header.timestamp
                <= last_block.header.timestamp {
                println!("Block timestamp is not greater than last block");
                return Err(BtcError::InvalidBlock)
            }
            // Verify all transactions in the block
            block.verify_transactions(
                self.block_height(),
                &self.utxos,
            )?;
        }

        let block_transactions: HashSet<_> = block
            .transactions
            .iter()
            .map(|tx| tx.hash())
            .collect();
        self.mempool.retain(|(_, tx) | {
            !block_transactions.contains(&tx.hash())
        });
        self.blocks.push(block);
        self.try_adjust_target();
        Ok(())
    }

    pub fn try_adjust_target(&mut self) {
        if self.blocks.is_empty()
        {
            return;
        }
        if self.blocks.len()
            & crate::DIFICULTY_UPDATE_INTERVAL as usize
            != 0
        {
            return;
        }
        let start_time = self.blocks[self.blocks.len()
            - crate::DIFICULTY_UPDATE_INTERVAL as usize]
            .header
            .timestamp;
        let end_time =
            self.blocks.last().unwrap().header.timestamp;
        let time_diff = end_time - start_time;
        let time_diff_secods = time_diff.num_seconds();
        let target_seconds = crate::IDEAL_BLOCK_TIME
            * crate::DIFICULTY_UPDATE_INTERVAL;

        let new_target = BigDecimal::parse_bytes(
            &self.target.to_string().as_bytes(),
            10,
        ).expect("BUG: impossible")
            * (BigDecimal::from(time_diff_secods)
            / BigDecimal::from(target_seconds));

        let new_target_str = new_target
            .to_string()
            .split('.')
            .next()
            .expect("BUG: Expected a decimal point")
            .to_owned();

        let new_target: U256 = U256::from_str_radix(
            &new_target_str, 10
        ).expect("BUG: impossible");

        let new_target = if new_target < self.target / 4 {
            self.target / 4
        } else if new_target > self.target * 4 {
            self.target * 4
        } else {
            new_target
        };

        self.target = new_target.min(crate::MIN_TARGET)
    }

    pub fn rebuild_utxos(&mut self) {
        for block in &self.blocks {
            for transaction in &block.transactions {
                for input in &transaction.inputs {
                    self.utxos.remove(
                        &input.prev_transaction_output_hash,
                    );
                }
                for output in transaction.outputs.iter() {
                    self.utxos.insert(
                        transaction.hash(),
                        output.clone(),
                    );
                }
            }
        }
    }

    pub fn block_height(&self) -> u64 {
        unimplemented!()
    }

    pub fn mempool(
        &self,
    ) -> &[Transaction] {
        &self.mempool
    }
}

impl Block {
    pub fn new(
        header: BlockHeader,
        transactions: Vec<Transaction>
    ) -> Self {
        Block { header, transactions }
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }

    pub fn verify_transactions(
        &self,
        predicted_block_height: u64,
        utxos: &HashMap<Hash, TransactionOutput>,
    ) -> Result<()> {
        let mut inputs: HashMap<Hash, TransactionOutput> = HashMap::new();

        if self.transactions.is_empty() {
            return Err(BtcError::InvalidTransaction)
        }

        self.verifty_coinbase_transaction(
            predicted_block_height,
            utxos
        )?;

        for transaction in self.transactions.iter().skip(1) {
            let mut input_value = 0;
            let mut output_value = 0;
            for input in &transaction.inputs {
                let prev_output = utxos.get(
                    &input.prev_transaction_output_hash,
                );
                if prev_output.is_none() {
                    return Err(
                        BtcError::InvalidTransaction,
                    )
                }

                let prev_output = prev_output.unwrap();

                if inputs.contains_key(&input.prev_transaction_output_hash) {
                    return Err(
                        BtcError::InvalidTransaction,
                    )
                }

                if !input.signature.verify(
                    &input.prev_transaction_output_hash,
                    &prev_output.pubkey,
                ) {
                    return Err(BtcError::InvalidSignature)
                }
                input_value += prev_output.value;
                inputs.insert(
                    input.prev_transaction_output_hash,
                    prev_output.clone(),
                );
            }

            for output in &transaction.outputs {
                output_value += output.value;
            }
            if input_value < output_value {
                return Err(BtcError::InvalidTransaction)
            }
        }
        Ok(())
    }

    pub fn verifty_coinbase_transaction(
        &self,
        predicted_block_height: u64,
        utxos: &HashMap<Hash, TransactionOutput>,
    ) -> Result<()> {
        let coinbase_transaction = &self.transactions[0];
        if coinbase_transaction.inputs.len() != 0 {
            return Err(BtcError::InvalidTransaction)
        }
        if coinbase_transaction.outputs.len() == 0 {
            return Err(BtcError::InvalidTransaction)
        }
        let miner_fees = self.calucate_miner_fees(utxos)?;
        let block_reward = crate::INITIAL_REWARD
            * 10u64.pow(8)
            / 2u64.pow(
            (predicted_block_height
                / crate::HALVING_INTERVAL)
                as u32,
        );

        let total_coinbase_outputs: u64 =
            coinbase_transaction
                .outputs
                .iter()
                .map(|output| output.value)
                .sum();

        if total_coinbase_outputs != block_reward + miner_fees {
            return Err(BtcError::InvalidTransaction)
        }
        Ok(())
    }

    pub fn calucate_miner_fees(
        &self,
        utxos: &HashMap<Hash, TransactionOutput>,
    ) -> Result<u64> {
        let mut inputs: HashMap<Hash, TransactionOutput> = HashMap::new();
        let mut outputs: HashMap<Hash, TransactionOutput> = HashMap::new();
        for transaction in self.transactions.iter().skip(1){
            for input in &transaction.inputs {
                let prev_output = utxos.get(
                    &input.prev_transaction_output_hash,
                );
                if prev_output.is_none() {
                    return Err(
                        BtcError::InvalidTransaction,
                    );
                }
                let prev_output = prev_output.unwrap();
                if inputs.contains_key(&input.prev_transaction_output_hash) {
                    return Err(BtcError::InvalidTransaction)
                }
                inputs.insert(
                    input.prev_transaction_output_hash,
                    prev_output.clone(),
                );
            }
            for output in &transaction.outputs {
                if outputs.contains_key(&output.hash()){
                    return Err(BtcError::InvalidTransaction);
                }
                outputs.insert(
                    output.hash(),
                    output.clone(),
                );
            }
        }
        let input_value: u64 = inputs
            .values()
            .map(|output| output.value)
            .sum();
        let output_values: u64 = outputs
            .values()
            .map(|output| output.value)
            .sum();
        Ok(input_value - output_values)
    }

}

impl BlockHeader {
    pub fn new(
        timestamp: DateTime<Utc>,
        nonce: u64,
        prev_block_hash: Hash,
        merkle_root: MerkleRoot,
        target: U256,
    ) -> Self {
        BlockHeader {
            timestamp,
            nonce,
            prev_block_hash,
            merkle_root,
            target,
        }
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

impl Transaction {
    pub fn new(
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
    ) -> Transaction {
        Transaction { inputs, outputs }
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}