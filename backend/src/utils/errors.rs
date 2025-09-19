use thiserror::Error;

pub type Result<T> = std::result::Result<T, GovernanceError>;

#[derive(Error, Debug)]
pub enum GovernanceError {
    #[error("Blockchain error: {0}")]
    Blockchain(#[from] ethers::providers::ProviderError),

    #[error("IPFS error: {message}")]
    Ipfs { message: String },

    #[error("Proposal not found: {proposal_id}")]
    ProposalNotFound { proposal_id: u64 },

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Insufficient voting power: required {required}, available {available}")]
    InsufficientVotingPower { required: u64, available: u64 },

    #[error("Voting period ended: {proposal_id}")]
    VotingPeriodEnded { proposal_id: u64 },

    #[error("Validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Network error: {0}")]
    Network(#[from] hyper::Error),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl GovernanceError {
    pub fn ipfs<T: Into<String>>(message: T) -> Self {
        Self::Ipfs {
            message: message.into(),
        }
    }

    pub fn invalid_signature<T: Into<String>>(message: T) -> Self {
        Self::InvalidSignature(message.into())
    }
}