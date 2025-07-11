use serde_json::Value;
use tracing::{info, warn};

const MIN_GAS_LIMIT: u64 = 21_000;
const MAX_GAS_LIMIT: u64 = 1_000_000;

#[derive(Debug)]
pub enum GasFilterError {
    TooLow,
    TooHigh,
    InvalidFormat,
    MissingGas,
    MissingParams,
}

pub fn check_gas_limit(method: &str, body: &Value) -> Result<(), GasFilterError> {
    if method != "eth_sendTransaction" {
        return Ok(());
    }

    let tx = body
        .get("params")
        .and_then(|params| params.get(0))
        .ok_or(GasFilterError::MissingParams)?;

    let gas_hex = tx
        .get("gas")
        .and_then(|g| g.as_str())
        .ok_or(GasFilterError::MissingGas)?;

    let gas_value = u64::from_str_radix(gas_hex.trim_start_matches("0x"), 16)
        .map_err(|_| GasFilterError::InvalidFormat)?;

    info!("⛽ Gas used: {}", gas_value);

    if gas_value < MIN_GAS_LIMIT {
        warn!("🚫 Gas {} is below minimum {}", gas_value, MIN_GAS_LIMIT);
        return Err(GasFilterError::TooLow);
    }

    if gas_value > MAX_GAS_LIMIT {
        warn!("🚫 Gas {} exceeds maximum {}", gas_value, MAX_GAS_LIMIT);
        return Err(GasFilterError::TooHigh);
    }

    Ok(())
}
