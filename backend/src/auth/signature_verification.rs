use crate::utils::errors::{GovernanceError, Result};
use ethers::core::types::Address;
use ethers::utils::hash_message;
use secp256k1::{ecdsa::RecoverableSignature, Message, Secp256k1};
use sha3::{Digest, Keccak256};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct SignatureVerifier {
    secp: Secp256k1<secp256k1::All>,
}

impl SignatureVerifier {
    pub fn new() -> Self {
        Self {
            secp: Secp256k1::new(),
        }
    }

    /// Verify a signature and recover the signer's address
    pub fn verify_signature(
        &self,
        message: &str,
        signature: &str,
    ) -> Result<Address> {
        // Parse signature
        let signature_bytes = hex::decode(signature.strip_prefix("0x").unwrap_or(signature))
            .map_err(|_| GovernanceError::invalid_signature("Invalid hex signature"))?;

        if signature_bytes.len() != 65 {
            return Err(GovernanceError::invalid_signature("Signature must be 65 bytes"));
        }

        let recovery_id = signature_bytes[64];
        let signature_data = &signature_bytes[0..64];

        // Create recoverable signature
        let recovery_id = secp256k1::ecdsa::RecoveryId::from_u8_masked(recovery_id);

        let signature = RecoverableSignature::from_compact(signature_data, recovery_id)
            .map_err(|_| GovernanceError::invalid_signature("Invalid signature format"))?;

        // Hash the message using Ethereum's signing scheme
        let message_hash = hash_message(message);
        let message = Message::from_digest(message_hash.as_fixed_bytes().clone());

        // Recover public key
        let public_key = self.secp.recover_ecdsa(message, &signature)
            .map_err(|_| GovernanceError::invalid_signature("Failed to recover public key"))?;

        // Convert to Ethereum address
        let address = self.public_key_to_address(&public_key);
        
        Ok(address)
    }

    /// Verify that a signature was created by a specific address
    pub fn verify_signature_for_address(
        &self,
        message: &str,
        signature: &str,
        expected_address: &Address,
    ) -> Result<bool> {
        let recovered_address = self.verify_signature(message, signature)?;
        Ok(recovered_address == *expected_address)
    }

    /// Convert a secp256k1 public key to an Ethereum address
    fn public_key_to_address(&self, public_key: &secp256k1::PublicKey) -> Address {
        let public_key_bytes = public_key.serialize_uncompressed();
        
        // Take the last 64 bytes (remove the 0x04 prefix)
        let public_key_hash = Keccak256::digest(&public_key_bytes[1..]);
        
        // Take the last 20 bytes as the address
        let mut address_bytes = [0u8; 20];
        address_bytes.copy_from_slice(&public_key_hash[12..]);
        
        Address::from(address_bytes)
    }

    /// Create a message for signing following EIP-191 standard
    pub fn create_sign_message(&self, nonce: &str, template: &str) -> String {
        template.replace("{nonce}", nonce)
    }

    /// Generate a cryptographically secure nonce
    pub fn generate_nonce() -> String {
        use rand::Rng;
        let mut rng = rand::rng();
        let nonce: u64 = rng.random();
        format!("{:016x}", nonce)
    }

    /// Validate message format
    pub fn validate_message(&self, message: &str) -> Result<()> {
        if message.is_empty() {
            return Err(GovernanceError::invalid_signature("Message cannot be empty"));
        }

        if message.len() > 1000 {
            return Err(GovernanceError::invalid_signature("Message too long"));
        }

        Ok(())
    }

    /// Check if a signature is in the correct format
    pub fn is_valid_signature_format(&self, signature: &str) -> bool {
        // Remove 0x prefix if present
        let sig = signature.strip_prefix("0x").unwrap_or(signature);
        
        // Must be exactly 130 hex characters (65 bytes)
        sig.len() == 130 && sig.chars().all(|c| c.is_ascii_hexdigit())
    }
}

impl Default for SignatureVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for address validation
pub fn is_valid_ethereum_address(address: &str) -> bool {
    if let Ok(_) = Address::from_str(address) {
        true
    } else {
        false
    }
}

pub fn normalize_address(address: &str) -> Result<Address> {
    Address::from_str(address)
        .map_err(|_| GovernanceError::invalid_signature("Invalid address format"))
}

/// Authentication message templates
pub struct AuthMessageTemplates;

impl AuthMessageTemplates {
    pub const DEFAULT: &'static str = 
        "Sign this message to authenticate with Somnia Governance Engine: {nonce}";
    
    pub const WITH_TIMESTAMP: &'static str = 
        "Authenticate with Somnia Governance Engine\nNonce: {nonce}\nTimestamp: {timestamp}";
        
    pub const WITH_DOMAIN: &'static str = 
        "governance.somnia.network wants you to sign in with your Ethereum account:\n{address}\n\nSign this message to authenticate.\n\nNonce: {nonce}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_verifier_creation() {
        let verifier = SignatureVerifier::new();
        assert!(true); // Just test that it doesn't panic
    }

    #[test]
    fn test_nonce_generation() {
        let nonce1 = SignatureVerifier::generate_nonce();
        let nonce2 = SignatureVerifier::generate_nonce();
        
        assert_ne!(nonce1, nonce2);
        assert_eq!(nonce1.len(), 16); // 8 bytes as hex = 16 chars
        assert!(nonce1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_message_creation() {
        let verifier = SignatureVerifier::new();
        let nonce = "1234567890abcdef";
        let message = verifier.create_sign_message(nonce, AuthMessageTemplates::DEFAULT);
        
        assert!(message.contains(nonce));
        assert!(!message.contains("{nonce}"));
    }

    #[test]
    fn test_signature_format_validation() {
        let verifier = SignatureVerifier::new();
        
        // Valid format (mock signature)
        let valid_sig = "0x".to_owned() + &"a".repeat(130);
        assert!(verifier.is_valid_signature_format(&valid_sig));
        
        // Without 0x prefix
        let valid_sig_no_prefix = "a".repeat(130);
        assert!(verifier.is_valid_signature_format(&valid_sig_no_prefix));
        
        // Too short
        let invalid_sig = "0x".to_owned() + &"a".repeat(128);
        assert!(!verifier.is_valid_signature_format(&invalid_sig));

        // Invalid hex
        let invalid_hex = "0x".to_owned() + &"g".repeat(130);
        assert!(!verifier.is_valid_signature_format(&invalid_hex));
    }

    #[test]
    fn test_message_validation() {
        let verifier = SignatureVerifier::new();
        
        // Valid message
        assert!(verifier.validate_message("Valid message").is_ok());
        
        // Empty message
        assert!(verifier.validate_message("").is_err());
        
        // Too long message
        let long_message = "a".repeat(1001);
        assert!(verifier.validate_message(&long_message).is_err());
    }

    #[test]
    fn test_address_validation() {
        // Valid address
        assert!(is_valid_ethereum_address("0x742d35Cc6634C0532925a3b8D5c1b9E9C4F5e5A1"));
        
        // Invalid address
        assert!(!is_valid_ethereum_address("invalid_address"));
        assert!(!is_valid_ethereum_address("0x123")); // too short
    }
}