use config::{ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub blockchain: BlockchainConfig,
    pub ipfs: IpfsConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub rpc_url: String,
    pub chain_id: u64,
    pub contracts: ContractConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractConfig {
    pub governance_hub: Option<String>,
    pub proposal_manager: Option<String>,
    pub simple_voting: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpfsConfig {
    pub api_url: String,
    pub gateway_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub message_template: String,
    pub signature_ttl: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut builder = config::Config::builder()
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("blockchain.rpc_url", "http://localhost:8545")?
            .set_default("blockchain.chain_id", 1337)?
            .set_default("ipfs.api_url", "http://localhost:5001")?
            .set_default("ipfs.gateway_url", "http://localhost:8080")?
            .set_default("auth.message_template", "Sign this message to authenticate with Somnia Governance Engine: {nonce}")?
            .set_default("auth.signature_ttl", 300)?; // 5 minutes

        // Try to load from config file if it exists
        if let Ok(config_path) = env::var("CONFIG_PATH") {
            builder = builder.add_source(File::with_name(&config_path).required(false));
        } else {
            builder = builder.add_source(File::with_name("config.toml").required(false));
        }

        // Override with environment variables
        builder = builder.add_source(Environment::with_prefix("GOVERNANCE").separator("_"));

        let config = builder.build()?;
        config.try_deserialize()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
            },
            blockchain: BlockchainConfig {
                rpc_url: "http://localhost:8545".to_string(),
                chain_id: 1337,
                contracts: ContractConfig {
                    governance_hub: None,
                    proposal_manager: None,
                    simple_voting: None,
                },
            },
            ipfs: IpfsConfig {
                api_url: "http://localhost:5001".to_string(),
                gateway_url: "http://localhost:8080".to_string(),
            },
            auth: AuthConfig {
                message_template: "Sign this message to authenticate with Somnia Governance Engine: {nonce}".to_string(),
                signature_ttl: 300,
            },
        }
    }
}