use config::Config;

lazy_static! {
    pub static ref CONFIG: Config = load_config();
}

fn load_config() -> Config {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Config")).unwrap();
    settings
}

pub fn websocket_url() -> String {
    CONFIG.get_str("websocket-url").unwrap()
}