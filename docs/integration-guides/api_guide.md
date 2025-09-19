# API Guide

This guide provides comprehensive documentation for using the Somnia Governance Engine as a Rust library.

## Table of Contents

- [Getting Started](#getting-started)
- [Core Components](#core-components)
- [Configuration](#configuration)
- [Blockchain Integration](#blockchain-integration)
- [Database Operations](#database-operations)
- [API Usage](#api-usage)
- [Event Monitoring](#event-monitoring)
- [Error Handling](#error-handling)
- [Examples](#examples)

## Getting Started

### Adding the Dependency

Choose the features you need:

```toml
[dependencies]
# Minimal - just governance logic (no external dependencies)
somnia-governance-engine = { path = "path/to/backend", default-features = false, features = ["core"] }

# With blockchain integration (requires RPC endpoint)
somnia-governance-engine = { path = "path/to/backend", features = ["blockchain"] }

# With database support (requires PostgreSQL)
somnia-governance-engine = { path = "path/to/backend", features = ["database"] }

# Full features (requires PostgreSQL + RPC)
somnia-governance-engine = { path = "path/to/backend" }

# Common additional dependencies
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
```

### Dependency Requirements by Feature

| Feature | External Dependencies | Use Case |
|---------|----------------------|----------|
| `core` | None | Pure governance logic, testing |
| `blockchain` | RPC endpoint | Interact with smart contracts |
| `database` | PostgreSQL | Persistent storage |
| `api` | PostgreSQL + RPC | Full REST API server |

### Basic Setup

```rust
use somnia_governance_engine::{
    config::Config,
    blockchain::ContractManager,
    database::Database,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from environment
    let config = Config::from_env()?;

    // Initialize database connection
    let db = Database::connect(&config.database_url).await?;

    // Initialize blockchain contract manager
    let contract_manager = ContractManager::new(config.clone()).await?;

    println!("Governance engine initialized successfully!");

    Ok(())
}
```

## Core Components

### 1. Configuration (`config::Config`)

The configuration module handles all environment variables and settings.

```rust
use somnia_governance_engine::config::Config;

// Load from environment variables
let config = Config::from_env()?;

// Access configuration values
println!("RPC URL: {}", config.rpc_url);
println!("Chain ID: {}", config.chain_id);
println!("Server port: {}", config.server.port);

// Custom configuration
let custom_config = Config {
    rpc_url: "https://custom-rpc.com".to_string(),
    chain_id: 1234,
    private_key: "0x...".to_string(),
    database_url: "postgresql://localhost/governance".to_string(),
    contracts: ContractAddresses {
        governance_hub: "0x123...".to_string(),
        simple_voting: "0x456...".to_string(),
        governance_token: "0x789...".to_string(),
        timelock: "0xabc...".to_string(),
    },
    server: ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 3000,
    },
};
```

### 2. Contract Manager (`blockchain::ContractManager`)

The contract manager handles all blockchain interactions.

```rust
use somnia_governance_engine::{
    blockchain::ContractManager,
    models::{ProposalType, VoteChoice},
};

let contract_manager = ContractManager::new(config).await?;

// Create a proposal
let proposal_id = contract_manager.create_proposal(
    "QmProposalContent123",
    86400, // 24 hours
    ProposalType::Standard,
).await?;

// Vote on a proposal
contract_manager.vote(
    proposal_id,
    VoteChoice::For,
    Some("Reason for voting".to_string()),
).await?;

// Get proposal details
let proposal = contract_manager.get_proposal(proposal_id).await?;
println!("Proposal status: {:?}", proposal.status);

// Execute a proposal (after voting period)
contract_manager.execute_proposal(proposal_id).await?;
```

### 3. Database Operations (`database::Database`)

Persistent storage for governance data and caching.

```rust
use somnia_governance_engine::{
    database::Database,
    models::{Proposal, Vote, NewProposal},
};

let db = Database::connect("postgresql://localhost/governance").await?;

// Store a proposal
let new_proposal = NewProposal {
    id: 1,
    ipfs_hash: "QmHash123".to_string(),
    creator: "0x123...".to_string(),
    proposal_type: "standard".to_string(),
    status: "active".to_string(),
};

db.create_proposal(&new_proposal).await?;

// Retrieve proposals
let proposals = db.get_all_proposals().await?;
let proposal = db.get_proposal_by_id(1).await?;

// Store a vote
let vote = Vote {
    id: 1,
    proposal_id: 1,
    voter: "0x456...".to_string(),
    choice: "for".to_string(),
    voting_power: "1000000000000000000000".to_string(),
    reasoning: Some("Good proposal".to_string()),
    created_at: chrono::Utc::now(),
};

db.create_vote(&vote).await?;
```

## Configuration

### Environment Variables

Create a `.env` file in your project root:

```bash
# Database Configuration
DATABASE_URL=postgresql://user:password@localhost/governance_db

# Blockchain Configuration
RPC_URL=https://somnia-testnet-rpc-url
CHAIN_ID=1234
PRIVATE_KEY=0xYourPrivateKeyHere

# Contract Addresses
GOVERNANCE_HUB_ADDRESS=0x1234567890123456789012345678901234567890
SIMPLE_VOTING_ADDRESS=0x2345678901234567890123456789012345678901
GOVERNANCE_TOKEN_ADDRESS=0x3456789012345678901234567890123456789012
TIMELOCK_ADDRESS=0x4567890123456789012345678901234567890123

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Security
JWT_SECRET=your-jwt-secret-key
CORS_ORIGINS=http://localhost:3000,https://yourdomain.com

# Logging
RUST_LOG=info
```

### Programmatic Configuration

```rust
use somnia_governance_engine::config::{Config, ContractAddresses, ServerConfig};

let config = Config {
    database_url: "postgresql://localhost/governance".to_string(),
    rpc_url: "https://somnia-rpc.com".to_string(),
    chain_id: 1234,
    private_key: "0x...".to_string(),
    contracts: ContractAddresses {
        governance_hub: "0x123...".to_string(),
        simple_voting: "0x456...".to_string(),
        governance_token: "0x789...".to_string(),
        timelock: "0xabc...".to_string(),
    },
    server: ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 3000,
    },
    jwt_secret: "secret".to_string(),
    cors_origins: vec!["http://localhost:3000".to_string()],
};
```

## Blockchain Integration

### Proposal Management

```rust
use somnia_governance_engine::{
    blockchain::ContractManager,
    models::{ProposalType, ProposalStatus},
};

let contract_manager = ContractManager::new(config).await?;

// Create different types of proposals
let standard_proposal = contract_manager.create_proposal(
    "QmStandardProposal",
    86400, // 24 hours
    ProposalType::Standard,
).await?;

let emergency_proposal = contract_manager.create_proposal(
    "QmEmergencyProposal",
    3600, // 1 hour
    ProposalType::Emergency,
).await?;

let constitutional_proposal = contract_manager.create_proposal(
    "QmConstitutionalProposal",
    604800, // 7 days
    ProposalType::Constitutional,
).await?;

// Check proposal status
let proposal = contract_manager.get_proposal(standard_proposal).await?;
match proposal.status {
    ProposalStatus::Active => println!("Proposal is accepting votes"),
    ProposalStatus::Succeeded => println!("Proposal passed and can be executed"),
    ProposalStatus::Failed => println!("Proposal failed to meet requirements"),
    ProposalStatus::Executed => println!("Proposal has been executed"),
}
```

### Voting Operations

```rust
use somnia_governance_engine::models::VoteChoice;

// Cast votes with different choices
contract_manager.vote(
    proposal_id,
    VoteChoice::For,
    Some("I support this proposal because...".to_string()),
).await?;

contract_manager.vote(
    proposal_id,
    VoteChoice::Against,
    Some("I oppose this proposal because...".to_string()),
).await?;

contract_manager.vote(
    proposal_id,
    VoteChoice::Abstain,
    None, // No reasoning required for abstain
).await?;

// Check if user has voted
let has_voted = contract_manager.has_voted(proposal_id, "0x123...").await?;
if has_voted {
    println!("User has already voted on this proposal");
}

// Get vote details
let vote = contract_manager.get_vote(proposal_id, "0x123...").await?;
println!("User voted: {:?} with power: {}", vote.choice, vote.voting_power);
```

### Simple Voting Sessions

```rust
use somnia_governance_engine::blockchain::SimpleVotingManager;

let voting_manager = SimpleVotingManager::new(config).await?;

// Create a voting session
let session_id = voting_manager.create_session(
    "Should we implement feature X?",
    3600, // 1 hour
    "QmSessionDetails",
    0, // Use default quorum
).await?;

// Vote in session
voting_manager.vote_in_session(
    session_id,
    true, // Yes vote
).await?;

// Get session results
let results = voting_manager.get_session_results(session_id).await?;
println!("Yes votes: {}, No votes: {}", results.yes_votes, results.no_votes);

// End session (by creator or admin)
voting_manager.end_session(session_id).await?;
```

### Token Operations

```rust
use somnia_governance_engine::blockchain::TokenManager;

let token_manager = TokenManager::new(config).await?;

// Check token balance
let balance = token_manager.get_balance("0x123...").await?;
println!("Token balance: {}", balance);

// Check voting power (delegated tokens)
let voting_power = token_manager.get_voting_power("0x123...").await?;
println!("Voting power: {}", voting_power);

// Delegate tokens to self or another address
token_manager.delegate("0x123...").await?; // Self-delegation
token_manager.delegate("0x456...").await?; // Delegate to another address

// Transfer tokens
token_manager.transfer("0x456...", "1000000000000000000000").await?; // 1000 tokens
```

## Database Operations

### Proposal Database Operations

```rust
use somnia_governance_engine::{
    database::Database,
    models::{NewProposal, ProposalFilter},
};

let db = Database::connect(&config.database_url).await?;

// Create proposal record
let new_proposal = NewProposal {
    id: 1,
    ipfs_hash: "QmHash123".to_string(),
    creator: "0x123...".to_string(),
    proposal_type: "standard".to_string(),
    status: "active".to_string(),
};

db.create_proposal(&new_proposal).await?;

// Query proposals with filters
let filter = ProposalFilter {
    creator: Some("0x123...".to_string()),
    status: Some("active".to_string()),
    proposal_type: None,
    limit: Some(10),
    offset: Some(0),
};

let proposals = db.get_proposals_filtered(&filter).await?;

// Update proposal status
db.update_proposal_status(1, "succeeded").await?;

// Get proposal statistics
let stats = db.get_proposal_stats().await?;
println!("Total proposals: {}", stats.total);
println!("Active proposals: {}", stats.active);
```

### Vote Database Operations

```rust
use somnia_governance_engine::models::{Vote, VoteFilter};

// Store vote
let vote = Vote {
    id: 1,
    proposal_id: 1,
    voter: "0x123...".to_string(),
    choice: "for".to_string(),
    voting_power: "1000000000000000000000".to_string(),
    reasoning: Some("Good proposal".to_string()),
    created_at: chrono::Utc::now(),
};

db.create_vote(&vote).await?;

// Query votes
let vote_filter = VoteFilter {
    proposal_id: Some(1),
    voter: None,
    choice: Some("for".to_string()),
};

let votes = db.get_votes_filtered(&vote_filter).await?;

// Get vote summary for a proposal
let summary = db.get_vote_summary(1).await?;
println!("For: {}, Against: {}, Abstain: {}",
         summary.for_votes, summary.against_votes, summary.abstain_votes);
```

## API Usage

### Running the HTTP Server

```rust
use somnia_governance_engine::api::start_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let db = Database::connect(&config.database_url).await?;
    let contract_manager = ContractManager::new(config.clone()).await?;

    // Start the HTTP server
    start_server(config, db, contract_manager).await?;

    Ok(())
}
```

### Custom API Integration

```rust
use somnia_governance_engine::api::governance::{GovernanceAPI, CreateProposalRequest};

let governance_api = GovernanceAPI::new(contract_manager, db);

// Create proposal via API
let request = CreateProposalRequest {
    ipfs_hash: "QmProposal123".to_string(),
    duration: 86400,
    proposal_type: "standard".to_string(),
};

let proposal = governance_api.create_proposal(request).await?;

// Vote via API
let vote_request = VoteRequest {
    choice: "for".to_string(),
    reasoning: Some("Supporting this".to_string()),
    signature: "0x...".to_string(),
};

governance_api.vote(proposal.id, vote_request).await?;
```

## Event Monitoring

### Setting Up Event Monitoring

```rust
use somnia_governance_engine::{
    blockchain::EventMonitor,
    models::GovernanceEvent,
};

let mut event_monitor = EventMonitor::new(contract_manager.clone());

// Start monitoring in background
tokio::spawn(async move {
    event_monitor.start_monitoring().await.unwrap();
});

// Listen for events
while let Some(event) = event_monitor.next_event().await {
    match event {
        GovernanceEvent::ProposalCreated { id, creator, ipfs_hash, .. } => {
            println!("New proposal {} created by {}: {}", id, creator, ipfs_hash);
            // Handle new proposal (send notifications, update UI, etc.)
        }

        GovernanceEvent::VoteCast { proposal_id, voter, choice, voting_power, .. } => {
            println!("Vote cast on proposal {}: {} voted {:?} with power {}",
                     proposal_id, voter, choice, voting_power);
            // Handle vote (update real-time vote counts, etc.)
        }

        GovernanceEvent::ProposalExecuted { id, .. } => {
            println!("Proposal {} has been executed", id);
            // Handle execution (trigger follow-up actions, etc.)
        }

        GovernanceEvent::SessionCreated { id, creator, question, .. } => {
            println!("New voting session {}: {} by {}", id, question, creator);
            // Handle new session
        }

        _ => {}
    }
}
```

### Custom Event Handlers

```rust
use somnia_governance_engine::blockchain::{EventHandler, EventMonitor};

struct CustomEventHandler {
    db: Database,
}

#[async_trait::async_trait]
impl EventHandler for CustomEventHandler {
    async fn handle_proposal_created(&self, event: ProposalCreatedEvent) -> anyhow::Result<()> {
        // Custom logic for proposal creation
        let proposal = NewProposal {
            id: event.id,
            ipfs_hash: event.ipfs_hash,
            creator: event.creator,
            proposal_type: "standard".to_string(),
            status: "active".to_string(),
        };

        self.db.create_proposal(&proposal).await?;

        // Send notifications, update cache, etc.
        Ok(())
    }

    async fn handle_vote_cast(&self, event: VoteCastEvent) -> anyhow::Result<()> {
        // Custom logic for vote handling
        let vote = Vote {
            id: event.id,
            proposal_id: event.proposal_id,
            voter: event.voter,
            choice: event.choice,
            voting_power: event.voting_power,
            reasoning: event.reasoning,
            created_at: chrono::Utc::now(),
        };

        self.db.create_vote(&vote).await?;
        Ok(())
    }
}

// Use custom handler
let handler = CustomEventHandler { db: db.clone() };
let mut event_monitor = EventMonitor::with_handler(contract_manager, Box::new(handler));
event_monitor.start_monitoring().await?;
```

## Error Handling

### Error Types

```rust
use somnia_governance_engine::errors::{
    GovernanceError,
    BlockchainError,
    DatabaseError,
    ApiError,
};

// Handle different error types
match governance_api.create_proposal(request).await {
    Ok(proposal) => println!("Proposal created: {}", proposal.id),
    Err(GovernanceError::Blockchain(BlockchainError::InsufficientVotingPower)) => {
        println!("User doesn't have enough tokens to create proposal");
    }
    Err(GovernanceError::Database(DatabaseError::ConnectionFailed)) => {
        println!("Database connection failed");
    }
    Err(GovernanceError::Api(ApiError::InvalidRequest(msg))) => {
        println!("Invalid request: {}", msg);
    }
    Err(e) => println!("Unexpected error: {}", e),
}
```

### Result Handling Patterns

```rust
use anyhow::Result;

// Basic error handling
async fn create_and_vote() -> Result<()> {
    let proposal_id = contract_manager.create_proposal(
        "QmHash",
        86400,
        ProposalType::Standard,
    ).await?;

    contract_manager.vote(
        proposal_id,
        VoteChoice::For,
        None,
    ).await?;

    Ok(())
}

// Advanced error handling with context
async fn advanced_governance_flow() -> Result<()> {
    let proposal_id = contract_manager.create_proposal(
        "QmHash",
        86400,
        ProposalType::Standard,
    ).await
    .context("Failed to create governance proposal")?;

    let proposal = contract_manager.get_proposal(proposal_id).await
        .context("Failed to fetch proposal details")?;

    if proposal.status == ProposalStatus::Active {
        contract_manager.vote(proposal_id, VoteChoice::For, None).await
            .context("Failed to cast vote on proposal")?;
    }

    Ok(())
}
```

## Examples

### Complete Integration Example

```rust
use somnia_governance_engine::{
    config::Config,
    blockchain::ContractManager,
    database::Database,
    api::governance::GovernanceAPI,
    models::*,
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize
    let config = Config::from_env()?;
    let db = Database::connect(&config.database_url).await?;
    let contract_manager = ContractManager::new(config.clone()).await?;
    let governance_api = GovernanceAPI::new(contract_manager.clone(), db.clone());

    // Create a comprehensive governance proposal
    let proposal_request = CreateProposalRequest {
        ipfs_hash: "QmDetailedProposal123".to_string(),
        duration: 172800, // 48 hours
        proposal_type: "standard".to_string(),
    };

    let proposal = governance_api.create_proposal(proposal_request).await?;
    println!("Created proposal: {}", proposal.id);

    // Simulate multiple votes
    let voters = vec![
        ("0x111...", VoteChoice::For, "This proposal looks good"),
        ("0x222...", VoteChoice::Against, "I disagree with this approach"),
        ("0x333...", VoteChoice::For, "Supporting this initiative"),
        ("0x444...", VoteChoice::Abstain, ""),
    ];

    for (voter, choice, reasoning) in voters {
        let vote_request = VoteRequest {
            choice: choice.to_string(),
            reasoning: if reasoning.is_empty() {
                None
            } else {
                Some(reasoning.to_string())
            },
            signature: "0x...".to_string(), // Would be real signature
        };

        match governance_api.vote(proposal.id, vote_request).await {
            Ok(_) => println!("Vote cast by {}: {:?}", voter, choice),
            Err(e) => println!("Vote failed for {}: {}", voter, e),
        }
    }

    // Monitor events for this proposal
    let mut event_monitor = EventMonitor::new(contract_manager.clone());
    tokio::spawn(async move {
        while let Some(event) = event_monitor.next_event().await {
            if let GovernanceEvent::VoteCast { proposal_id, voter, choice, .. } = event {
                if proposal_id == proposal.id {
                    println!("Real-time vote: {} voted {:?}", voter, choice);
                }
            }
        }
    });

    // Get final results
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    let final_proposal = governance_api.get_proposal(proposal.id).await?;

    println!("Final results:");
    println!("  For votes: {}", final_proposal.for_votes);
    println!("  Against votes: {}", final_proposal.against_votes);
    println!("  Status: {:?}", final_proposal.status);

    Ok(())
}
```

### Custom Governance Workflow

```rust
use somnia_governance_engine::*;

struct CustomGovernanceWorkflow {
    contract_manager: ContractManager,
    db: Database,
}

impl CustomGovernanceWorkflow {
    pub fn new(contract_manager: ContractManager, db: Database) -> Self {
        Self { contract_manager, db }
    }

    pub async fn create_community_proposal(
        &self,
        title: &str,
        description: &str,
        duration_hours: u64,
    ) -> Result<u64> {
        // Upload to IPFS (simulated)
        let ipfs_hash = self.upload_to_ipfs(title, description).await?;

        // Create proposal on-chain
        let proposal_id = self.contract_manager.create_proposal(
            &ipfs_hash,
            duration_hours * 3600,
            ProposalType::Standard,
        ).await?;

        // Store in database
        let new_proposal = NewProposal {
            id: proposal_id,
            ipfs_hash,
            creator: self.contract_manager.get_address(),
            proposal_type: "community".to_string(),
            status: "active".to_string(),
        };

        self.db.create_proposal(&new_proposal).await?;

        // Notify community (webhook, email, etc.)
        self.notify_community(proposal_id, title).await?;

        Ok(proposal_id)
    }

    async fn upload_to_ipfs(&self, title: &str, description: &str) -> Result<String> {
        // Implement IPFS upload
        Ok(format!("Qm{}", "hash123"))
    }

    async fn notify_community(&self, proposal_id: u64, title: &str) -> Result<()> {
        // Implement notification logic
        println!("📢 New proposal #{}: {}", proposal_id, title);
        Ok(())
    }
}
```

This comprehensive API guide provides everything needed to integrate the Somnia Governance Engine into your Rust applications, from basic setup to advanced custom workflows.