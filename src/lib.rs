use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub mod prelude {
    pub use crate::Blockchain;
    pub use crate::Block;
    pub use crate::Transaction;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub index: u32,
    pub timestamp: u64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u32,
    pub difficulty: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<String>,
    pub difficulty: u32,
    pub mining_reward: f64,
    // For a simple node implementation
    pub nodes: HashMap<String, bool>, // URL -> is_active
}

impl Block {
    pub fn new(index: u32, data: String, previous_hash: String, difficulty: u32) -> Block {
        let timestamp = get_current_timestamp();
        let mut nonce = 0;
        let mut hash = calculate_hash(index, &previous_hash, timestamp, &data, nonce, difficulty);
        
        println!("Mining block {}...", index);
        
        // Mining process
        while !is_hash_valid(&hash, difficulty) {
            nonce += 1;
            hash = calculate_hash(index, &previous_hash, timestamp, &data, nonce, difficulty);
        }
        
        println!("Block mined: {}", hash);
        
        Block { 
            index, 
            timestamp, 
            data, 
            previous_hash, 
            hash, 
            nonce,
            difficulty,
        }
    }
}

// Helper functions
pub fn calculate_hash(index: u32, previous_hash: &str, timestamp: u64, data: &str, nonce: u32, difficulty: u32) -> String {
    let input = format!("{}{}{}{}{}{}", index, previous_hash, timestamp, data, nonce, difficulty);
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn is_hash_valid(hash: &str, difficulty: u32) -> bool {
    let prefix = "0".repeat(difficulty as usize);
    hash.starts_with(&prefix)
}

pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}


impl Blockchain {
    // Create a new blockchain with genesis block
    pub fn new(difficulty: u32, mining_reward: f64) -> Blockchain {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            difficulty,
            mining_reward,
            nodes: HashMap::new(),
        };
        
        // Create genesis block
        blockchain.create_genesis_block();
        blockchain
    }
    
    // Create the first block
    pub fn create_genesis_block(&mut self) {
        let genesis_block = Block::new(
            0,
            String::from("Genesis Block"),
            String::from("0"),
            self.difficulty
        );
        self.chain.push(genesis_block);
    }
    
    // Get the latest block
    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }
    
    // Add a new block to the chain
    pub fn add_block(&mut self, data: String) -> Result<(), String> {
        if let Some(latest_block) = self.get_latest_block() {
            let new_block = Block::new(
                latest_block.index + 1,
                data,
                latest_block.hash.clone(),
                self.difficulty
            );
            
            if self.is_block_valid(&new_block, latest_block) {
                self.chain.push(new_block);
                Ok(())
            } else {
                Err(String::from("Invalid block"))
            }
        } else {
            Err(String::from("Chain is empty"))
        }
    }
    
    // Validate a block
    pub fn is_block_valid(&self, block: &Block, previous_block: &Block) -> bool {
        // Check index
        if block.index != previous_block.index + 1 {
            println!("Invalid index");
            return false;
        }
        
        // Check previous hash
        if block.previous_hash != previous_block.hash {
            println!("Invalid previous hash");
            return false;
        }
        
        // Check hash
        let calculated_hash = calculate_hash(
            block.index,
            &block.previous_hash,
            block.timestamp,
            &block.data,
            block.nonce,
            block.difficulty
        );
        
        if block.hash != calculated_hash {
            println!("Invalid hash: {} vs {}", block.hash, calculated_hash);
            return false;
        }
        
        // Check if hash meets difficulty
        if !is_hash_valid(&block.hash, block.difficulty) {
            println!("Hash doesn't meet difficulty requirements");
            return false;
        }
        
        true
    }
    
    // Validate the entire chain
    pub fn is_chain_valid(&self) -> bool {
        if self.chain.is_empty() {
            return true;
        }
        
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];
            
            if !self.is_block_valid(current_block, previous_block) {
                return false;
            }
        }
        
        true
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
    pub timestamp: u64,
    pub signature: Option<String>, // Would be used in a real system
}

impl Transaction {
    pub fn new(sender: String, recipient: String, amount: f64) -> Transaction {
        Transaction {
            sender,
            recipient,
            amount,
            timestamp: get_current_timestamp(),
            signature: None,
        }
    }
    
    // In a real system, you'd implement signing here
    pub fn sign(&mut self, _private_key: &str) {
        // This would be a real signature in production
        self.signature = Some(String::from("signed"));
    }
    
    pub fn is_valid(&self) -> bool {
        // Simple validation for this example
        if self.sender.is_empty() || self.recipient.is_empty() {
            return false;
        }
        
        if self.amount <= 0.0 {
            return false;
        }
        
        // In a real system, verify signature here
        true
    }
}

