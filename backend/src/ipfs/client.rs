use crate::config::Config;
use crate::ipfs::content_types::*;
use crate::utils::errors::{GovernanceError, Result};
use futures::StreamExt;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient as IpfsHttpClient, TryFromUri};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use lru::LruCache;
use std::num::NonZeroUsize;

#[derive(Clone)]
pub struct IpfsClient {
    client: IpfsHttpClient,
    gateway_url: String,
    cache: Arc<RwLock<LruCache<String, CachedContent>>>,
}

#[derive(Debug, Clone)]
struct CachedContent {
    content: serde_json::Value,
    cached_at: chrono::DateTime<chrono::Utc>,
    ttl: Option<chrono::Duration>,
}

impl CachedContent {
    fn new(content: serde_json::Value, ttl: Option<chrono::Duration>) -> Self {
        Self {
            content,
            cached_at: chrono::Utc::now(),
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            chrono::Utc::now() > self.cached_at + ttl
        } else {
            false // No TTL means never expires (immutable content)
        }
    }
}

// Removed trait to avoid Send issues for hackathon performance

impl IpfsClient {
    pub async fn new(config: &Config) -> Result<Self> {
        let client = IpfsHttpClient::from_str(&config.ipfs.api_url)
            .map_err(|e| GovernanceError::ipfs(format!("Failed to create IPFS client: {}", e)))?;
        
        // Test connection
        client
            .version()
            .await
            .map_err(|e| GovernanceError::ipfs(format!("Failed to connect to IPFS: {}", e)))?;

        let cache = Arc::new(RwLock::new(LruCache::new(NonZeroUsize::new(1000).unwrap())));

        Ok(Self {
            client,
            gateway_url: config.ipfs.gateway_url.clone(),
            cache,
        })
    }

    pub async fn add_proposal_content(&self, content: &ProposalIPFSContent) -> Result<String> {
        validator::Validate::validate(content)
            .map_err(GovernanceError::Validation)?;
        
        self.add_json(content).await
    }

    pub async fn get_proposal_content(&self, hash: &str) -> Result<ProposalIPFSContent> {
        self.get_json(hash).await
    }

    pub async fn add_vote_content(&self, content: &VoteIPFSContent) -> Result<String> {
        validator::Validate::validate(content)
            .map_err(GovernanceError::Validation)?;
        
        self.add_json(content).await
    }

    pub async fn get_vote_content(&self, hash: &str) -> Result<VoteIPFSContent> {
        self.get_json(hash).await
    }

    pub async fn add_user_profile(&self, content: &UserProfileIPFS) -> Result<String> {
        validator::Validate::validate(content)
            .map_err(GovernanceError::Validation)?;
        
        self.add_json(content).await
    }

    pub async fn get_user_profile(&self, hash: &str) -> Result<UserProfileIPFS> {
        self.get_json(hash).await
    }

    pub async fn get_gateway_url(&self, hash: &str) -> String {
        format!("{}/ipfs/{}", self.gateway_url, hash)
    }

    async fn get_from_cache<T>(&self, hash: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut cache = self.cache.write().await;
        if let Some(cached) = cache.get(hash) {
            if !cached.is_expired() {
                return serde_json::from_value(cached.content.clone()).ok();
            }
        }
        None
    }

    async fn store_in_cache(&self, hash: &str, content: serde_json::Value, ttl: Option<chrono::Duration>) {
        let mut cache = self.cache.write().await;
        cache.put(hash.to_string(), CachedContent::new(content, ttl));
    }

    pub async fn add_json<T>(&self, content: &T) -> Result<String>
    where
        T: Serialize + Send + Sync,
    {
        let json_bytes = serde_json::to_vec(content)
            .map_err(GovernanceError::Serialization)?;

        let response = self
            .client
            .add(std::io::Cursor::new(json_bytes))
            .await
            .map_err(|e| GovernanceError::ipfs(format!("Failed to add content to IPFS: {}", e)))?;

        let hash = response.hash;
        
        // Pin the content to ensure it stays available
        self.pin_content(&hash).await?;
        
        tracing::info!("Added content to IPFS: {}", hash);
        Ok(hash)
    }

    pub async fn get_json<T>(&self, hash: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        // Check cache first
        if let Some(cached) = self.get_from_cache::<T>(hash).await {
            return Ok(cached);
        }

        let response = self
            .client
            .cat(hash);

        let mut bytes = Vec::new();
        let mut stream = response;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| GovernanceError::ipfs(format!("Failed to read IPFS chunk: {}", e)))?;
            bytes.extend_from_slice(&chunk);
        }
        let json_value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(GovernanceError::Serialization)?;
        
        let content: T = serde_json::from_value(json_value.clone())
            .map_err(GovernanceError::Serialization)?;

        // Cache the content (IPFS content is immutable, so no TTL)
        self.store_in_cache(hash, json_value, None).await;
        
        tracing::debug!("Retrieved content from IPFS: {}", hash);
        Ok(content)
    }

    pub async fn pin_content(&self, hash: &str) -> Result<()> {
        self.client
            .pin_add(hash, true)
            .await
            .map_err(|e| GovernanceError::ipfs(format!("Failed to pin content: {}", e)))?;
        
        tracing::debug!("Pinned content: {}", hash);
        Ok(())
    }

    pub async fn unpin_content(&self, hash: &str) -> Result<()> {
        self.client
            .pin_rm(hash, true)
            .await
            .map_err(|e| GovernanceError::ipfs(format!("Failed to unpin content: {}", e)))?;
        
        tracing::debug!("Unpinned content: {}", hash);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_ipfs_operations() {
        let config = Config::default();
        let client = IpfsClient::new(&config).await;
        
        // Skip test if IPFS is not available
        if client.is_err() {
            return;
        }
        
        let client = client.unwrap();
        
        let test_content = serde_json::json!({
            "test": "data",
            "number": 42
        });
        
        let hash = client.add_json(&test_content).await.unwrap();
        assert!(!hash.is_empty());
        
        let retrieved: serde_json::Value = client.get_json(&hash).await.unwrap();
        assert_eq!(retrieved, test_content);
    }
}