use crate::utils::errors::{GovernanceError, Result};
use ethers::prelude::*;
use ethers::types::transaction::eip2718::TypedTransaction;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct TransactionManager {
    provider: Arc<Provider<Ws>>,
    pending_transactions: Arc<RwLock<HashMap<H256, PendingTransaction>>>,
    gas_oracle: GasOracle,
}

#[derive(Debug, Clone)]
pub struct PendingTransaction {
    pub hash: H256,
    pub transaction_type: TransactionType,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub confirmations_required: u64,
    pub current_confirmations: u64,
    pub max_wait_time: std::time::Duration,
}

#[derive(Debug, Clone)]
pub enum TransactionType {
    CreateProposal { ipfs_hash: String },
    CastVote { proposal_id: u64, choice: u8 },
    ExecuteProposal { proposal_id: u64 },
}

#[derive(Debug, Clone)]
pub struct GasOracle {
    base_fee: U256,
    priority_fee: U256,
    max_fee_per_gas: U256,
}

impl TransactionManager {
    pub fn new(provider: Arc<Provider<Ws>>) -> Self {
        Self {
            provider,
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
            gas_oracle: GasOracle::default(),
        }
    }

    pub async fn submit_transaction(
        &self,
        tx: TypedTransaction,
        transaction_type: TransactionType,
    ) -> Result<H256> {
        // Estimate gas
        let gas_estimate = self.provider
            .estimate_gas(&tx, None)
            .await
            .map_err(GovernanceError::Blockchain)?;

        // Set gas parameters
        let mut tx = tx;
        tx.set_gas(gas_estimate * 110 / 100); // Add 10% buffer
        
        // Set gas price using oracle
        self.set_gas_price(&mut tx).await?;

        // Submit transaction
        let pending_tx = self.provider
            .send_transaction(tx, None)
            .await
            .map_err(GovernanceError::Blockchain)?;

        let tx_hash = pending_tx.tx_hash();

        // Track the transaction
        let pending = PendingTransaction {
            hash: tx_hash,
            transaction_type,
            submitted_at: chrono::Utc::now(),
            confirmations_required: 1, // Somnia has fast finality
            current_confirmations: 0,
            max_wait_time: std::time::Duration::from_secs(30),
        };

        self.pending_transactions
            .write()
            .await
            .insert(tx_hash, pending);

        tracing::info!("Submitted transaction: {:?}", tx_hash);
        Ok(tx_hash)
    }

