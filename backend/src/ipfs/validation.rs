use crate::ipfs::content_types::*;
use crate::utils::errors::{GovernanceError, Result};
use validator::Validate;

pub fn validate_proposal_content(content: &ProposalIPFSContent) -> Result<()> {
    // Basic validation using validator crate
    content.validate()
        .map_err(GovernanceError::Validation)?;

    // Additional business logic validation
    if content.title.trim().is_empty() {
        return Err(GovernanceError::ipfs("Proposal title cannot be empty"));
    }

    if content.description.trim().is_empty() {
        return Err(GovernanceError::ipfs("Proposal description cannot be empty"));
    }

    if content.content_type != "proposal" {
        return Err(GovernanceError::ipfs("Invalid content type for proposal"));
    }

    if content.version.is_empty() {
        return Err(GovernanceError::ipfs("Content version is required"));
    }

    // Validate metadata
    validate_proposal_metadata(&content.metadata)?;

    Ok(())
}

pub fn validate_vote_content(content: &VoteIPFSContent) -> Result<()> {
    content.validate()
        .map_err(GovernanceError::Validation)?;

    if content.content_type != "vote" {
        return Err(GovernanceError::ipfs("Invalid content type for vote"));
    }

    if content.metadata.version.is_empty() {
        return Err(GovernanceError::ipfs("Content version is required"));
    }

    Ok(())
}

pub fn validate_user_profile(content: &UserProfileIPFS) -> Result<()> {
    content.validate()
        .map_err(GovernanceError::Validation)?;

    if content.content_type != "userProfile" {
        return Err(GovernanceError::ipfs("Invalid content type for user profile"));
    }

    if content.version.is_empty() {
        return Err(GovernanceError::ipfs("Content version is required"));
    }

    // Validate social links format
    if let Some(twitter) = &content.social.twitter {
        if !twitter.is_empty() && !is_valid_twitter_handle(twitter) {
            return Err(GovernanceError::ipfs("Invalid Twitter handle format"));
        }
    }

    if let Some(github) = &content.social.github {
        if !github.is_empty() && !is_valid_github_username(github) {
            return Err(GovernanceError::ipfs("Invalid GitHub username format"));
        }
    }

    if let Some(website) = &content.social.website {
        if !website.is_empty() && !is_valid_url(website) {
            return Err(GovernanceError::ipfs("Invalid website URL format"));
        }
    }

    Ok(())
}

fn validate_proposal_metadata(metadata: &ProposalMetadata) -> Result<()> {
    if metadata.category.trim().is_empty() {
        return Err(GovernanceError::ipfs("Proposal category cannot be empty"));
    }

    // Validate tags
    for tag in &metadata.tags {
        if tag.trim().is_empty() {
            return Err(GovernanceError::ipfs("Tags cannot be empty"));
        }
        if tag.len() > 50 {
            return Err(GovernanceError::ipfs("Tags must be less than 50 characters"));
        }
    }

    if metadata.tags.len() > 10 {
        return Err(GovernanceError::ipfs("Maximum 10 tags allowed"));
    }

    // Validate attachment hashes
    for attachment in &metadata.attachments {
        if !crate::utils::helpers::validate_ipfs_hash(attachment) {
            return Err(GovernanceError::ipfs("Invalid IPFS hash in attachments"));
        }
    }

    if metadata.attachments.len() > 20 {
        return Err(GovernanceError::ipfs("Maximum 20 attachments allowed"));
    }

    // Validate execution data if present
    if let Some(execution_data) = &metadata.execution_data {
        validate_execution_data(execution_data)?;
    }

    Ok(())
}

fn validate_execution_data(execution_data: &ExecutionData) -> Result<()> {
    if !crate::utils::helpers::validate_ethereum_address(&execution_data.target_contract) {
        return Err(GovernanceError::ipfs("Invalid target contract address"));
    }

    if execution_data.function_signature.trim().is_empty() {
        return Err(GovernanceError::ipfs("Function signature cannot be empty"));
    }

    // Basic validation for function signature format
    if !execution_data.function_signature.contains('(') || !execution_data.function_signature.contains(')') {
        return Err(GovernanceError::ipfs("Invalid function signature format"));
    }

    // Validate call data is valid hex
    if !execution_data.call_data.is_empty() {
        let call_data = execution_data.call_data.strip_prefix("0x").unwrap_or(&execution_data.call_data);
        if !call_data.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(GovernanceError::ipfs("Invalid call data format"));
        }
    }

    // Validate value is a valid number
    if execution_data.value.parse::<u64>().is_err() {
        return Err(GovernanceError::ipfs("Invalid value format"));
    }

    Ok(())
}

fn is_valid_twitter_handle(handle: &str) -> bool {
    let handle = handle.strip_prefix('@').unwrap_or(handle);
    handle.len() <= 15 
        && handle.chars().all(|c| c.is_alphanumeric() || c == '_')
        && !handle.is_empty()
}

fn is_valid_github_username(username: &str) -> bool {
    username.len() <= 39
        && username.chars().all(|c| c.is_alphanumeric() || c == '-')
        && !username.starts_with('-')
        && !username.ends_with('-')
        && !username.is_empty()
}

fn is_valid_url(url: &str) -> bool {
    url::Url::parse(url).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_validate_proposal_content() {
        let content = ProposalIPFSContent {
            title: "Test Proposal".to_string(),
            description: "This is a test proposal description.".to_string(),
            metadata: ProposalMetadata::default(),
            version: "1.0".to_string(),
            content_type: "proposal".to_string(),
            created_at: Utc::now(),
        };

        assert!(validate_proposal_content(&content).is_ok());

        // Test empty title
        let mut invalid_content = content.clone();
        invalid_content.title = "".to_string();
        assert!(validate_proposal_content(&invalid_content).is_err());
    }

    #[test]
    fn test_twitter_handle_validation() {
        assert!(is_valid_twitter_handle("@username"));
        assert!(is_valid_twitter_handle("username"));
        assert!(is_valid_twitter_handle("user_name"));
        assert!(!is_valid_twitter_handle("user-name"));
        assert!(!is_valid_twitter_handle(""));
        assert!(!is_valid_twitter_handle("a".repeat(16).as_str()));
    }

    #[test]
    fn test_github_username_validation() {
        assert!(is_valid_github_username("username"));
        assert!(is_valid_github_username("user-name"));
        assert!(!is_valid_github_username("-username"));
        assert!(!is_valid_github_username("username-"));
        assert!(!is_valid_github_username(""));
        assert!(!is_valid_github_username("a".repeat(40).as_str()));
    }
}