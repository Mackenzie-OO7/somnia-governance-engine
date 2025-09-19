# Custom Contract Integration

Learn how to integrate the Somnia Governance Engine with your existing smart contracts or build custom governance contracts.

## Overview

The Rust governance engine can work with **any** smart contracts that implement basic governance interfaces. You don't need to use our contracts - the engine is designed to be contract-agnostic.

## Integration Approaches

### Approach 1: Use Your Existing Contracts

If you already have governance contracts, you can point our engine to them:

```rust
use somnia_governance_engine::{
    blockchain::ContractManager,
    config::{Config, ContractAddresses},
};

let config = Config {
    rpc_url: "https://your-rpc".to_string(),
    chain_id: 1234,
    private_key: "0x...".to_string(),

    // Point to YOUR contract addresses
    contracts: ContractAddresses {
        governance_hub: "0xYourGovernanceContract".to_string(),
        governance_token: "0xYourTokenContract".to_string(),
        timelock: "0xYourTimelockContract".to_string(),
        simple_voting: "0xYourVotingContract".to_string(),
    },

    // Other config...
};

let contract_manager = ContractManager::new(config).await?;

// Now our Rust engine works with YOUR contracts
let proposal_id = contract_manager.create_proposal(
    "QmProposalHash",
    86400,
    ProposalType::Standard,
).await?;
```

### Approach 2: Extend Our Contracts

Inherit from our contracts and add your custom features:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./GovernanceHub.sol";

contract MyCustomGovernance is GovernanceHub {
    // Add your custom fields
    mapping(uint256 => CustomProposalData) public customData;

    struct CustomProposalData {
        string category;
        address[] stakeholders;
        uint256 budgetRequest;
        bool requiresAudit;
    }

    constructor(
        address _governanceToken,
        address payable _timelock,
        address _admin
    ) GovernanceHub(_governanceToken, _timelock, _admin) {}

    // Override or add custom functions
    function createProposalWithCustomData(
        string calldata ipfsHash,
        uint256 duration,
        ProposalType proposalType,
        CustomProposalData calldata customProposalData
    ) external returns (uint256 proposalId) {
        // Call parent function
        proposalId = createProposal(ipfsHash, duration, proposalType);

        // Store custom data
        customData[proposalId] = customProposalData;

        emit CustomProposalCreated(proposalId, customProposalData);
    }

    // Add custom voting logic
    function voteWithJustification(
        uint256 proposalId,
        VoteChoice choice,
        string calldata reasoning,
        bytes32[] calldata evidenceHashes
    ) external {
        // Call parent vote function
        vote(proposalId, choice, reasoning);

        // Store additional evidence
        emit EvidenceSubmitted(proposalId, msg.sender, evidenceHashes);
    }

    event CustomProposalCreated(uint256 indexed proposalId, CustomProposalData data);
    event EvidenceSubmitted(uint256 indexed proposalId, address indexed voter, bytes32[] evidence);
}
```

### Approach 3: Build Completely Custom Contracts

Create your own governance contracts that implement our expected interfaces:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract MyGovernanceSystem {
    // Implement the interfaces our Rust engine expects

    struct Proposal {
        uint256 id;
        address creator;
        string ipfsHash;
        uint256 startTime;
        uint256 endTime;
        uint256 forVotes;
        uint256 againstVotes;
        uint256 abstainVotes;
        bool executed;
        ProposalStatus status;
    }

    enum ProposalStatus { Active, Succeeded, Failed, Executed }

    mapping(uint256 => Proposal) public proposals;
    uint256 public proposalCount;

    // Events our Rust engine listens for
    event ProposalCreated(
        uint256 indexed id,
        address indexed creator,
        string ipfsHash,
        uint256 startTime,
        uint256 endTime
    );

    event VoteCast(
        uint256 indexed proposalId,
        address indexed voter,
        uint8 choice,
        uint256 power,
        string reasoning
    );

    event ProposalExecuted(uint256 indexed proposalId);

    // Implement functions our Rust engine calls
    function createProposal(
        string calldata ipfsHash,
        uint256 duration,
        uint8 proposalType
    ) external returns (uint256) {
        uint256 proposalId = proposalCount++;

        proposals[proposalId] = Proposal({
            id: proposalId,
            creator: msg.sender,
            ipfsHash: ipfsHash,
            startTime: block.timestamp,
            endTime: block.timestamp + duration,
            forVotes: 0,
            againstVotes: 0,
            abstainVotes: 0,
            executed: false,
            status: ProposalStatus.Active
        });

        emit ProposalCreated(
            proposalId,
            msg.sender,
            ipfsHash,
            block.timestamp,
            block.timestamp + duration
        );

        return proposalId;
    }

    function vote(
        uint256 proposalId,
        uint8 choice,
        string calldata reasoning
    ) external {
        Proposal storage proposal = proposals[proposalId];
        require(proposal.status == ProposalStatus.Active, "Proposal not active");
        require(block.timestamp <= proposal.endTime, "Voting ended");

        // Your custom voting logic here
        uint256 votingPower = getVotingPower(msg.sender);

        if (choice == 1) { // For
            proposal.forVotes += votingPower;
        } else if (choice == 0) { // Against
            proposal.againstVotes += votingPower;
        } else { // Abstain
            proposal.abstainVotes += votingPower;
        }

        emit VoteCast(proposalId, msg.sender, choice, votingPower, reasoning);
    }

    function executeProposal(uint256 proposalId) external {
        Proposal storage proposal = proposals[proposalId];
        require(block.timestamp > proposal.endTime, "Voting still active");
        require(!proposal.executed, "Already executed");

        // Your custom execution logic
        if (proposal.forVotes > proposal.againstVotes) {
            proposal.status = ProposalStatus.Succeeded;
            proposal.executed = true;

            // Execute the proposal actions
            _executeProposalActions(proposalId);

            emit ProposalExecuted(proposalId);
        } else {
            proposal.status = ProposalStatus.Failed;
        }
    }

    // Custom functions for your specific use case
    function getVotingPower(address voter) public view returns (uint256) {
        // Implement your voting power calculation
        // Could be token balance, NFT count, reputation, etc.
        return 1; // Simplified
    }

    function _executeProposalActions(uint256 proposalId) internal {
        // Implement your proposal execution logic
        // Could interact with other contracts, transfer funds, etc.
    }
}
```

