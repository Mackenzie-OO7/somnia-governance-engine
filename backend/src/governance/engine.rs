use crate::blockchain::client::SomniaClient;
use crate::ipfs::client::IpfsClient;
use crate::utils::errors::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct GovernanceEngine {
    blockchain_client: Arc<SomniaClient>,
    ipfs_client: Arc<IpfsClient>,
}

impl GovernanceEngine {
    pub async fn new(
        blockchain_client: SomniaClient,
        ipfs_client: IpfsClient,
    ) -> Result<Self> {
        Ok(Self {
            blockchain_client: Arc::new(blockchain_client),
            ipfs_client: Arc::new(ipfs_client),
        })
    }

    pub fn blockchain_client(&self) -> &Arc<SomniaClient> {
        &self.blockchain_client
    }

    pub fn ipfs_client(&self) -> &Arc<IpfsClient> {
        &self.ipfs_client
    }
}