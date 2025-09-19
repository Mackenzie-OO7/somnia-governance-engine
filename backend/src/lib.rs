pub mod config;
pub mod blockchain;
pub mod ipfs;
pub mod governance;
pub mod api;
pub mod auth;
pub mod indexer;
pub mod performance;
pub mod utils;

pub use config::Config;
pub use utils::errors::Result;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub blockchain_client: blockchain::client::SomniaClient,
    pub ipfs_client: ipfs::client::IpfsClient,
    pub governance_engine: governance::engine::GovernanceEngine,
}