## Configuring the Rust Engine for Custom Contracts

### 1. Create Custom Contract Adapter

```rust
use somnia_governance_engine::{
    blockchain::{ContractAdapter, ContractCall},
    models::{Proposal, Vote, ProposalType},
};
use ethers::prelude::*;

pub struct CustomContractAdapter {
    contract: Contract<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

#[async_trait::async_trait]
impl ContractAdapter for CustomContractAdapter {
    async fn create_proposal(
        &self,
        ipfs_hash: &str,
        duration: u64,
        proposal_type: ProposalType,
    ) -> Result<u64> {
        let tx = self.contract
            .method::<_, U256>("createProposal", (ipfs_hash, duration, proposal_type as u8))?
            .send()
            .await?
            .await?;

        // Extract proposal ID from logs
        let proposal_id = self.extract_proposal_id_from_logs(&tx.logs)?;
        Ok(proposal_id)
    }

    async fn vote(
        &self,
        proposal_id: u64,
        choice: VoteChoice,
        reasoning: Option<String>,
    ) -> Result<()> {
        let reasoning = reasoning.unwrap_or_default();

        self.contract
            .method::<_, ()>("vote", (proposal_id, choice as u8, reasoning))?
            .send()
            .await?
            .await?;

        Ok(())
    }

    async fn get_proposal(&self, proposal_id: u64) -> Result<Proposal> {
        let proposal_data: (U256, Address, String, U256, U256, U256, U256, U256, bool, u8) =
            self.contract
                .method("proposals", proposal_id)?
                .call()
                .await?;

        Ok(Proposal {
            id: proposal_data.0.as_u64(),
            creator: format!("{:?}", proposal_data.1),
            ipfs_hash: proposal_data.2,
            start_time: proposal_data.3.as_u64(),
            end_time: proposal_data.4.as_u64(),
            for_votes: proposal_data.5.to_string(),
            against_votes: proposal_data.6.to_string(),
            abstain_votes: proposal_data.7.to_string(),
            executed: proposal_data.8,
            status: ProposalStatus::from_u8(proposal_data.9),
        })
    }

    // Implement other required methods...
}

impl CustomContractAdapter {
    pub fn new(contract_address: &str, provider_url: &str, private_key: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(provider_url)?;
        let wallet: LocalWallet = private_key.parse()?;
        let client = SignerMiddleware::new(provider, wallet);

        // Load your contract ABI
        let abi = include_str!("../abis/MyGovernanceSystem.json");
        let contract = Contract::new(
            contract_address.parse::<Address>()?,
            serde_json::from_str::<Abi>(abi)?,
            Arc::new(client),
        );

        Ok(Self { contract })
    }

    fn extract_proposal_id_from_logs(&self, logs: &[Log]) -> Result<u64> {
        // Parse ProposalCreated event to extract proposal ID
        for log in logs {
            if let Ok(event) = self.contract.decode_event::<(U256, Address, String, U256, U256)>(
                "ProposalCreated",
                log.topics.clone(),
                log.data.clone(),
            ) {
                return Ok(event.0.as_u64());
            }
        }
        Err(anyhow::anyhow!("ProposalCreated event not found"))
    }
}
```

