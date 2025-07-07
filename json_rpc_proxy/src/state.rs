use std::sync::{Arc, RwLock};
use std::sync::Mutex;


use crate::config::filter_config::FilterConfig;
use crate::wasm::plugin_engine::WasmPlugin;

#[derive(Clone)]

pub struct AppState {
    pub config: Arc<RwLock<FilterConfig>>,
    pub plugin: Arc<Mutex<WasmPlugin>>, // ✅ Wrap WasmPlugin in Mutex
}
