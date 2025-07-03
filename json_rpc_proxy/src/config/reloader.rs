use crate::FilterConfig;
use notify::{Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    fs,
    path::Path,
    sync::{mpsc::channel, Arc, RwLock},
    thread,
};
use tracing::{info, warn};

const CONFIG_PATH: &str = "config/rules.yaml";

/// Load filter config from the specified YAML path
pub fn load_filter_config() -> FilterConfig {
    let data = fs::read_to_string(CONFIG_PATH)
        .unwrap_or_else(|e| panic!("❌ Failed to read filter config at '{}': {:?}", CONFIG_PATH, e));
    serde_yaml::from_str(&data)
        .unwrap_or_else(|e| panic!("❌ Invalid YAML format in '{}': {:?}", CONFIG_PATH, e))
}

/// Watch the config file and update the shared config state on changes
pub fn start_watching_config(shared_config: Arc<RwLock<FilterConfig>>) {
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default())
        .expect("❌ Failed to initialize file watcher");

    watcher
        .watch(Path::new(CONFIG_PATH), RecursiveMode::NonRecursive)
        .expect("❌ Failed to watch config file");

    thread::spawn(move || {
        for event in rx {
            match event {
                Ok(_) => {
                    match fs::read_to_string(CONFIG_PATH) {
                        Ok(content) => match serde_yaml::from_str::<FilterConfig>(&content) {
                            Ok(new_config) => {
                                let mut config = shared_config.write().unwrap();
                                *config = new_config;
                                info!("🔁 Reloaded filter config from '{}'", CONFIG_PATH);
                            }
                            Err(err) => warn!("⚠️ Failed to parse config '{}': {:?}", CONFIG_PATH, err),
                        },
                        Err(err) => warn!("⚠️ Failed to read config '{}': {:?}", CONFIG_PATH, err),
                    }
                }
                Err(e) => warn!("⚠️ File watcher error: {:?}", e),
            }
        }
    });
}