### 2. Use Custom Adapter

```rust
use somnia_governance_engine::{
    blockchain::ContractManager,
    config::Config,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Create custom adapter for your contracts
    let custom_adapter = CustomContractAdapter::new(
        "0xYourGovernanceContract",
        "https://your-rpc-url",
        "0xYourPrivateKey",
    )?;

    // Create contract manager with custom adapter
    let mut contract_manager = ContractManager::with_custom_adapter(
        Box::new(custom_adapter)
    ).await?;

    // Now use our APIs with your contracts
    let proposal_id = contract_manager.create_proposal(
        "QmYourProposal",
        86400,
        ProposalType::Standard,
    ).await?;

    println!("Created proposal {} on your custom contract", proposal_id);

    Ok(())
}
```

## Event Monitoring for Custom Contracts

Configure event monitoring for your custom contract events:

```rust
use somnia_governance_engine::blockchain::{EventMonitor, CustomEventFilter};

// Define custom event filters
let custom_filters = vec![
    CustomEventFilter {
        contract_address: "0xYourGovernanceContract".to_string(),
        event_signature: "ProposalCreated(uint256,address,string,uint256,uint256)".to_string(),
        handler: Box::new(|log| {
            // Parse your custom ProposalCreated event
            println!("Custom proposal created: {:?}", log);
        }),
    },
    CustomEventFilter {
        contract_address: "0xYourGovernanceContract".to_string(),
        event_signature: "VoteCast(uint256,address,uint8,uint256,string)".to_string(),
        handler: Box::new(|log| {
            // Parse your custom VoteCast event
            println!("Custom vote cast: {:?}", log);
        }),
    },
];

let mut event_monitor = EventMonitor::with_custom_filters(custom_filters);
event_monitor.start_monitoring().await?;
```

## Database Integration for Custom Data

Store custom proposal data alongside our standard schema:

```rust
use somnia_governance_engine::database::{Database, DatabaseAdapter};

// Extend database schema for custom data
#[derive(Debug, Serialize, Deserialize)]
struct CustomProposalData {
    proposal_id: u64,
    category: String,
    budget_request: String,
    stakeholders: Vec<String>,
    requires_audit: bool,
}

impl Database {
    pub async fn store_custom_proposal_data(
        &self,
        data: &CustomProposalData,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO custom_proposal_data
            (proposal_id, category, budget_request, stakeholders, requires_audit)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            data.proposal_id as i64,
            data.category,
            data.budget_request,
            &data.stakeholders,
            data.requires_audit,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_custom_proposal_data(
        &self,
        proposal_id: u64,
    ) -> Result<Option<CustomProposalData>> {
        let row = sqlx::query!(
            "SELECT * FROM custom_proposal_data WHERE proposal_id = $1",
            proposal_id as i64,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| CustomProposalData {
            proposal_id: r.proposal_id as u64,
            category: r.category,
            budget_request: r.budget_request,
            stakeholders: r.stakeholders,
            requires_audit: r.requires_audit,
        }))
    }
}
```

## Testing Custom Integrations

