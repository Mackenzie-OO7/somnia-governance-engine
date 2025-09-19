use validator::ValidationError;

pub fn validate_ethereum_address(address: &str) -> Result<(), ValidationError> {
    if address.len() != 42 {
        return Err(ValidationError::new("invalid_length"));
    }
    
    if !address.starts_with("0x") {
        return Err(ValidationError::new("invalid_prefix"));
    }
    
    if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::new("invalid_hex"));
    }
    
    Ok(())
}

pub fn validate_ipfs_hash(hash: &str) -> Result<(), ValidationError> {
    if hash.is_empty() {
        return Err(ValidationError::new("empty_hash"));
    }
    
    if hash.len() < 46 {
        return Err(ValidationError::new("hash_too_short"));
    }
    
    // Check for common IPFS hash prefixes
    if !hash.starts_with("Qm") && !hash.starts_with("baf") && !hash.starts_with("bae") {
        return Err(ValidationError::new("invalid_hash_format"));
    }
    
    Ok(())
}

pub fn validate_voting_duration(duration: u64) -> Result<(), ValidationError> {
    const MIN_DURATION: u64 = 3600; // 1 hour
    const MAX_DURATION: u64 = 2592000; // 30 days
    
    if duration < MIN_DURATION {
        return Err(ValidationError::new("duration_too_short"));
    }
    
    if duration > MAX_DURATION {
        return Err(ValidationError::new("duration_too_long"));
    }
    
    Ok(())
}

pub fn validate_proposal_title(title: &str) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        return Err(ValidationError::new("empty_title"));
    }
    
    if title.len() > 200 {
        return Err(ValidationError::new("title_too_long"));
    }
    
    Ok(())
}

pub fn validate_proposal_description(description: &str) -> Result<(), ValidationError> {
    if description.trim().is_empty() {
        return Err(ValidationError::new("empty_description"));
    }
    
    if description.len() > 50000 {
        return Err(ValidationError::new("description_too_long"));
    }
    
    Ok(())
}