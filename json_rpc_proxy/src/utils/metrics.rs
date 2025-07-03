use prometheus::{IntCounterVec, HistogramVec, Encoder, TextEncoder, register_int_counter_vec, register_histogram_vec};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REQUESTS: IntCounterVec = register_int_counter_vec!(
        "rpc_requests_total",
        "Total number of RPC requests",
        &["method"]
    ).unwrap();

    pub static ref BLOCKED_REQUESTS: IntCounterVec = register_int_counter_vec!(
        "rpc_blocked_total",
        "Total number of blocked requests",
        &["reason"]
    ).unwrap();

    pub static ref LATENCIES: HistogramVec = register_histogram_vec!(
        "rpc_latency_seconds",
        "Request latency in seconds",
        &["method"]
    ).unwrap();
}

pub async fn metrics_handler() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}