// Update Blockchain struct
impl Blockchain {
    // Add a transaction to pending transactions
    pub fn create_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        if !transaction.is_valid() {
            return Err(String::from("Invalid transaction"));
        }
        
        let transaction_json = serde_json::to_string(&transaction)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        self.pending_transactions.push(transaction_json);
        Ok(())
    }
    
    // Mine pending transactions and reward the miner
    pub fn mine_pending_transactions(&mut self, miner_address: &str) -> Result<(), String> {
        // Create reward transaction
        let reward_transaction = Transaction::new(
            String::from("System"),
            miner_address.to_string(),
            self.mining_reward
        );
        
        let mut transactions = self.pending_transactions.clone();
        self.pending_transactions.clear();
        
        let reward_json = serde_json::to_string(&reward_transaction)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        transactions.push(reward_json);
        
        // Create a block with all transactions
        let transactions_data = transactions.join("|");
        self.add_block(transactions_data)?;
        
        Ok(())
    }
    
    // Get balance for an address
    pub fn get_balance_of_address(&self, address: &str) -> f64 {
        let mut balance = 0.0;
        
        for block in &self.chain {
            let transactions: Vec<&str> = block.data.split('|').collect();
            
            for transaction_json in transactions {
                if let Ok(transaction) = serde_json::from_str::<Transaction>(transaction_json) {
                    if transaction.recipient == address {
                        balance += transaction.amount;
                    }
                    
                    if transaction.sender == address {
                        balance -= transaction.amount;
                    }
                }
            }
        }
        balance

        
    }
}

impl Blockchain {
    // Register a new node
    pub fn register_node(&mut self, address: String) {
        self.nodes.insert(address, true);
    }
    
    // Consensus: resolve conflicts by replacing our chain with the longest valid chain
    pub fn resolve_conflicts(&mut self, other_chains: Vec<Vec<Block>>) -> bool {
        let mut new_chain: Option<Vec<Block>> = None;
        let mut max_length = self.chain.len();
        
        // Look for chains longer than ours
        for chain in other_chains {
            let length = chain.len();
            
            // Check if the chain is longer and valid
            if length > max_length {
                let temp_blockchain = Blockchain {
                    chain: chain.clone(),
                    pending_transactions: Vec::new(),
                    difficulty: self.difficulty,
                    mining_reward: self.mining_reward,
                    nodes: HashMap::new(),
                };
                
                if temp_blockchain.is_chain_valid() {
                    max_length = length;
                    new_chain = Some(chain);
                }
            }
        }
        
        // Replace our chain if we found a longer valid one
        if let Some(chain) = new_chain {
            self.chain = chain;
            true
        } else {
            false
        }
    }
}

impl Blockchain {
    // Save blockchain to a file
    pub fn save_to_file(&self, filename: &str) -> Result<(), String> {
        let json = serde_json::to_string(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        fs::write(filename, json)
            .map_err(|e| format!("File write error: {}", e))?;
        
        Ok(())
    }
    
    // Load blockchain from a file
    pub fn load_from_file(filename: &str) -> Result<Blockchain, String> {
        if !Path::new(filename).exists() {
            return Err(format!("File {} does not exist", filename));
        }
        
        let json = fs::read_to_string(filename)
            .map_err(|e| format!("File read error: {}", e))?;
        
        serde_json::from_str(&json)
            .map_err(|e| format!("Deserialization error: {}", e))
    }
}

// Example with simple networking (pseudocode)
// In a real implementation, you'd use a proper web framework like Actix

pub fn handle_get_chain(blockchain: &Blockchain) -> String {
    serde_json::to_string(blockchain).unwrap_or_default()
}

pub fn handle_mine_block(blockchain: &mut Blockchain, miner_address: &str) -> String {
    match blockchain.mine_pending_transactions(miner_address) {
        Ok(_) => format!("Block mined successfully. Reward sent to {}", miner_address),
        Err(e) => format!("Error mining block: {}", e),
    }
}

pub fn handle_new_transaction(blockchain: &mut Blockchain, sender: &str, recipient: &str, amount: f64) -> String {
    let transaction = Transaction::new(
        sender.to_string(),
        recipient.to_string(),
        amount
    );
    
    match blockchain.create_transaction(transaction) {
        Ok(_) => String::from("Transaction added to pending transactions"),
        Err(e) => format!("Error creating transaction: {}", e),
    }
}

pub fn handle_get_balance(blockchain: &Blockchain, address: &str) -> String {
    let balance = blockchain.get_balance_of_address(address);
    format!("Balance of {}: {}", address, balance)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;

    // Helper function to create a test blockchain
    fn create_test_blockchain() -> Blockchain {
        Blockchain::new(2, 100.0) // Lower difficulty for faster tests
    }

    #[test]
    fn test_genesis_block_creation() {
        let blockchain = create_test_blockchain();
        
        // Check chain has exactly one block
        assert_eq!(blockchain.chain.len(), 1);
        
        // Check genesis block properties
        let genesis = &blockchain.chain[0];
        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.previous_hash, "0");
        assert_eq!(genesis.data, "Genesis Block");
        assert!(is_hash_valid(&genesis.hash, genesis.difficulty));
    }