Create tests for your custom contract integration:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use somnia_governance_engine::testing::{TestEnvironment, MockProvider};

    #[tokio::test]
    async fn test_custom_contract_integration() {
        let test_env = TestEnvironment::new().await;

        // Deploy your custom contract
        let custom_contract_address = test_env.deploy_custom_contract(
            include_bytes!("../contracts/MyGovernanceSystem.sol"),
        ).await?;

        // Create adapter
        let adapter = CustomContractAdapter::new(
            &custom_contract_address,
            &test_env.rpc_url(),
            &test_env.private_key(),
        )?;

        // Test proposal creation
        let proposal_id = adapter.create_proposal(
            "QmTestProposal",
            3600,
            ProposalType::Standard,
        ).await?;

        assert_eq!(proposal_id, 0);

        // Test voting
        adapter.vote(
            proposal_id,
            VoteChoice::For,
            Some("Test vote".to_string()),
        ).await?;

        // Test proposal retrieval
        let proposal = adapter.get_proposal(proposal_id).await?;
        assert_eq!(proposal.id, proposal_id);
        assert!(!proposal.for_votes.is_empty());
    }

    #[tokio::test]
    async fn test_custom_event_monitoring() {
        let test_env = TestEnvironment::new().await;
        // Test custom event parsing...
    }
}
```

## Migration Strategies

### Migrating from Existing Governance

If you're migrating from an existing governance system:

```rust
pub struct GovernanceMigration {
    old_contract: OldGovernanceContract,
    new_contract: CustomContractAdapter,
    database: Database,
}

impl GovernanceMigration {
    pub async fn migrate_proposals(&self) -> Result<()> {
        // 1. Export data from old system
        let old_proposals = self.old_contract.get_all_proposals().await?;

        for old_proposal in old_proposals {
            // 2. Create equivalent proposal in new system
            let new_proposal_id = self.new_contract.create_proposal(
                &old_proposal.ipfs_hash,
                old_proposal.remaining_duration(),
                old_proposal.proposal_type,
            ).await?;

            // 3. Migrate vote history
            self.migrate_votes(&old_proposal, new_proposal_id).await?;

            // 4. Update database mappings
            self.database.store_migration_mapping(
                old_proposal.id,
                new_proposal_id,
            ).await?;
        }

        Ok(())
    }

    async fn migrate_votes(
        &self,
        old_proposal: &OldProposal,
        new_proposal_id: u64,
    ) -> Result<()> {
        let votes = self.old_contract.get_proposal_votes(old_proposal.id).await?;

        for vote in votes {
            // Recreate votes in new system
            self.new_contract.vote(
                new_proposal_id,
                vote.choice,
                vote.reasoning,
            ).await?;
        }

        Ok(())
    }
}
```

## Best Practices

### 1. Interface Compatibility

Ensure your contracts implement the expected interfaces:

```solidity
interface IGovernanceContract {
    function createProposal(string calldata ipfsHash, uint256 duration, uint8 proposalType) external returns (uint256);
    function vote(uint256 proposalId, uint8 choice, string calldata reasoning) external;
    function executeProposal(uint256 proposalId) external;
    function getProposal(uint256 proposalId) external view returns (ProposalData memory);
}
```

### 2. Event Standardization

Use consistent event signatures:

```solidity
event ProposalCreated(uint256 indexed id, address indexed creator, string ipfsHash, uint256 startTime, uint256 endTime);
event VoteCast(uint256 indexed proposalId, address indexed voter, uint8 choice, uint256 power, string reasoning);
event ProposalExecuted(uint256 indexed proposalId);
```

### 3. Error Handling

Implement proper error handling in your adapter:

```rust
impl CustomContractAdapter {
    async fn create_proposal(&self, ...) -> Result<u64> {
        match self.contract.method("createProposal", ...).send().await {
            Ok(tx) => {
                match tx.await {
                    Ok(receipt) => self.extract_proposal_id(&receipt),
                    Err(e) => Err(anyhow::anyhow!("Transaction failed: {}", e)),
                }
            }
            Err(e) => Err(anyhow::anyhow!("Failed to send transaction: {}", e)),
        }
    }
}
```

### 4. Gas Optimization

Optimize for gas efficiency in your custom contracts:

```solidity
// Use packed structs
struct PackedProposal {
    uint128 forVotes;     // Instead of uint256
    uint128 againstVotes; // Pack into single slot
    uint64 startTime;     // Sufficient for timestamps
    uint64 endTime;
    bool executed;
    ProposalStatus status; // enum fits in remaining space
}
```

This comprehensive guide covers all aspects of integrating custom contracts with the Somnia Governance Engine. The modular design allows you to leverage our robust Rust infrastructure while maintaining full control over your governance logic.