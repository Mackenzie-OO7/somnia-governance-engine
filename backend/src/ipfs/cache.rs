use lru::LruCache;
use serde_json::Value;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone)]
pub struct CachedItem {
    pub content: Value,
    pub cached_at: DateTime<Utc>,
    pub ttl: Option<Duration>,
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
}

impl CachedItem {
    pub fn new(content: Value, ttl: Option<Duration>) -> Self {
        let now = Utc::now();
        Self {
            content,
            cached_at: now,
            ttl,
            access_count: 1,
            last_accessed: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            Utc::now() > self.cached_at + ttl
        } else {
            false // No TTL means never expires (immutable IPFS content)
        }
    }

    pub fn access(&mut self) -> &Value {
        self.access_count += 1;
        self.last_accessed = Utc::now();
        &self.content
    }
}

#[derive(Clone)]
pub struct IpfsCache {
    cache: Arc<RwLock<LruCache<String, CachedItem>>>,
    max_size: usize,
}

impl IpfsCache {
    pub fn new(max_size: usize) -> Self {
        let cache = Arc::new(RwLock::new(
            LruCache::new(NonZeroUsize::new(max_size).unwrap())
        ));
        
        Self {
            cache,
            max_size,
        }
    }

    pub async fn get(&self, hash: &str) -> Option<Value> {
        let mut cache = self.cache.write().await;
        
        if let Some(item) = cache.get_mut(hash) {
            if item.is_expired() {
                cache.pop(hash);
                None
            } else {
                Some(item.access().clone())
            }
        } else {
            None
        }
    }

    pub async fn put(&self, hash: String, content: Value, ttl: Option<Duration>) {
        let mut cache = self.cache.write().await;
        let item = CachedItem::new(content, ttl);
        cache.put(hash, item);
    }

    pub async fn remove(&self, hash: &str) -> Option<CachedItem> {
        let mut cache = self.cache.write().await;
        cache.pop(hash)
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    pub async fn size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let mut total_access_count = 0;
        let mut expired_count = 0;

        for (_, item) in cache.iter() {
            total_access_count += item.access_count;
            if item.is_expired() {
                expired_count += 1;
            }
        }

        CacheStats {
            total_items: cache.len(),
            max_capacity: self.max_size,
            total_access_count,
            expired_items: expired_count,
        }
    }

    pub async fn cleanup_expired(&self) -> usize {
        let mut cache = self.cache.write().await;
        let mut expired_keys = Vec::new();
        
        for (key, item) in cache.iter() {
            if item.is_expired() {
                expired_keys.push(key.clone());
            }
        }
        
        let count = expired_keys.len();
        for key in expired_keys {
            cache.pop(&key);
        }
        
        count
    }

    // Get cache hit rate for monitoring
    pub async fn hit_rate(&self) -> f64 {
        let stats = self.stats().await;
        if stats.total_access_count == 0 {
            0.0
        } else {
            // This is a simplified calculation
            // In a real implementation, you'd track hits vs misses
            stats.total_items as f64 / stats.max_capacity as f64
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_items: usize,
    pub max_capacity: usize,
    pub total_access_count: u64,
    pub expired_items: usize,
}

// Background task to periodically clean up expired items
pub async fn start_cache_cleanup_task(cache: Arc<IpfsCache>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
    
    loop {
        interval.tick().await;
        let cleaned = cache.cleanup_expired().await;
        if cleaned > 0 {
            tracing::debug!("Cleaned up {} expired cache items", cleaned);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_cache_operations() {
        let cache = IpfsCache::new(10);
        let test_hash = "QmTest123";
        let test_content = json!({"test": "data"});

        // Test put and get
        cache.put(test_hash.to_string(), test_content.clone(), None).await;
        let retrieved = cache.get(test_hash).await.unwrap();
        assert_eq!(retrieved, test_content);

        // Test stats
        let stats = cache.stats().await;
        assert_eq!(stats.total_items, 1);
        assert_eq!(stats.max_capacity, 10);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = IpfsCache::new(10);
        let test_hash = "QmTest456";
        let test_content = json!({"test": "expiring_data"});
        let short_ttl = Duration::milliseconds(100);

        // Put with short TTL
        cache.put(test_hash.to_string(), test_content, Some(short_ttl)).await;
        
        // Should be available immediately
        assert!(cache.get(test_hash).await.is_some());
        
        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        
        // Should be expired and removed
        assert!(cache.get(test_hash).await.is_none());
    }
}