    #[test]
    fn test_add_block() {
        let mut blockchain = create_test_blockchain();
        let initial_length = blockchain.chain.len();
        
        // Add a new block
        blockchain.add_block("Test Block Data".to_string()).unwrap();
        
        // Check chain length increased
        assert_eq!(blockchain.chain.len(), initial_length + 1);
        
        // Check new block properties
        let new_block = blockchain.chain.last().unwrap();
        assert_eq!(new_block.index, 1);
        assert_eq!(new_block.data, "Test Block Data");
        assert_eq!(new_block.previous_hash, blockchain.chain[0].hash);
        assert!(is_hash_valid(&new_block.hash, new_block.difficulty));
    }

    #[test]
    fn test_block_validation() {
        let mut blockchain = create_test_blockchain();
        blockchain.add_block("Test Block".to_string()).unwrap();
        
        let latest_block = blockchain.get_latest_block().unwrap();
        let previous_block = &blockchain.chain[blockchain.chain.len() - 2];
        
        // Valid block should pass validation
        assert!(blockchain.is_block_valid(latest_block, previous_block));
        
        // Create an invalid block with wrong index
        let mut invalid_block = latest_block.clone();
        invalid_block.index = 999;
        assert!(!blockchain.is_block_valid(&invalid_block, previous_block));
        
        // Create an invalid block with wrong previous hash
        let mut invalid_block = latest_block.clone();
        invalid_block.previous_hash = "invalid_hash".to_string();
        assert!(!blockchain.is_block_valid(&invalid_block, previous_block));
        
        // Create an invalid block with modified data (hash won't match)
        let mut invalid_block = latest_block.clone();
        invalid_block.data = "Tampered data".to_string();
        assert!(!blockchain.is_block_valid(&invalid_block, previous_block));
        
        // Create an invalid block with invalid hash
        let mut invalid_block = latest_block.clone();
        invalid_block.hash = "invalid_hash".to_string();
        assert!(!blockchain.is_block_valid(&invalid_block, previous_block));
    }

    #[test]
    fn test_chain_validation() {
        let mut blockchain = create_test_blockchain();
        
        // Add a few blocks
        blockchain.add_block("Block 1".to_string()).unwrap();
        blockchain.add_block("Block 2".to_string()).unwrap();
        blockchain.add_block("Block 3".to_string()).unwrap();
        
        // Chain should be valid
        assert!(blockchain.is_chain_valid());
        
        // Tamper with a block in the middle and verify chain is invalid
        blockchain.chain[2].data = "Tampered Block 2".to_string();
        assert!(!blockchain.is_chain_valid());
    }

    #[test]
    fn test_mining_difficulty() {
        // Create blockchains with different difficulties
        let mut blockchain_easy = Blockchain::new(1, 100.0);
        let mut blockchain_hard = Blockchain::new(4, 100.0);
        
        // Track time to mine blocks
        let start_easy = SystemTime::now();
        blockchain_easy.add_block("Easy Block".to_string()).unwrap();
        let duration_easy = SystemTime::now()
            .duration_since(start_easy)
            .unwrap_or_else(|_| Duration::from_secs(0));
        
        let start_hard = SystemTime::now();
        blockchain_hard.add_block("Hard Block".to_string()).unwrap();
        let duration_hard = SystemTime::now()
            .duration_since(start_hard)
            .unwrap_or_else(|_| Duration::from_secs(0));
        
        // Check that harder difficulty took longer to mine
        assert!(duration_hard > duration_easy);
        
        // Check hash patterns
        let easy_block = blockchain_easy.get_latest_block().unwrap();
        let hard_block = blockchain_hard.get_latest_block().unwrap();
        
        assert!(easy_block.hash.starts_with("0"));
        assert!(hard_block.hash.starts_with("0000"));
    }

    #[test]
    fn test_transactions() {
        let mut blockchain = create_test_blockchain();
        
        // Create transactions
        let tx1 = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            50.0
        );
        
