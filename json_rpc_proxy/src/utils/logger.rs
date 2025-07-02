

pub fn init() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .compact()
        .init();
}
