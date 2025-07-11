// services/multichain/router.rs
use super::{ChainAdapter, ChainId};
use std::sync::Arc;

pub fn get_adapter(chain: &ChainId) -> Arc<dyn ChainAdapter> {
    match chain {
        ChainId::Ethereum => Arc::new(super::ethereum::EthereumAdapter),
        ChainId::Solana => Arc::new(super::solana::SolanaAdapter),
        ChainId::Aptos => Arc::new(super::aptos::AptosAdapter),
        ChainId::Sui => Arc::new(super::sui::SuiAdapter),
    }
}