    pub async fn wait_for_confirmation(
        &self,
        tx_hash: H256,
        confirmations: u64,
    ) -> Result<TransactionReceipt> {
        let timeout = std::time::Duration::from_secs(60);
        
        tokio::time::timeout(timeout, async {
            loop {
                if let Some(receipt) = self.provider
                    .get_transaction_receipt(tx_hash)
                    .await
                    .map_err(GovernanceError::Blockchain)?
                {
                    if receipt.status == Some(U64::from(1)) {
                        // Update pending transaction
                        if let Some(pending) = self.pending_transactions
                            .write()
                            .await
                            .get_mut(&tx_hash)
                        {
                            pending.current_confirmations = confirmations;
                        }

                        return Ok(receipt);
                    } else {
                        return Err(GovernanceError::ipfs("Transaction failed"));
                    }
                }

                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        })
        .await
        .map_err(|_| GovernanceError::ipfs("Transaction confirmation timeout"))?
    }

    pub async fn get_transaction_status(&self, tx_hash: H256) -> Result<TransactionStatus> {
        // Check if it's a pending transaction we're tracking
        if let Some(_pending) = self.pending_transactions.read().await.get(&tx_hash) {
            // Check for receipt
            if let Some(receipt) = self.provider
                .get_transaction_receipt(tx_hash)
                .await
                .map_err(GovernanceError::Blockchain)?
            {
                if receipt.status == Some(U64::from(1)) {
                    return Ok(TransactionStatus::Confirmed(receipt));
                } else {
                    return Ok(TransactionStatus::Failed(receipt));
                }
            } else {
                return Ok(TransactionStatus::Pending(_pending.clone()));
            }
        }

        // Not in our tracking, check on-chain
        match self.provider
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(GovernanceError::Blockchain)?
        {
            Some(receipt) => {
                if receipt.status == Some(U64::from(1)) {
                    Ok(TransactionStatus::Confirmed(receipt))
                } else {
                    Ok(TransactionStatus::Failed(receipt))
                }
            }
            None => Ok(TransactionStatus::NotFound),
        }
    }

    async fn set_gas_price(&self, tx: &mut TypedTransaction) -> Result<()> {
        // For Somnia, gas prices should be very low
        // This is a simplified implementation
        
        match tx {
            TypedTransaction::Eip1559(ref mut eip1559_tx) => {
                eip1559_tx.max_fee_per_gas = Some(self.gas_oracle.max_fee_per_gas);
                eip1559_tx.max_priority_fee_per_gas = Some(self.gas_oracle.priority_fee);
            }
            TypedTransaction::Legacy(ref mut legacy_tx) => {
                legacy_tx.gas_price = Some(self.gas_oracle.base_fee);
            }
            TypedTransaction::Eip2930(ref mut eip2930_tx) => {
                eip2930_tx.tx.gas_price = Some(self.gas_oracle.base_fee);
            }
        }

        Ok(())
    }

    pub async fn update_gas_oracle(&mut self) -> Result<()> {
        // In production, this would fetch current gas prices from the network
        // For Somnia, gas prices should be very low and stable
        
        self.gas_oracle = GasOracle {
            base_fee: U256::from(1_000_000_000u64), // 1 Gwei
            priority_fee: U256::from(1_000_000_000u64), // 1 Gwei
            max_fee_per_gas: U256::from(2_000_000_000u64), // 2 Gwei
        };

        tracing::debug!("Updated gas oracle: {:?}", self.gas_oracle);
        Ok(())
    }

    pub async fn cleanup_old_transactions(&self) {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(1);
        
        let mut pending = self.pending_transactions.write().await;
        pending.retain(|_, tx| tx.submitted_at > cutoff);
        
        tracing::debug!("Cleaned up old transactions, {} remaining", pending.len());
    }

    pub async fn get_pending_count(&self) -> usize {
        self.pending_transactions.read().await.len()
    }
}

#[derive(Debug, Clone)]
pub enum TransactionStatus {
    Pending(PendingTransaction),
    Confirmed(TransactionReceipt),
    Failed(TransactionReceipt),
    NotFound,
}

impl Default for GasOracle {
    fn default() -> Self {
        Self {
            base_fee: U256::from(1_000_000_000u64), // 1 Gwei
            priority_fee: U256::from(1_000_000_000u64), // 1 Gwei  
            max_fee_per_gas: U256::from(2_000_000_000u64), // 2 Gwei
        }
    }
}

// Helper functions for creating common transactions
pub fn create_proposal_transaction(
    contract_address: Address,
    ipfs_hash: String,
    voting_duration: U256,
    proposal_type: u8,
    nonce: U256,
) -> TypedTransaction {
    // In production, this would use proper ABI encoding
    // For now, we create a mock transaction
    
    let tx = Eip1559TransactionRequest::new()
        .to(contract_address)
        .nonce(nonce)
        .data(format!("createProposal({},{},{})", ipfs_hash, voting_duration, proposal_type).into_bytes())
        .value(U256::zero());

    TypedTransaction::Eip1559(tx)
}

pub fn cast_vote_transaction(
    contract_address: Address,
    proposal_id: u64,
    choice: u8,
    ipfs_hash: Option<String>,
    nonce: U256,
) -> TypedTransaction {
    // In production, this would use proper ABI encoding
    
    let data = match ipfs_hash {
        Some(hash) => format!("castVote({},{},{})", proposal_id, choice, hash),
        None => format!("castVote({},{})", proposal_id, choice),
    };

    let tx = Eip1559TransactionRequest::new()
        .to(contract_address)
        .nonce(nonce)
        .data(data.into_bytes())
        .value(U256::zero());

    TypedTransaction::Eip1559(tx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_oracle_default() {
        let oracle = GasOracle::default();
        assert_eq!(oracle.base_fee, U256::from(1_000_000_000u64));
        assert_eq!(oracle.priority_fee, U256::from(1_000_000_000u64));
        assert_eq!(oracle.max_fee_per_gas, U256::from(2_000_000_000u64));
    }

    #[test]
    fn test_transaction_creation() {
        let contract_addr = Address::random();
        let tx = create_proposal_transaction(
            contract_addr,
            "QmTest123".to_string(),
            U256::from(86400),
            0,
            U256::from(1),
        );

        match tx {
            TypedTransaction::Eip1559(eip1559_tx) => {
                assert_eq!(eip1559_tx.to, Some(NameOrAddress::Address(contract_addr)));
                assert_eq!(eip1559_tx.nonce, Some(U256::from(1)));
            }
            _ => panic!("Expected EIP-1559 transaction"),
        }
    }
}