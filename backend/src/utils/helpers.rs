use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

impl PaginationParams {
    pub fn page(&self) -> u64 {
        self.page.unwrap_or(1)
    }

    pub fn limit(&self) -> u64 {
        self.limit.unwrap_or(20).min(100) // Max 100 items per page
    }

    pub fn offset(&self) -> u64 {
        (self.page() - 1) * self.limit()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: u64,
    pub limit: u64,
    pub total: u64,
    pub has_next: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: u64, limit: u64, total: u64) -> Self {
        let has_next = (page * limit) < total;
        Self {
            data,
            page,
            limit,
            total,
            has_next,
        }
    }
}

pub fn current_timestamp() -> u64 {
    Utc::now().timestamp() as u64
}

pub fn format_address(address: &str) -> String {
    if address.len() >= 10 {
        format!("{}...{}", &address[0..6], &address[address.len()-4..])
    } else {
        address.to_string()
    }
}

pub fn validate_ethereum_address(address: &str) -> bool {
    address.len() == 42 && address.starts_with("0x") && address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

pub fn validate_ipfs_hash(hash: &str) -> bool {
    // Basic IPFS hash validation (CIDv1)
    hash.len() >= 46 && (hash.starts_with("Qm") || hash.starts_with("baf"))
}