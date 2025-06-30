use std::fs;
use std::path::Path;

fn main() {
    let content = r#"deny_methods:
  - eth_sendTransaction
  - personal_sign
  - eth_sendRawTransaction
"#;

    let path = Path::new("config/rules.yaml");
    fs::create_dir_all("config").expect("Could not create config directory");
    fs::write(path, content).expect("Could not write rules.yaml");
    println!("✅ rules.yaml created successfully.");
}
