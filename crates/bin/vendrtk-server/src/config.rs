use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::OnceLock,
};

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFIG.get_or_init(Config::from_env)
}

pub struct Config {
    pub ip: IpAddr,
    pub port: u16,
    pub public_dir: String,
    pub log_level: String,
    pub azure_cognitive_services_endpoint: String,
    pub azure_cognitive_services_key: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 8080,
            public_dir: "public".into(),
            log_level: "info".into(),
            azure_cognitive_services_endpoint: "".into(),
            azure_cognitive_services_key: "".into(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let defaults = Self::default();

        Self {
            ip: env::var("SERVER_IP")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.ip),

            port: env::var("SERVER_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.port),

            public_dir: env::var("PUBLIC_DIR").unwrap_or(defaults.public_dir),

            log_level: env::var("LOG_LEVEL").unwrap_or(defaults.log_level),

            azure_cognitive_services_endpoint: env::var("AZURE_COGNITIVE_SERVICES_ENDPOINT").unwrap_or(defaults.azure_cognitive_services_endpoint),
            azure_cognitive_services_key: env::var("AZURE_COGNITIVE_SERVICES_KEY").unwrap_or(defaults.azure_cognitive_services_key),
        }
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }
}
