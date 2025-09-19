use crate::utils::errors::{GovernanceError, Result};
use async_trait::async_trait;
use ethers::prelude::*;
use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Smart contract interfaces (will be auto-generated from ABIs later)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalData {
    pub id: u64,
    pub ipfs_hash: String,
    pub proposer: Address,
    pub start_time: U256,
    pub end_time: U256,
    pub proposal_type: u8,
    pub status: ProposalStatus,
    pub total_votes: U256,
    pub yes_votes: U256,
    pub no_votes: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Pending = 0,
    Active = 1,
    Passed = 2,
    Rejected = 3,
    Executed = 4,
}

impl From<u8> for ProposalStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => ProposalStatus::Pending,
            1 => ProposalStatus::Active,
            2 => ProposalStatus::Passed,
            3 => ProposalStatus::Rejected,
            4 => ProposalStatus::Executed,
            _ => ProposalStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteData {
    pub proposal_id: u64,
    pub voter: Address,
    pub choice: u8, // 0=no, 1=yes, 2=abstain
    pub power: U256,
    pub timestamp: U256,
    pub ipfs_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteCastEvent {
    pub proposal_id: u64,
    pub voter: Address,
    pub choice: u8,
    pub power: U256,
    pub timestamp: U256,
    pub ipfs_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalCreatedEvent {
    pub proposal_id: u64,
    pub proposer: Address,
    pub ipfs_hash: String,
    pub start_time: U256,
    pub end_time: U256,
    pub proposal_type: u8,
}

// Trait definitions for contract interactions
#[async_trait]
pub trait GovernanceHubContract {
    async fn create_proposal(
        &self,
        ipfs_hash: String,
        voting_duration: U256,
        proposal_type: u8,
    ) -> Result<TransactionReceipt>;

    async fn get_proposal(&self, proposal_id: u64) -> Result<ProposalData>;
    async fn get_proposal_count(&self) -> Result<u64>;
    async fn get_proposals_by_status(&self, status: ProposalStatus) -> Result<Vec<ProposalData>>;
    async fn get_user_voting_power(&self, user: Address) -> Result<U256>;
}

#[async_trait]
pub trait SimpleVotingContract {
    async fn cast_vote(
        &self,
        proposal_id: u64,
        choice: u8,
        ipfs_hash: Option<String>,
    ) -> Result<TransactionReceipt>;

    async fn get_vote(&self, proposal_id: u64, voter: Address) -> Result<Option<VoteData>>;
    async fn get_proposal_votes(&self, proposal_id: u64) -> Result<Vec<VoteData>>;
    async fn has_voted(&self, proposal_id: u64, voter: Address) -> Result<bool>;
    async fn get_vote_tally(&self, proposal_id: u64) -> Result<(U256, U256, U256)>; // (yes, no, abstain)
}

// Mock implementations for testing (will be replaced with real contract calls)
pub struct MockGovernanceHub {
    pub proposals: std::sync::Mutex<std::collections::HashMap<u64, ProposalData>>,
    pub next_id: std::sync::Mutex<u64>,
}

impl MockGovernanceHub {
    pub fn new() -> Self {
        Self {
            proposals: std::sync::Mutex::new(std::collections::HashMap::new()),
            next_id: std::sync::Mutex::new(1),
        }
    }
}

#[async_trait]
impl GovernanceHubContract for MockGovernanceHub {
    async fn create_proposal(
        &self,
        ipfs_hash: String,
        voting_duration: U256,
        proposal_type: u8,
    ) -> Result<TransactionReceipt> {
        let mut next_id = self.next_id.lock().unwrap();
        let proposal_id = *next_id;
        *next_id += 1;

        let now = U256::from(chrono::Utc::now().timestamp());
        let proposal = ProposalData {
            id: proposal_id,
            ipfs_hash,
            proposer: Address::zero(), // Would be msg.sender in real contract
            start_time: now,
            end_time: now + voting_duration,
            proposal_type,
            status: ProposalStatus::Active,
            total_votes: U256::zero(),
            yes_votes: U256::zero(),
            no_votes: U256::zero(),
        };

        let mut proposals = self.proposals.lock().unwrap();
        proposals.insert(proposal_id, proposal);

        // Mock transaction receipt
        Ok(TransactionReceipt {
            transaction_hash: H256::random(),
            transaction_index: U64::from(0),
            block_hash: Some(H256::random()),
            block_number: Some(U64::from(1000)),
            from: Address::zero(),
            to: Some(Address::random()),
            cumulative_gas_used: U256::from(100000),
            gas_used: Some(U256::from(50000)),
            contract_address: None,
            logs: vec![],
            status: Some(U64::from(1)),
            root: None,
            logs_bloom: Bloom::default(),
            transaction_type: Some(U64::from(2)),
            effective_gas_price: Some(U256::from(20_000_000_000u64)),
            other: Default::default(),
        })
    }

    async fn get_proposal(&self, proposal_id: u64) -> Result<ProposalData> {
        let proposals = self.proposals.lock().unwrap();
        proposals
            .get(&proposal_id)
            .cloned()
            .ok_or_else(|| GovernanceError::ProposalNotFound { proposal_id })
    }

    async fn get_proposal_count(&self) -> Result<u64> {
        let next_id = self.next_id.lock().unwrap();
        Ok(*next_id - 1)
    }

    async fn get_proposals_by_status(&self, status: ProposalStatus) -> Result<Vec<ProposalData>> {
        let proposals = self.proposals.lock().unwrap();
        let filtered: Vec<ProposalData> = proposals
            .values()
            .filter(|p| std::mem::discriminant(&p.status) == std::mem::discriminant(&status))
            .cloned()
            .collect();
        Ok(filtered)
    }

    async fn get_user_voting_power(&self, _user: Address) -> Result<U256> {
        // Mock voting power
        Ok(U256::from(1000))
    }
}

pub struct MockSimpleVoting {
    pub votes: std::sync::Mutex<std::collections::HashMap<(u64, Address), VoteData>>,
}

impl MockSimpleVoting {
    pub fn new() -> Self {
        Self {
            votes: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait]
impl SimpleVotingContract for MockSimpleVoting {
    async fn cast_vote(
        &self,
        proposal_id: u64,
        choice: u8,
        ipfs_hash: Option<String>,
    ) -> Result<TransactionReceipt> {
        let voter = Address::random(); // Would be msg.sender in real contract
        let vote_data = VoteData {
            proposal_id,
            voter,
            choice,
            power: U256::from(1000), // Mock voting power
            timestamp: U256::from(chrono::Utc::now().timestamp()),
            ipfs_hash,
        };

        let mut votes = self.votes.lock().unwrap();
        votes.insert((proposal_id, voter), vote_data);

        // Mock transaction receipt
        Ok(TransactionReceipt {
            transaction_hash: H256::random(),
            transaction_index: U64::from(0),
            block_hash: Some(H256::random()),
            block_number: Some(U64::from(1001)),
            from: voter,
            to: Some(Address::random()),
            cumulative_gas_used: U256::from(80000),
            gas_used: Some(U256::from(40000)),
            contract_address: None,
            logs: vec![],
            status: Some(U64::from(1)),
            root: None,
            logs_bloom: Bloom::default(),
            transaction_type: Some(U64::from(2)),
            effective_gas_price: Some(U256::from(20_000_000_000u64)),
            other: Default::default(),
        })
    }

    async fn get_vote(&self, proposal_id: u64, voter: Address) -> Result<Option<VoteData>> {
        let votes = self.votes.lock().unwrap();
        Ok(votes.get(&(proposal_id, voter)).cloned())
    }

    async fn get_proposal_votes(&self, proposal_id: u64) -> Result<Vec<VoteData>> {
        let votes = self.votes.lock().unwrap();
        let proposal_votes: Vec<VoteData> = votes
            .values()
            .filter(|v| v.proposal_id == proposal_id)
            .cloned()
            .collect();
        Ok(proposal_votes)
    }

    async fn has_voted(&self, proposal_id: u64, voter: Address) -> Result<bool> {
        let votes = self.votes.lock().unwrap();
        Ok(votes.contains_key(&(proposal_id, voter)))
    }

    async fn get_vote_tally(&self, proposal_id: u64) -> Result<(U256, U256, U256)> {
        let votes = self.votes.lock().unwrap();
        let mut yes_votes = U256::zero();
        let mut no_votes = U256::zero();
        let mut abstain_votes = U256::zero();

        for vote in votes.values().filter(|v| v.proposal_id == proposal_id) {
            match vote.choice {
                0 => no_votes += vote.power,
                1 => yes_votes += vote.power,
                2 => abstain_votes += vote.power,
                _ => {}
            }
        }

        Ok((yes_votes, no_votes, abstain_votes))
    }
}

// Contract factory for creating contract instances
pub struct ContractFactory {
    pub governance_hub: Option<Address>,
    pub simple_voting: Option<Address>,
}

impl ContractFactory {
    pub fn new() -> Self {
        Self {
            governance_hub: None,
            simple_voting: None,
        }
    }

    pub fn with_addresses(governance_hub: Address, simple_voting: Address) -> Self {
        Self {
            governance_hub: Some(governance_hub),
            simple_voting: Some(simple_voting),
        }
    }

    pub fn create_mock_governance_hub(&self) -> Arc<dyn GovernanceHubContract + Send + Sync> {
        Arc::new(MockGovernanceHub::new())
    }

    pub fn create_mock_simple_voting(&self) -> Arc<dyn SimpleVotingContract + Send + Sync> {
        Arc::new(MockSimpleVoting::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_governance_hub() {
        let hub = MockGovernanceHub::new();
        
        let receipt = hub
            .create_proposal(
                "QmTest123".to_string(),
                U256::from(86400), // 24 hours
                0, // Simple voting
            )
            .await
            .unwrap();
        
        assert_eq!(receipt.status, Some(U64::from(1)));
        
        let proposal = hub.get_proposal(1).await.unwrap();
        assert_eq!(proposal.id, 1);
        assert_eq!(proposal.ipfs_hash, "QmTest123");
        
        let count = hub.get_proposal_count().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_mock_simple_voting() {
        let voting = MockSimpleVoting::new();
        
        let receipt = voting
            .cast_vote(1, 1, Some("QmVote123".to_string()))
            .await
            .unwrap();
        
        assert_eq!(receipt.status, Some(U64::from(1)));
        
        let tally = voting.get_vote_tally(1).await.unwrap();
        assert_eq!(tally.0, U256::from(1000)); // yes votes
        assert_eq!(tally.1, U256::zero()); // no votes
        assert_eq!(tally.2, U256::zero()); // abstain votes
    }
}