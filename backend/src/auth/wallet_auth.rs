use crate::auth::signature_verification::{SignatureVerifier, normalize_address};
use crate::config::Config;
use crate::utils::errors::Result;
use chrono::{DateTime, Duration, Utc};
use ethers::core::types::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthChallenge {
    pub nonce: String,
    pub message: String,
    pub address: Address,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub address: Address,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub address: String,
    pub message: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeRequest {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub challenge: String,
    pub message: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub address: Option<Address>,
    pub expires_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct WalletAuthService {
    verifier: SignatureVerifier,
    challenges: Arc<RwLock<HashMap<Address, AuthChallenge>>>,
    tokens: Arc<RwLock<HashMap<String, AuthToken>>>,
    config: Arc<Config>,
}

impl WalletAuthService {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            verifier: SignatureVerifier::new(),
            challenges: Arc::new(RwLock::new(HashMap::new())),
            tokens: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Generate a new authentication challenge for an address
    pub async fn create_challenge(&self, address: &str) -> Result<ChallengeResponse> {
        // Validate and normalize address
        let address = normalize_address(address)?;

        // Generate nonce and create message
        let nonce = SignatureVerifier::generate_nonce();
        let message = self.verifier.create_sign_message(&nonce, &self.config.auth.message_template);

        // Validate message
        self.verifier.validate_message(&message)?;

        // Create challenge
        let expires_at = Utc::now() + Duration::seconds(self.config.auth.signature_ttl as i64);
        let challenge = AuthChallenge {
            nonce: nonce.clone(),
            message: message.clone(),
            address,
            created_at: Utc::now(),
            expires_at,
        };

        // Store challenge
        self.challenges.write().await.insert(address, challenge);

        // Clean up expired challenges
        self.cleanup_expired_challenges().await;

        Ok(ChallengeResponse {
            challenge: nonce,
            message,
            expires_at,
        })
    }

    /// Verify signature and create authentication token
    pub async fn authenticate(&self, auth_request: AuthRequest) -> Result<AuthResponse> {
        // Validate address format
        let address = match normalize_address(&auth_request.address) {
            Ok(addr) => addr,
            Err(_) => {
                return Ok(AuthResponse {
                    success: false,
                    token: None,
                    address: None,
                    expires_at: None,
                    error: Some("Invalid address format".to_string()),
                });
            }
        };

        // Get stored challenge
        let challenge = match self.challenges.read().await.get(&address).cloned() {
            Some(challenge) => challenge,
            None => {
                return Ok(AuthResponse {
                    success: false,
                    token: None,
                    address: None,
                    expires_at: None,
                    error: Some("No challenge found for this address".to_string()),
                });
            }
        };

        // Check if challenge has expired
        if Utc::now() > challenge.expires_at {
            // Remove expired challenge
            self.challenges.write().await.remove(&address);
            return Ok(AuthResponse {
                success: false,
                token: None,
                address: None,
                expires_at: None,
                error: Some("Challenge expired".to_string()),
            });
        }

        // Verify message matches challenge
        if auth_request.message != challenge.message {
            return Ok(AuthResponse {
                success: false,
                token: None,
                address: None,
                expires_at: None,
                error: Some("Message does not match challenge".to_string()),
            });
        }

        // Verify signature
        match self.verifier.verify_signature_for_address(
            &auth_request.message,
            &auth_request.signature,
            &address,
        ) {
            Ok(true) => {
                // Signature is valid, create token
                let token_id = uuid::Uuid::new_v4().to_string();
                let expires_at = Utc::now() + Duration::hours(24); // 24 hour token

                let auth_token = AuthToken {
                    address,
                    issued_at: Utc::now(),
                    expires_at,
                    nonce: challenge.nonce,
                };

                // Store token
                self.tokens.write().await.insert(token_id.clone(), auth_token);

                // Remove used challenge
                self.challenges.write().await.remove(&address);

                // Clean up expired tokens
                self.cleanup_expired_tokens().await;

                tracing::info!("User authenticated successfully: {:?}", address);

                Ok(AuthResponse {
                    success: true,
                    token: Some(token_id),
                    address: Some(address),
                    expires_at: Some(expires_at),
                    error: None,
                })
            }
            Ok(false) => Ok(AuthResponse {
                success: false,
                token: None,
                address: None,
                expires_at: None,
                error: Some("Invalid signature".to_string()),
            }),
            Err(e) => Ok(AuthResponse {
                success: false,
                token: None,
                address: None,
                expires_at: None,
                error: Some(format!("Signature verification failed: {}", e)),
            }),
        }
    }

    /// Verify an authentication token
    pub async fn verify_token(&self, token: &str) -> Result<Option<AuthToken>> {
        let tokens = self.tokens.read().await;
        
        if let Some(auth_token) = tokens.get(token) {
            if Utc::now() <= auth_token.expires_at {
                Ok(Some(auth_token.clone()))
            } else {
                // Token expired
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Revoke an authentication token
    pub async fn revoke_token(&self, token: &str) -> Result<bool> {
        let removed = self.tokens.write().await.remove(token).is_some();
        if removed {
            tracing::info!("Token revoked: {}", token);
        }
        Ok(removed)
    }

    /// Get all active tokens for an address (for debugging/admin)
    pub async fn get_tokens_for_address(&self, address: &Address) -> Vec<String> {
        let tokens = self.tokens.read().await;
        tokens
            .iter()
            .filter(|(_, token)| token.address == *address && Utc::now() <= token.expires_at)
            .map(|(token_id, _)| token_id.clone())
            .collect()
    }

    /// Clean up expired challenges
    async fn cleanup_expired_challenges(&self) {
        let now = Utc::now();
        let mut challenges = self.challenges.write().await;
        let initial_count = challenges.len();
        
        challenges.retain(|_, challenge| now <= challenge.expires_at);
        
        let removed_count = initial_count - challenges.len();
        if removed_count > 0 {
            tracing::debug!("Cleaned up {} expired challenges", removed_count);
        }
    }

    /// Clean up expired tokens
    async fn cleanup_expired_tokens(&self) {
        let now = Utc::now();
        let mut tokens = self.tokens.write().await;
        let initial_count = tokens.len();
        
        tokens.retain(|_, token| now <= token.expires_at);
        
        let removed_count = initial_count - tokens.len();
        if removed_count > 0 {
            tracing::debug!("Cleaned up {} expired tokens", removed_count);
        }
    }

    /// Get authentication statistics
    pub async fn get_stats(&self) -> AuthStats {
        let challenges = self.challenges.read().await;
        let tokens = self.tokens.read().await;
        
        AuthStats {
            active_challenges: challenges.len(),
            active_tokens: tokens.len(),
            total_addresses: challenges.keys().chain(tokens.values().map(|t| &t.address)).collect::<std::collections::HashSet<_>>().len(),
        }
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let service = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                service.cleanup_expired_challenges().await;
                service.cleanup_expired_tokens().await;
            }
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AuthStats {
    pub active_challenges: usize,
    pub active_tokens: usize,
    pub total_addresses: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_challenge_creation() {
        let config = Arc::new(Config::default());
        let auth_service = WalletAuthService::new(config);
        
        let address = "0x742d35Cc6634C0532925a3b8D5c1b9E9C4F5e5A1";
        let challenge = auth_service.create_challenge(address).await.unwrap();
        
        assert!(!challenge.challenge.is_empty());
        assert!(!challenge.message.is_empty());
        assert!(challenge.expires_at > Utc::now());
    }

    #[tokio::test]
    async fn test_invalid_address() {
        let config = Arc::new(Config::default());
        let auth_service = WalletAuthService::new(config);
        
        let result = auth_service.create_challenge("invalid_address").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_authentication_without_challenge() {
        let config = Arc::new(Config::default());
        let auth_service = WalletAuthService::new(config);
        
        let auth_request = AuthRequest {
            address: "0x742d35Cc6634C0532925a3b8D5c1b9E9C4F5e5A1".to_string(),
            message: "Test message".to_string(),
            signature: "0x".to_string() + &"a".repeat(130),
        };
        
        let response = auth_service.authenticate(auth_request).await.unwrap();
        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_token_verification() {
        let config = Arc::new(Config::default());
        let auth_service = WalletAuthService::new(config);
        
        // Non-existent token
        let result = auth_service.verify_token("non_existent_token").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_stats() {
        let config = Arc::new(Config::default());
        let auth_service = WalletAuthService::new(config);
        
        let stats = auth_service.get_stats().await;
        assert_eq!(stats.active_challenges, 0);
        assert_eq!(stats.active_tokens, 0);
        assert_eq!(stats.total_addresses, 0);
    }
}