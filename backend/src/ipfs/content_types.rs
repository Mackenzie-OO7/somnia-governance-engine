use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ProposalIPFSContent {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    
    #[validate(length(min = 1, max = 50000))]
    pub description: String,
    
    pub metadata: ProposalMetadata,
    pub version: String,
    pub content_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalMetadata {
    pub category: String,
    pub tags: Vec<String>,
    pub attachments: Vec<String>, // IPFS hashes
    pub proposal_type: ProposalType,
    pub execution_data: Option<ExecutionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    #[serde(rename = "simple")]
    Simple,
    #[serde(rename = "quadratic")]
    Quadratic,
    #[serde(rename = "ranked")]
    RankedChoice,
    #[serde(rename = "liquid")]
    LiquidDemocracy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionData {
    pub target_contract: String,
    pub function_signature: String,
    pub call_data: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VoteIPFSContent {
    pub choice: VoteChoice,
    
    #[validate(length(max = 1000))]
    pub comment: Option<String>,
    
    #[validate(length(max = 2000))]
    pub reasoning: Option<String>,
    
    pub metadata: VoteMetadata,
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteChoice {
    #[serde(rename = "yes")]
    Yes,
    #[serde(rename = "no")]
    No,
    #[serde(rename = "abstain")]
    Abstain,
}

impl From<u8> for VoteChoice {
    fn from(value: u8) -> Self {
        match value {
            0 => VoteChoice::No,
            1 => VoteChoice::Yes,
            2 => VoteChoice::Abstain,
            _ => VoteChoice::Abstain,
        }
    }
}

impl From<VoteChoice> for u8 {
    fn from(choice: VoteChoice) -> Self {
        match choice {
            VoteChoice::No => 0,
            VoteChoice::Yes => 1,
            VoteChoice::Abstain => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteMetadata {
    pub voting_power: String,
    pub delegated_votes: Option<Vec<DelegatedVote>>,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegatedVote {
    pub delegator: String,
    pub power: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserProfileIPFS {
    #[validate(length(max = 100))]
    pub display_name: Option<String>,
    
    #[validate(length(max = 500))]
    pub bio: Option<String>,
    
    pub avatar: Option<String>, // IPFS hash
    pub social: SocialLinks,
    pub preferences: UserPreferences,
    pub content_type: String,
    pub version: String,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialLinks {
    pub twitter: Option<String>,
    pub github: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub notifications: NotificationSettings,
    pub theme: ThemeSettings,
    pub privacy: PrivacySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_enabled: bool,
    pub browser_enabled: bool,
    pub proposal_updates: bool,
    pub vote_reminders: bool,
    pub governance_news: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeSettings {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "auto")]
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub profile_visible: bool,
    pub voting_history_visible: bool,
    pub activity_visible: bool,
}

impl Default for ProposalMetadata {
    fn default() -> Self {
        Self {
            category: "general".to_string(),
            tags: vec![],
            attachments: vec![],
            proposal_type: ProposalType::Simple,
            execution_data: None,
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            email_enabled: true,
            browser_enabled: true,
            proposal_updates: true,
            vote_reminders: true,
            governance_news: false,
        }
    }
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            profile_visible: true,
            voting_history_visible: true,
            activity_visible: true,
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            notifications: NotificationSettings::default(),
            theme: ThemeSettings::Auto,
            privacy: PrivacySettings::default(),
        }
    }
}