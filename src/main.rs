use rapid_blockchain::prelude::*;

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