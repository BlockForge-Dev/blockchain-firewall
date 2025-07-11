pub mod adapter;
pub mod ethereum;
pub mod solana;
pub mod aptos;
pub mod sui;

use crate::services::multichain::adapter::ChainAdapter;

#[derive(Debug, Clone)]
pub enum ChainId {
    Ethereum,
    Solana,
    Aptos,
    Sui,
}

impl ChainId {
    pub fn from_path(path: &str) -> Option<Self> {
        match path.to_lowercase().as_str() {
            "eth" | "ethereum" => Some(Self::Ethereum),
            "sol" | "solana" => Some(Self::Solana),
            "aptos" => Some(Self::Aptos),
            "sui" => Some(Self::Sui),
            _ => None,
        }
    }
}

// 👇 This function was missing
pub fn get_adapter(chain: &ChainId) -> Box<dyn ChainAdapter> {
    match chain {
        ChainId::Ethereum => Box::new(ethereum::EthereumAdapter),
        ChainId::Solana => Box::new(solana::SolanaAdapter),
        ChainId::Aptos => Box::new(aptos::AptosAdapter),
        ChainId::Sui => Box::new(sui::SuiAdapter),
    }
}
