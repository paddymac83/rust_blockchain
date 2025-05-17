use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Block {
    index: u32,
    timestamp: u64,
    data: String,
    previous_hash: String,
    hash: String,
    nonce: u32,
    difficulty: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<String>,
    difficulty: u32,
    mining_reward: f64,
    // For a simple node implementation
    nodes: HashMap<String, bool>, // URL -> is_active
}

impl Block {
    fn new(index: u32, data: String, previous_hash: String, difficulty: u32) -> Block {
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
fn calculate_hash(index: u32, previous_hash: &str, timestamp: u64, data: &str, nonce: u32, difficulty: u32) -> String {
    let input = format!("{}{}{}{}{}{}", index, previous_hash, timestamp, data, nonce, difficulty);
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn is_hash_valid(hash: &str, difficulty: u32) -> bool {
    let prefix = "0".repeat(difficulty as usize);
    hash.starts_with(&prefix)
}

fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}


impl Blockchain {
    // Create a new blockchain with genesis block
    fn new(difficulty: u32, mining_reward: f64) -> Blockchain {
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
    fn create_genesis_block(&mut self) {
        let genesis_block = Block::new(
            0,
            String::from("Genesis Block"),
            String::from("0"),
            self.difficulty
        );
        self.chain.push(genesis_block);
    }
    
    // Get the latest block
    fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }
    
    // Add a new block to the chain
    fn add_block(&mut self, data: String) -> Result<(), String> {
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
    fn is_block_valid(&self, block: &Block, previous_block: &Block) -> bool {
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
    fn is_chain_valid(&self) -> bool {
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
struct Transaction {
    sender: String,
    recipient: String,
    amount: f64,
    timestamp: u64,
    signature: Option<String>, // Would be used in a real system
}

impl Transaction {
    fn new(sender: String, recipient: String, amount: f64) -> Transaction {
        Transaction {
            sender,
            recipient,
            amount,
            timestamp: get_current_timestamp(),
            signature: None,
        }
    }
    
    // In a real system, you'd implement signing here
    fn sign(&mut self, _private_key: &str) {
        // This would be a real signature in production
        self.signature = Some(String::from("signed"));
    }
    
    fn is_valid(&self) -> bool {
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
    fn create_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        if !transaction.is_valid() {
            return Err(String::from("Invalid transaction"));
        }
        
        let transaction_json = serde_json::to_string(&transaction)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        self.pending_transactions.push(transaction_json);
        Ok(())
    }
    
    // Mine pending transactions and reward the miner
    fn mine_pending_transactions(&mut self, miner_address: &str) -> Result<(), String> {
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
    fn get_balance_of_address(&self, address: &str) -> f64 {
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
    fn register_node(&mut self, address: String) {
        self.nodes.insert(address, true);
    }
    
    // Consensus: resolve conflicts by replacing our chain with the longest valid chain
    fn resolve_conflicts(&mut self, other_chains: Vec<Vec<Block>>) -> bool {
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

use std::fs;
use std::path::Path;

impl Blockchain {
    // Save blockchain to a file
    fn save_to_file(&self, filename: &str) -> Result<(), String> {
        let json = serde_json::to_string(self)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        fs::write(filename, json)
            .map_err(|e| format!("File write error: {}", e))?;
        
        Ok(())
    }
    
    // Load blockchain from a file
    fn load_from_file(filename: &str) -> Result<Blockchain, String> {
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

fn handle_get_chain(blockchain: &Blockchain) -> String {
    serde_json::to_string(blockchain).unwrap_or_default()
}

fn handle_mine_block(blockchain: &mut Blockchain, miner_address: &str) -> String {
    match blockchain.mine_pending_transactions(miner_address) {
        Ok(_) => format!("Block mined successfully. Reward sent to {}", miner_address),
        Err(e) => format!("Error mining block: {}", e),
    }
}

fn handle_new_transaction(blockchain: &mut Blockchain, sender: &str, recipient: &str, amount: f64) -> String {
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

fn handle_get_balance(blockchain: &Blockchain, address: &str) -> String {
    let balance = blockchain.get_balance_of_address(address);
    format!("Balance of {}: {}", address, balance)
}

fn main() {
    // Create a new blockchain with difficulty 4 and 100 coins mining reward
    let mut blockchain = Blockchain::new(4, 100.0);
    
    println!("Mining genesis block...");
    
    // Add some transactions
    let tx1 = Transaction::new(
        String::from("Alice"),
        String::from("Bob"),
        50.0
    );
    
    let tx2 = Transaction::new(
        String::from("Bob"),
        String::from("Charlie"),
        25.0
    );
    
    blockchain.create_transaction(tx1).unwrap();
    blockchain.create_transaction(tx2).unwrap();
    
    println!("Starting mining...");
    blockchain.mine_pending_transactions("Miner1").unwrap();
    
    // Create more transactions
    let tx3 = Transaction::new(
        String::from("Charlie"),
        String::from("Alice"),
        10.0
    );
    
    blockchain.create_transaction(tx3).unwrap();
    blockchain.mine_pending_transactions("Miner1").unwrap();
    
    // Check balance
    println!("Balance of Miner1: {}", blockchain.get_balance_of_address("Miner1"));
    println!("Balance of Alice: {}", blockchain.get_balance_of_address("Alice"));
    println!("Balance of Bob: {}", blockchain.get_balance_of_address("Bob"));
    println!("Balance of Charlie: {}", blockchain.get_balance_of_address("Charlie"));
    
    // Validate the chain
    println!("Is blockchain valid? {}", blockchain.is_chain_valid());
    
    // Save and load the blockchain
    blockchain.save_to_file("blockchain.json").unwrap();
    let loaded_blockchain = Blockchain::load_from_file("blockchain.json").unwrap();
    
    println!("Loaded blockchain has {} blocks", loaded_blockchain.chain.len());
}