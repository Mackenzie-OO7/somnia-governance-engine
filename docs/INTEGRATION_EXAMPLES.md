# Integration Examples & Tutorials

This guide provides practical, step-by-step examples for integrating the Somnia Governance Engine into various types of projects.

## Table of Contents

- [Quick Start Examples](#quick-start-examples)
- [DeFi Protocol Integration](#defi-protocol-integration)
- [DAO Integration](#dao-integration)
- [Web3 Application Integration](#web3-application-integration)
- [CLI Tool Integration](#cli-tool-integration)
- [Microservice Integration](#microservice-integration)
- [Custom Workflows](#custom-workflows)

## Quick Start Examples

### 1. Simple Rust Application

Create a basic governance-enabled application:

```rust
// Cargo.toml
[package]
name = "my-governance-app"
version = "0.1.0"
edition = "2021"

[dependencies]
somnia-governance-engine = { path = "../backend" }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
clap = { version = "4.0", features = ["derive"] }

// src/main.rs
use anyhow::Result;
use clap::{Parser, Subcommand};
use somnia_governance_engine::{
    config::Config,
    blockchain::ContractManager,
    models::{ProposalType, VoteChoice},
};

#[derive(Parser)]
#[command(name = "governance-cli")]
#[command(about = "A simple governance CLI application")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new proposal
    CreateProposal {
        /// IPFS hash of proposal content
        #[arg(long)]
        ipfs_hash: String,
        /// Voting duration in hours
        #[arg(long, default_value = "24")]
        duration_hours: u64,
        /// Proposal type (standard, emergency, constitutional)
        #[arg(long, default_value = "standard")]
        proposal_type: String,
    },
    /// Vote on a proposal
    Vote {
        /// Proposal ID
        #[arg(long)]
        proposal_id: u64,
        /// Vote choice (for, against, abstain)
        #[arg(long)]
        choice: String,
        /// Optional reasoning
        #[arg(long)]
        reason: Option<String>,
    },
    /// List all proposals
    List,
    /// Get proposal details
    Details {
        /// Proposal ID
        #[arg(long)]
        proposal_id: u64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize governance engine
    let config = Config::from_env()?;
    let contract_manager = ContractManager::new(config).await?;

    match cli.command {
        Commands::CreateProposal { ipfs_hash, duration_hours, proposal_type } => {
            let proposal_type = match proposal_type.as_str() {
                "standard" => ProposalType::Standard,
                "emergency" => ProposalType::Emergency,
                "constitutional" => ProposalType::Constitutional,
                _ => return Err(anyhow::anyhow!("Invalid proposal type")),
            };

            let proposal_id = contract_manager.create_proposal(
                &ipfs_hash,
                duration_hours * 3600,
                proposal_type,
            ).await?;

            println!("✅ Proposal created with ID: {}", proposal_id);
        }

        Commands::Vote { proposal_id, choice, reason } => {
            let vote_choice = match choice.as_str() {
                "for" => VoteChoice::For,
                "against" => VoteChoice::Against,
                "abstain" => VoteChoice::Abstain,
                _ => return Err(anyhow::anyhow!("Invalid vote choice")),
            };

            contract_manager.vote(proposal_id, vote_choice, reason).await?;
            println!("✅ Vote cast successfully on proposal {}", proposal_id);
        }

        Commands::List => {
            let proposal_count = contract_manager.get_proposal_count().await?;
            println!("📋 Total proposals: {}", proposal_count);

            for i in 0..proposal_count {
                let proposal = contract_manager.get_proposal(i).await?;
                println!("  {}. {} - Status: {:?}", i, proposal.ipfs_hash, proposal.status);
            }
        }

        Commands::Details { proposal_id } => {
            let proposal = contract_manager.get_proposal(proposal_id).await?;
            println!("📋 Proposal Details:");
            println!("  ID: {}", proposal.id);
            println!("  IPFS Hash: {}", proposal.ipfs_hash);
            println!("  Creator: {}", proposal.creator);
            println!("  Status: {:?}", proposal.status);
            println!("  For Votes: {}", proposal.for_votes);
            println!("  Against Votes: {}", proposal.against_votes);
            println!("  Abstain Votes: {}", proposal.abstain_votes);
        }
    }

    Ok(())
}
```

**Usage**:
```bash
# Set up environment
export RPC_URL="https://somnia-testnet-rpc"
export PRIVATE_KEY="0x..."
export GOVERNANCE_HUB_ADDRESS="0x..."

# Create proposal
cargo run -- create-proposal --ipfs-hash "QmProposal123" --duration-hours 48

# Vote on proposal
cargo run -- vote --proposal-id 0 --choice for --reason "Good proposal"

# List proposals
cargo run -- list

# Get proposal details
cargo run -- details --proposal-id 0
```

### 2. Web Service Integration

Create a REST API wrapper:

```rust
// src/main.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use somnia_governance_engine::{
    config::Config,
    blockchain::ContractManager,
    models::{ProposalType, VoteChoice},
};

#[derive(Clone)]
struct AppState {
    contract_manager: Arc<ContractManager>,
}

#[derive(Deserialize)]
struct CreateProposalRequest {
    ipfs_hash: String,
    duration_hours: u64,
    proposal_type: String,
}

#[derive(Deserialize)]
struct VoteRequest {
    choice: String,
    reason: Option<String>,
}

#[derive(Serialize)]
struct ProposalResponse {
    id: u64,
    ipfs_hash: String,
    creator: String,
    status: String,
    for_votes: String,
    against_votes: String,
    abstain_votes: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize governance engine
    let config = Config::from_env()?;
    let contract_manager = Arc::new(ContractManager::new(config).await?);

    let state = AppState { contract_manager };

    // Build router
    let app = Router::new()
        .route("/proposals", post(create_proposal))
        .route("/proposals", get(list_proposals))
        .route("/proposals/:id", get(get_proposal))
        .route("/proposals/:id/vote", post(vote_on_proposal))
        .with_state(state);

    // Start server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Server running on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_proposal(
    State(state): State<AppState>,
    Json(request): Json<CreateProposalRequest>,
) -> Result<Json<ProposalResponse>, (StatusCode, Json<ErrorResponse>)> {
    let proposal_type = match request.proposal_type.as_str() {
        "standard" => ProposalType::Standard,
        "emergency" => ProposalType::Emergency,
        "constitutional" => ProposalType::Constitutional,
        _ => return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Invalid proposal type".to_string() }),
        )),
    };

    match state.contract_manager.create_proposal(
        &request.ipfs_hash,
        request.duration_hours * 3600,
        proposal_type,
    ).await {
        Ok(proposal_id) => {
            let proposal = state.contract_manager.get_proposal(proposal_id).await
                .map_err(|e| (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse { error: e.to_string() }),
                ))?;

            Ok(Json(ProposalResponse {
                id: proposal.id,
                ipfs_hash: proposal.ipfs_hash,
                creator: proposal.creator,
                status: format!("{:?}", proposal.status),
                for_votes: proposal.for_votes,
                against_votes: proposal.against_votes,
                abstain_votes: proposal.abstain_votes,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

async fn vote_on_proposal(
    State(state): State<AppState>,
    Path(proposal_id): Path<u64>,
    Json(request): Json<VoteRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let vote_choice = match request.choice.as_str() {
        "for" => VoteChoice::For,
        "against" => VoteChoice::Against,
        "abstain" => VoteChoice::Abstain,
        _ => return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: "Invalid vote choice".to_string() }),
        )),
    };

    match state.contract_manager.vote(proposal_id, vote_choice, request.reason).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

async fn get_proposal(
    State(state): State<AppState>,
    Path(proposal_id): Path<u64>,
) -> Result<Json<ProposalResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.contract_manager.get_proposal(proposal_id).await {
        Ok(proposal) => Ok(Json(ProposalResponse {
            id: proposal.id,
            ipfs_hash: proposal.ipfs_hash,
            creator: proposal.creator,
            status: format!("{:?}", proposal.status),
            for_votes: proposal.for_votes,
            against_votes: proposal.against_votes,
            abstain_votes: proposal.abstain_votes,
        })),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

async fn list_proposals(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProposalResponse>>, (StatusCode, Json<ErrorResponse>)> {
    match state.contract_manager.get_proposal_count().await {
        Ok(count) => {
            let mut proposals = Vec::new();
            for i in 0..count {
                if let Ok(proposal) = state.contract_manager.get_proposal(i).await {
                    proposals.push(ProposalResponse {
                        id: proposal.id,
                        ipfs_hash: proposal.ipfs_hash,
                        creator: proposal.creator,
                        status: format!("{:?}", proposal.status),
                        for_votes: proposal.for_votes,
                        against_votes: proposal.against_votes,
                        abstain_votes: proposal.abstain_votes,
                    });
                }
            }
            Ok(Json(proposals))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}
```

**API Usage**:
```bash
# Create proposal
curl -X POST http://localhost:3000/proposals \
  -H "Content-Type: application/json" \
  -d '{"ipfs_hash": "QmTest123", "duration_hours": 24, "proposal_type": "standard"}'

# Vote on proposal
curl -X POST http://localhost:3000/proposals/0/vote \
  -H "Content-Type: application/json" \
  -d '{"choice": "for", "reason": "Good proposal"}'

# Get proposal
curl http://localhost:3000/proposals/0

# List proposals
curl http://localhost:3000/proposals
```

## DeFi Protocol Integration

### Yield Farming Governance

Integrate governance into a yield farming protocol:

```rust
// src/defi_governance.rs
use somnia_governance_engine::{
    blockchain::ContractManager,
    models::{ProposalType, VoteChoice},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FarmingPool {
    pub token_pair: String,
    pub reward_rate: u64,      // Tokens per block
    pub allocation_points: u64, // Weight in the total allocation
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub title: String,
    pub description: String,
    pub actions: Vec<ProposalAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalAction {
    AddPool { token_pair: String, allocation_points: u64, reward_rate: u64 },
    UpdatePoolAllocation { pool_id: u64, new_allocation: u64 },
    UpdateRewardRate { pool_id: u64, new_rate: u64 },
    PausePool { pool_id: u64 },
    UpdateTreasuryAllocation { percentage: u64 },
    EmergencyWithdraw { token: String, amount: String },
}

pub struct DeFiGovernance {
    contract_manager: ContractManager,
    pools: HashMap<u64, FarmingPool>,
    treasury_allocation: u64, // Percentage of rewards going to treasury
}

impl DeFiGovernance {
    pub fn new(contract_manager: ContractManager) -> Self {
        Self {
            contract_manager,
            pools: HashMap::new(),
            treasury_allocation: 10, // 10% to treasury by default
        }
    }

    /// Create a governance proposal for protocol changes
    pub async fn create_protocol_proposal(
        &self,
        proposal: GovernanceProposal,
        voting_duration_hours: u64,
    ) -> anyhow::Result<u64> {
        // Upload proposal to IPFS (simplified)
        let ipfs_hash = self.upload_proposal_to_ipfs(&proposal).await?;

        // Determine proposal type based on actions
        let proposal_type = self.classify_proposal_type(&proposal.actions);

        // Create on-chain proposal
        let proposal_id = self.contract_manager.create_proposal(
            &ipfs_hash,
            voting_duration_hours * 3600,
            proposal_type,
        ).await?;

        println!("🗳️  Created DeFi governance proposal: {}", proposal.title);
        println!("📋 Proposal ID: {}", proposal_id);
        println!("🔗 IPFS: {}", ipfs_hash);

        Ok(proposal_id)
    }

    /// Vote on a protocol proposal with farming power
    pub async fn vote_with_farming_power(
        &self,
        proposal_id: u64,
        choice: VoteChoice,
        reasoning: &str,
    ) -> anyhow::Result<()> {
        // Calculate user's farming power (LP tokens + staked tokens)
        let farming_power = self.calculate_farming_power().await?;

        println!("🌾 Voting with farming power: {} tokens", farming_power);

        self.contract_manager.vote(
            proposal_id,
            choice,
            Some(format!("Farming perspective: {}", reasoning)),
        ).await?;

        Ok(())
    }

    /// Execute approved proposal actions
    pub async fn execute_proposal_actions(
        &self,
        proposal_id: u64,
        actions: Vec<ProposalAction>,
    ) -> anyhow::Result<()> {
        // First execute the on-chain proposal
        self.contract_manager.execute_proposal(proposal_id).await?;

        // Then execute the protocol-specific actions
        for action in actions {
            match action {
                ProposalAction::AddPool { token_pair, allocation_points, reward_rate } => {
                    self.add_farming_pool(token_pair, allocation_points, reward_rate).await?;
                }
                ProposalAction::UpdatePoolAllocation { pool_id, new_allocation } => {
                    self.update_pool_allocation(pool_id, new_allocation).await?;
                }
                ProposalAction::UpdateRewardRate { pool_id, new_rate } => {
                    self.update_reward_rate(pool_id, new_rate).await?;
                }
                ProposalAction::PausePool { pool_id } => {
                    self.pause_pool(pool_id).await?;
                }
                ProposalAction::UpdateTreasuryAllocation { percentage } => {
                    self.update_treasury_allocation(percentage).await?;
                }
                ProposalAction::EmergencyWithdraw { token, amount } => {
                    self.emergency_withdraw(token, amount).await?;
                }
            }
        }

        println!("✅ All proposal actions executed successfully");
        Ok(())
    }

    /// Monitor governance events and react to protocol changes
    pub async fn start_governance_monitoring(&self) -> anyhow::Result<()> {
        use somnia_governance_engine::blockchain::EventMonitor;
        use somnia_governance_engine::models::GovernanceEvent;

        let mut event_monitor = EventMonitor::new(self.contract_manager.clone());

        tokio::spawn(async move {
            while let Some(event) = event_monitor.next_event().await {
                match event {
                    GovernanceEvent::ProposalCreated { id, creator, .. } => {
                        println!("🆕 New DeFi proposal #{} by {}", id, creator);
                        // Notify farmers, update UI, etc.
                    }

                    GovernanceEvent::VoteCast { proposal_id, voter, choice, voting_power, .. } => {
                        println!("🗳️  Vote on proposal #{}: {} voted {:?} with {} power",
                                proposal_id, voter, choice, voting_power);
                        // Update real-time vote tracking
                    }

                    GovernanceEvent::ProposalExecuted { id, .. } => {
                        println!("⚡ Proposal #{} executed - Protocol changes active!", id);
                        // Trigger protocol updates, restart farming contracts, etc.
                    }

                    _ => {}
                }
            }
        });

        Ok(())
    }

    // Private helper methods
    async fn upload_proposal_to_ipfs(&self, proposal: &GovernanceProposal) -> anyhow::Result<String> {
        // In real implementation, upload to IPFS
        let content = serde_json::to_string(proposal)?;
        // Simulate IPFS upload
        Ok(format!("QmProposal{}", content.len()))
    }

    fn classify_proposal_type(&self, actions: &[ProposalAction]) -> ProposalType {
        // Classify based on action severity
        for action in actions {
            match action {
                ProposalAction::EmergencyWithdraw { .. } => return ProposalType::Emergency,
                ProposalAction::UpdateTreasuryAllocation { percentage } if *percentage > 50 => {
                    return ProposalType::Constitutional;
                }
                _ => {}
            }
        }
        ProposalType::Standard
    }

    async fn calculate_farming_power(&self) -> anyhow::Result<u64> {
        // Calculate based on LP tokens, staked tokens, etc.
        // This would integrate with your farming contracts
        Ok(1000000) // Simplified
    }

    async fn add_farming_pool(&self, token_pair: String, allocation_points: u64, reward_rate: u64) -> anyhow::Result<()> {
        println!("🏊 Adding new farming pool: {} (allocation: {}, rate: {})",
                token_pair, allocation_points, reward_rate);
        // Implement pool addition logic
        Ok(())
    }

    async fn update_pool_allocation(&self, pool_id: u64, new_allocation: u64) -> anyhow::Result<()> {
        println!("📊 Updating pool {} allocation to {}", pool_id, new_allocation);
        Ok(())
    }

    async fn update_reward_rate(&self, pool_id: u64, new_rate: u64) -> anyhow::Result<()> {
        println!("💰 Updating pool {} reward rate to {}", pool_id, new_rate);
        Ok(())
    }

    async fn pause_pool(&self, pool_id: u64) -> anyhow::Result<()> {
        println!("⏸️  Pausing pool {}", pool_id);
        Ok(())
    }

    async fn update_treasury_allocation(&self, percentage: u64) -> anyhow::Result<()> {
        println!("🏛️  Updating treasury allocation to {}%", percentage);
        Ok(())
    }

    async fn emergency_withdraw(&self, token: String, amount: String) -> anyhow::Result<()> {
        println!("🚨 Emergency withdraw: {} {}", amount, token);
        Ok(())
    }
}

// Usage example
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use somnia_governance_engine::config::Config;

    let config = Config::from_env()?;
    let contract_manager = ContractManager::new(config).await?;
    let defi_governance = DeFiGovernance::new(contract_manager);

    // Create a proposal to add a new farming pool
    let proposal = GovernanceProposal {
        title: "Add USDC/ETH Farming Pool".to_string(),
        description: "Proposal to add a new USDC/ETH liquidity farming pool with 15% allocation".to_string(),
        actions: vec![
            ProposalAction::AddPool {
                token_pair: "USDC/ETH".to_string(),
                allocation_points: 150,
                reward_rate: 100,
            }
        ],
    };

    let proposal_id = defi_governance.create_protocol_proposal(proposal, 48).await?;

    // Vote on the proposal
    defi_governance.vote_with_farming_power(
        proposal_id,
        VoteChoice::For,
        "This pool will provide good liquidity for the protocol"
    ).await?;

    // Start monitoring for governance events
    defi_governance.start_governance_monitoring().await?;

    Ok(())
}
```

## DAO Integration

### Community DAO with Treasury Management

```rust
// src/dao_treasury.rs
use somnia_governance_engine::{
    blockchain::ContractManager,
    models::{ProposalType, VoteChoice},
    database::Database,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryProposal {
    pub title: String,
    pub description: String,
    pub recipient: String,
    pub amount: String,
    pub token: String,
    pub category: ProposalCategory,
    pub milestones: Vec<Milestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalCategory {
    Development,
    Marketing,
    Operations,
    Community,
    Security,
    Research,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub description: String,
    pub amount: String,
    pub deadline: chrono::DateTime<chrono::Utc>,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub address: String,
    pub role: MemberRole,
    pub reputation: u64,
    pub proposals_created: u64,
    pub votes_cast: u64,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemberRole {
    Member,
    Contributor,
    CoreTeam,
    Council,
}

pub struct CommunityDAO {
    contract_manager: ContractManager,
    database: Database,
    members: HashMap<String, Member>,
    treasury_balance: HashMap<String, String>, // token -> balance
}

impl CommunityDAO {
    pub fn new(contract_manager: ContractManager, database: Database) -> Self {
        Self {
            contract_manager,
            database,
            members: HashMap::new(),
            treasury_balance: HashMap::new(),
        }
    }

    /// Create a treasury spending proposal
    pub async fn create_treasury_proposal(
        &self,
        proposal: TreasuryProposal,
    ) -> anyhow::Result<u64> {
        // Validate proposal
        self.validate_treasury_proposal(&proposal).await?;

        // Calculate voting duration based on amount
        let voting_duration = self.calculate_voting_duration(&proposal.amount)?;

        // Upload to IPFS
        let ipfs_hash = self.upload_proposal_to_ipfs(&proposal).await?;

        // Create on-chain proposal
        let proposal_id = self.contract_manager.create_proposal(
            &ipfs_hash,
            voting_duration,
            ProposalType::Standard,
        ).await?;

        // Store in database
        self.database.create_treasury_proposal(&proposal, proposal_id).await?;

        // Notify community
        self.notify_community_proposal(&proposal, proposal_id).await?;

        println!("💰 Treasury proposal created: {}", proposal.title);
        println!("💵 Amount: {} {}", proposal.amount, proposal.token);
        println!("👤 Recipient: {}", proposal.recipient);

        Ok(proposal_id)
    }

    /// Vote on treasury proposal with reputation weighting
    pub async fn vote_with_reputation(
        &self,
        proposal_id: u64,
        voter: &str,
        choice: VoteChoice,
        reasoning: &str,
    ) -> anyhow::Result<()> {
        // Get member reputation
        let member = self.members.get(voter)
            .ok_or_else(|| anyhow::anyhow!("Member not found"))?;

        // Calculate reputation-weighted voting power
        let base_voting_power = self.contract_manager.get_voting_power(voter).await?;
        let reputation_multiplier = self.calculate_reputation_multiplier(member.reputation);
        let total_voting_power = base_voting_power * reputation_multiplier / 100;

        println!("🗳️  {} voting with {} reputation ({}x multiplier)",
                voter, member.reputation, reputation_multiplier);

        // Cast vote
        self.contract_manager.vote(
            proposal_id,
            choice,
            Some(format!("Reputation-weighted vote: {}", reasoning)),
        ).await?;

        // Update member stats
        self.update_member_voting_stats(voter).await?;

        Ok(())
    }

    /// Execute treasury proposal and distribute funds
    pub async fn execute_treasury_proposal(
        &self,
        proposal_id: u64,
        proposal: TreasuryProposal,
    ) -> anyhow::Result<()> {
        // Execute on-chain proposal
        self.contract_manager.execute_proposal(proposal_id).await?;

        // Process treasury transfer
        if proposal.milestones.is_empty() {
            // Single payment
            self.transfer_treasury_funds(
                &proposal.recipient,
                &proposal.amount,
                &proposal.token,
            ).await?;
        } else {
            // Milestone-based payment - only first milestone
            let first_milestone = &proposal.milestones[0];
            self.transfer_treasury_funds(
                &proposal.recipient,
                &first_milestone.amount,
                &proposal.token,
            ).await?;

            // Schedule remaining milestones
            self.schedule_milestone_payments(proposal_id, &proposal.milestones[1..]).await?;
        }

        // Update treasury balance
        self.update_treasury_balance(&proposal.token, &proposal.amount).await?;

        // Grant reputation to proposal creator
        self.grant_reputation(&proposal.recipient, 10).await?;

        println!("✅ Treasury proposal executed: {} {} sent to {}",
                proposal.amount, proposal.token, proposal.recipient);

        Ok(())
    }

    /// Member onboarding and role management
    pub async fn onboard_member(
        &mut self,
        address: &str,
        role: MemberRole,
    ) -> anyhow::Result<()> {
        let member = Member {
            address: address.to_string(),
            role,
            reputation: 1,
            proposals_created: 0,
            votes_cast: 0,
            joined_at: chrono::Utc::now(),
        };

        self.members.insert(address.to_string(), member.clone());
        self.database.create_member(&member).await?;

        // Grant initial governance tokens
        let initial_tokens = match member.role {
            MemberRole::Member => "1000",
            MemberRole::Contributor => "5000",
            MemberRole::CoreTeam => "10000",
            MemberRole::Council => "25000",
        };

        self.contract_manager.mint_tokens(address, initial_tokens).await?;

        println!("🎉 New member onboarded: {} as {:?}", address, member.role);
        println!("🎁 Granted {} governance tokens", initial_tokens);

        Ok(())
    }

    /// Community health monitoring
    pub async fn generate_community_report(&self) -> anyhow::Result<CommunityReport> {
        let total_members = self.members.len();
        let active_proposals = self.contract_manager.get_active_proposal_count().await?;
        let treasury_value = self.calculate_total_treasury_value().await?;

        let member_activity = self.calculate_member_activity().await?;
        let proposal_success_rate = self.calculate_proposal_success_rate().await?;

        let report = CommunityReport {
            total_members,
            active_proposals,
            treasury_value,
            member_activity,
            proposal_success_rate,
            top_contributors: self.get_top_contributors(5).await?,
        };

        println!("📊 Community Health Report:");
        println!("  👥 Total Members: {}", report.total_members);
        println!("  🗳️  Active Proposals: {}", report.active_proposals);
        println!("  💰 Treasury Value: ${}", report.treasury_value);
        println!("  📈 Member Activity: {:.1}%", report.member_activity);
        println!("  ✅ Proposal Success Rate: {:.1}%", report.proposal_success_rate);

        Ok(report)
    }

    // Helper methods
    async fn validate_treasury_proposal(&self, proposal: &TreasuryProposal) -> anyhow::Result<()> {
        // Check treasury balance
        let current_balance = self.treasury_balance.get(&proposal.token)
            .ok_or_else(|| anyhow::anyhow!("Token not in treasury"))?;

        let requested_amount: u64 = proposal.amount.parse()?;
        let available_amount: u64 = current_balance.parse()?;

        if requested_amount > available_amount {
            return Err(anyhow::anyhow!("Insufficient treasury funds"));
        }

        // Validate recipient
        if proposal.recipient.len() != 42 || !proposal.recipient.starts_with("0x") {
            return Err(anyhow::anyhow!("Invalid recipient address"));
        }

        Ok(())
    }

    fn calculate_voting_duration(&self, amount: &str) -> anyhow::Result<u64> {
        let amount: u64 = amount.parse()?;

        // Scale voting duration based on amount
        let duration = match amount {
            0..=1000 => 24 * 3600,      // 24 hours for small amounts
            1001..=10000 => 48 * 3600,  // 48 hours for medium amounts
            _ => 72 * 3600,             // 72 hours for large amounts
        };

        Ok(duration)
    }

    fn calculate_reputation_multiplier(&self, reputation: u64) -> u64 {
        // Reputation multiplier: 100% + (reputation * 2)%, capped at 200%
        std::cmp::min(100 + (reputation * 2), 200)
    }

    async fn upload_proposal_to_ipfs(&self, proposal: &TreasuryProposal) -> anyhow::Result<String> {
        let content = serde_json::to_string(proposal)?;
        // Simulate IPFS upload
        Ok(format!("QmTreasury{}", content.len()))
    }

    async fn notify_community_proposal(&self, proposal: &TreasuryProposal, proposal_id: u64) -> anyhow::Result<()> {
        println!("📢 New treasury proposal notification sent to community");
        println!("  Title: {}", proposal.title);
        println!("  ID: {}", proposal_id);
        // Implement actual notification logic (Discord, email, etc.)
        Ok(())
    }

    async fn transfer_treasury_funds(&self, recipient: &str, amount: &str, token: &str) -> anyhow::Result<()> {
        println!("💸 Transferring {} {} to {}", amount, token, recipient);
        // Implement actual treasury transfer
        Ok(())
    }

    async fn schedule_milestone_payments(&self, proposal_id: u64, milestones: &[Milestone]) -> anyhow::Result<()> {
        println!("📅 Scheduling {} milestone payments for proposal {}", milestones.len(), proposal_id);
        // Implement milestone scheduling
        Ok(())
    }

    async fn update_treasury_balance(&self, token: &str, amount: &str) -> anyhow::Result<()> {
        println!("📊 Updating treasury balance for {}: -{}", token, amount);
        Ok(())
    }

    async fn grant_reputation(&self, address: &str, amount: u64) -> anyhow::Result<()> {
        println!("⭐ Granting {} reputation to {}", amount, address);
        Ok(())
    }

    async fn update_member_voting_stats(&self, voter: &str) -> anyhow::Result<()> {
        println!("📈 Updating voting stats for {}", voter);
        Ok(())
    }

    async fn calculate_member_activity(&self) -> anyhow::Result<f64> {
        Ok(85.5) // Simplified
    }

    async fn calculate_proposal_success_rate(&self) -> anyhow::Result<f64> {
        Ok(72.3) // Simplified
    }

    async fn calculate_total_treasury_value(&self) -> anyhow::Result<f64> {
        Ok(450000.0) // Simplified
    }

    async fn get_top_contributors(&self, limit: usize) -> anyhow::Result<Vec<String>> {
        // Return top contributors by reputation
        let mut contributors: Vec<_> = self.members.values().collect();
        contributors.sort_by(|a, b| b.reputation.cmp(&a.reputation));

        Ok(contributors.into_iter()
           .take(limit)
           .map(|m| m.address.clone())
           .collect())
    }
}

#[derive(Debug, Serialize)]
pub struct CommunityReport {
    pub total_members: usize,
    pub active_proposals: u64,
    pub treasury_value: f64,
    pub member_activity: f64,
    pub proposal_success_rate: f64,
    pub top_contributors: Vec<String>,
}
```

This comprehensive integration guide provides practical examples for implementing governance in real-world scenarios, from simple CLI tools to complex DeFi protocols and DAOs. Each example includes complete code and usage instructions.