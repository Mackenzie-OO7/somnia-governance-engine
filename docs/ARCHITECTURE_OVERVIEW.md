# Architecture Overview - Modular Design

## 🧩 Key Clarifications

### Smart Contracts - Optional and Replaceable

**YES, our smart contracts are completely optional!** The Somnia Governance Engine is designed as a modular toolkit where you can:

#### Option 1: Use Your Own Contracts + Our Rust Engine
```rust
// Configure our Rust engine to work with YOUR contracts
use somnia_governance_engine::blockchain::ContractManager;

let mut contract_manager = ContractManager::new(config).await?;

// Point to YOUR contract addresses instead of ours
contract_manager.configure_custom_contracts(CustomContracts {
    governance_address: "0xYourGovernanceContract",
    token_address: "0xYourTokenContract",
    // Our engine adapts to your contract interfaces
}).await?;

// Now use our APIs, event monitoring, database layer with YOUR contracts
let proposal_id = contract_manager.create_proposal_on_custom_contract(
    "Your proposal data"
).await?;
```

#### Option 2: Pure Rust Library (No Blockchain)
```rust
// Use just the governance logic without any smart contracts
use somnia_governance_engine::core::{VotingEngine, SignatureVerifier};

let mut governance = VotingEngine::new();
let proposal_id = governance.create_signed_proposal(signed_proposal).await?;
governance.cast_signed_vote(proposal_id, signed_vote).await?;
```

#### Option 3: Extend Our Contracts
```solidity
// Inherit from our contracts and add your features
import "./GovernanceHub.sol";

contract MyCustomGovernance is GovernanceHub {
    // Add your custom logic here
    function myCustomFunction() external {
        // Your implementation
    }
}
```

### Dependency Requirements Clarified

#### Node.js - When You Need It:
```bash
# ✅ REQUIRED for:
# 1. Deploying/modifying smart contracts (Foundry ecosystem)
cd contracts && forge build  # This needs Node.js

# 2. Frontend integration with ethers.js
npm install ethers

# 3. Running contract build/test system
forge test

# ❌ NOT NEEDED for:
# 1. Pure Rust library usage
cargo add somnia-governance-engine  # No Node.js needed

# 2. Connecting to existing contracts
# 3. Off-chain governance only
```

#### PostgreSQL - When You Need It:
```bash
# ✅ REQUIRED for:
# 1. Using our database persistence layer
DATABASE_URL=postgresql://localhost/governance

# ❌ NOT NEEDED for:
# 1. In-memory governance (testing/simple use)
let engine = VotingEngine::in_memory().await?;

# 2. Your own database
let engine = VotingEngine::with_custom_storage(your_db).await?;

# 3. File-based storage
let engine = VotingEngine::with_file_storage("./data").await?;

# 4. Pure computation without persistence
```

### Minimal Integration Examples

#### Example 1: Just Governance Logic (Zero External Dependencies)
```rust
// Cargo.toml
[dependencies]
somnia-governance-engine = { path = "path/to/backend", default-features = false, features = ["core"] }

// main.rs
use somnia_governance_engine::core::{Proposal, Vote, GovernanceEngine};

#[tokio::main]
async fn main() -> Result<()> {
    // No database, no blockchain, no external services
    let mut engine = GovernanceEngine::in_memory();

    let proposal = Proposal::new("Upgrade protocol", "Description here");
    let id = engine.create_proposal(proposal).await?;

    engine.vote(id, Vote::For, "0xvoter").await?;

    let result = engine.get_results(id).await?;
    println!("Result: {:?}", result);

    Ok(())
}
```

#### Example 2: With Your Existing Contracts
```rust
// Point our engine to your deployed contracts
let config = Config {
    // Your contract addresses
    governance_hub_address: "0xYourGovernanceContract",
    token_address: "0xYourTokenContract",

    // No need for our contract addresses
    rpc_url: "https://your-rpc",

    // Optional: skip database if you don't need persistence
    database_url: None,
};

let engine = GovernanceEngine::with_config(config).await?;

// Now our Rust APIs work with your contracts
let proposal_id = engine.create_proposal_via_your_contract().await?;
```

#### Example 3: Custom Storage (No PostgreSQL)
```rust
// Implement your storage
struct MyStorage;

impl StorageAdapter for MyStorage {
    async fn store_proposal(&self, proposal: &Proposal) -> Result<()> {
        // Store in Redis, MongoDB, files, etc.
        Ok(())
    }
}

// Use with our engine
let engine = GovernanceEngine::with_storage(MyStorage).await?;
```

## 🎯 Choose Your Integration Level

| Component | Always Required | Optional |
|-----------|----------------|----------|
| Rust Library | ✅ | |
| Our Smart Contracts | | ✅ (use yours instead) |
| Node.js | | ✅ (only for contract dev) |
| PostgreSQL | | ✅ (many storage options) |
| Foundry | | ✅ (only for our contracts) |

## 🚀 Integration Approaches

### Approach 1: Minimal (Pure Rust)
```
Your App ◄── Rust Governance Engine
```
- **Dependencies**: Just Rust
- **Use case**: Off-chain governance, testing, simple voting
- **Setup time**: < 5 minutes

### Approach 2: With Your Contracts
```
Your App ◄── Rust Engine ◄── Your Smart Contracts
```
- **Dependencies**: Rust + Your existing blockchain setup
- **Use case**: Add governance to existing DeFi/DAO project
- **Setup time**: < 30 minutes

### Approach 3: Full Stack (All Components)
```
Your App ◄── Rust Engine ◄── Our Contracts ◄── Blockchain
           ▲
    PostgreSQL Database
```
- **Dependencies**: Everything
- **Use case**: New governance system from scratch
- **Setup time**: 1-2 hours

## 📋 Quick Decision Guide

**Use our smart contracts if:**
- ✅ You're building governance from scratch
- ✅ You want production-tested security patterns
- ✅ You need timelock, delegation, anti-spam features

**Use your smart contracts if:**
- ✅ You already have governance contracts
- ✅ You need custom governance logic
- ✅ You want to add our Rust tooling to existing system

**Skip PostgreSQL if:**
- ✅ You have existing database infrastructure
- ✅ You're doing simple/testing scenarios
- ✅ You prefer other storage solutions (Redis, MongoDB, files)

**Skip Node.js if:**
- ✅ You're only using the Rust library
- ✅ You're not working with smart contracts
- ✅ You have your own contract deployment process