        let tx2 = Transaction::new(
            "Bob".to_string(),
            "Charlie".to_string(),
            25.0
        );
        
        // Add transactions and mine
        blockchain.create_transaction(tx1).unwrap();
        blockchain.create_transaction(tx2).unwrap();
        blockchain.mine_pending_transactions("Miner1").unwrap();
        
        // Check balances
        assert_eq!(blockchain.get_balance_of_address("Alice"), -50.0);
        assert_eq!(blockchain.get_balance_of_address("Bob"), 25.0);
        assert_eq!(blockchain.get_balance_of_address("Charlie"), 25.0);
        assert_eq!(blockchain.get_balance_of_address("Miner1"), 100.0);
        
        // Add more transactions and mine again
        let tx3 = Transaction::new(
            "Charlie".to_string(),
            "Alice".to_string(),
            10.0
        );
        
        blockchain.create_transaction(tx3).unwrap();
        blockchain.mine_pending_transactions("Miner1").unwrap();
        
        // Check updated balances
        assert_eq!(blockchain.get_balance_of_address("Alice"), -40.0);
        assert_eq!(blockchain.get_balance_of_address("Bob"), 25.0);
        assert_eq!(blockchain.get_balance_of_address("Charlie"), 15.0);
        assert_eq!(blockchain.get_balance_of_address("Miner1"), 200.0);
    }

    #[test]
    fn test_transaction_validation() {
        // Valid transaction
        let valid_tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            50.0
        );
        assert!(valid_tx.is_valid());
        
        // Invalid transactions
        let invalid_sender = Transaction::new(
            "".to_string(),
            "Bob".to_string(),
            50.0
        );
        assert!(!invalid_sender.is_valid());
        
        let invalid_recipient = Transaction::new(
            "Alice".to_string(),
            "".to_string(),
            50.0
        );
        assert!(!invalid_recipient.is_valid());
        
        let invalid_amount = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            -10.0
        );
        assert!(!invalid_amount.is_valid());
    }

    #[test]
    fn test_file_persistence() {
        let mut blockchain = create_test_blockchain();
        
        // Add some blocks and transactions
        blockchain.add_block("Test Block 1".to_string()).unwrap();
        
        let tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            30.0
        );
        
        blockchain.create_transaction(tx).unwrap();
        blockchain.mine_pending_transactions("Miner1").unwrap();
        
        // Save to file
        let filename = "test_blockchain.json";
        blockchain.save_to_file(filename).unwrap();
        
        // Load from file
        let loaded_blockchain = Blockchain::load_from_file(filename).unwrap();
        
        // Verify loaded blockchain matches original
        assert_eq!(loaded_blockchain.chain.len(), blockchain.chain.len());
        assert_eq!(loaded_blockchain.difficulty, blockchain.difficulty);
        assert_eq!(loaded_blockchain.mining_reward, blockchain.mining_reward);
        
        // Cleanup test file
        let _ = fs::remove_file(filename);
    }

    #[test]
    fn test_consensus_mechanism() {
        let mut blockchain1 = create_test_blockchain();
        let mut blockchain2 = create_test_blockchain();
        
        // Make blockchain1 longer
        blockchain1.add_block("Block 1-1".to_string()).unwrap();
        blockchain1.add_block("Block 1-2".to_string()).unwrap();
        
        // Make blockchain2 with only one additional block
        blockchain2.add_block("Block 2-1".to_string()).unwrap();
        
        // Create a collection of chains
        let chains = vec![
            blockchain1.chain.clone(),
            blockchain2.chain.clone(),
        ];
        
        // Test consensus - blockchain2 should adopt the longer chain
        let changed = blockchain2.resolve_conflicts(chains);
        assert!(changed);
        assert_eq!(blockchain2.chain.len(), 3); // Genesis + 2 blocks
        
        // The chains should now be identical
        assert_eq!(blockchain2.chain[1].data, "Block 1-1");
        assert_eq!(blockchain2.chain[2].data, "Block 1-2");
    }

    #[test]
    fn test_node_registration() {
        let mut blockchain = create_test_blockchain();
        
        // Register nodes
        blockchain.register_node("http://localhost:3001".to_string());
        blockchain.register_node("http://localhost:3002".to_string());
        
        // Check nodes were registered
        assert!(blockchain.nodes.contains_key("http://localhost:3001"));
        assert!(blockchain.nodes.contains_key("http://localhost:3002"));
        assert_eq!(blockchain.nodes.len(), 2);
        
        // Register same node again (should not duplicate)
        blockchain.register_node("http://localhost:3001".to_string());
        assert_eq!(blockchain.nodes.len(), 2);
    }

    #[test]
    fn test_mining_empty_transactions() {
        let mut blockchain = create_test_blockchain();
        
        // Mine block with no pending transactions (just mining reward)
        blockchain.mine_pending_transactions("Miner1").unwrap();
        
        // There should be a new block with the reward transaction
        assert_eq!(blockchain.chain.len(), 2);
        assert_eq!(blockchain.get_balance_of_address("Miner1"), 100.0);
        
        // Pending transactions should be empty
        assert_eq!(blockchain.pending_transactions.len(), 0);
    }

    #[test]
    fn test_concurrent_mining() {
        let mut blockchain = create_test_blockchain();
        
        // Add some transactions
        let tx1 = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            20.0
        );
        
        let tx2 = Transaction::new(
            "Charlie".to_string(),
            "Dave".to_string(),
            30.0
        );
        
        blockchain.create_transaction(tx1).unwrap();
        blockchain.create_transaction(tx2).unwrap();
        
        // Mine in the main thread
        blockchain.mine_pending_transactions("Miner1").unwrap();
        
        // Add more transactions
        let tx3 = Transaction::new(
            "Eve".to_string(),
            "Frank".to_string(),
            15.0
        );
        
        blockchain.create_transaction(tx3).unwrap();
        
        // Mine in a separate thread to simulate concurrent mining
        let blockchain_clone = blockchain.clone();
        let handle = thread::spawn(move || {
            let mut bc = blockchain_clone;
            bc.mine_pending_transactions("Miner2").unwrap();
            bc
        });
        
        // Wait for the thread to finish
        thread::sleep(Duration::from_millis(100));
        
        // Mine in the main thread too
        blockchain.mine_pending_transactions("Miner1").unwrap();
        
        // Get the result from the thread
        let thread_blockchain = handle.join().unwrap();
        
        // Both blockchains are valid but may have different chains
        assert!(blockchain.is_chain_valid());
        assert!(thread_blockchain.is_chain_valid());
        
        // They should have different latest blocks (different miners)
        let main_last_block = blockchain.get_latest_block().unwrap();
        let thread_last_block = thread_blockchain.get_latest_block().unwrap();
        
        // Different miners = different blocks (even with same transactions)
        assert_ne!(main_last_block.hash, thread_last_block.hash);
    }

    #[test]
    fn test_malicious_balance_change() {
        let mut blockchain = create_test_blockchain();
        
        // Add a legitimate transaction
        let tx = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            50.0
        );
        
        blockchain.create_transaction(tx).unwrap();
        blockchain.mine_pending_transactions("Miner1").unwrap();
        
        // Initial balance check
        assert_eq!(blockchain.get_balance_of_address("Alice"), -50.0);
        assert_eq!(blockchain.get_balance_of_address("Bob"), 50.0);
        
        // Attempt to tamper with a previous block
        // This is a simulated attack where someone tries to modify transaction data
        let block_data = &mut blockchain.chain[1].data;
        
        // Parse transactions
        let transactions: Vec<&str> = block_data.split('|').collect();
        let mut modified_transactions = Vec::new();
        
        for tx_json in transactions {
            if let Ok(mut tx) = serde_json::from_str::<Transaction>(tx_json) {
                if tx.sender == "Alice" && tx.recipient == "Bob" {
                    // Try to change the amount
                    tx.amount = 1.0; // Change from 50.0 to 1.0
                }
                let modified_json = serde_json::to_string(&tx).unwrap();
                modified_transactions.push(modified_json);
            } else {
                modified_transactions.push(tx_json.to_string());
            }
        }
        
        // Replace block data with modified transactions
        *block_data = modified_transactions.join("|");
        
        // The chain should no longer be valid after tampering
        assert!(!blockchain.is_chain_valid());
        
        // If someone tried to use this tampered chain, validation would fail
        // In a real system, other nodes would reject this chain
    }

    #[test]
    fn test_large_blockchain() {
        let mut blockchain = create_test_blockchain();
        
        // Add many blocks to test performance and stability
        for i in 1..10 {
            blockchain.add_block(format!("Test Block {}", i)).unwrap();
        }
        
        // Chain should still be valid
        assert!(blockchain.is_chain_valid());
        assert_eq!(blockchain.chain.len(), 11); // Genesis + 10 blocks
        
        // Each block should link to the previous one
        for i in 1..blockchain.chain.len() {
            assert_eq!(blockchain.chain[i].previous_hash, blockchain.chain[i-1].hash);
        }
    }
}