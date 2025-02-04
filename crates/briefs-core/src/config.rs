use home::home_dir;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

const CONFIG_DIR: &str = ".briefs";

pub struct BriefsConfig {
    /// Socket address used to serve. Should be <ip>:<port>
    /// Example: 127.0.0.1:8080
    pub socket: SocketAddr,
    /// Server Certificate file; Should be <name>.pem file
    pub cert: PathBuf,
    /// Server private key used with certificate; Should be <name>.pem file
    pub pkey: PathBuf,
    /// Path to sqlite Db.
    /// Optional
    pub db: PathBuf,
}

impl Default for BriefsConfig {
    fn default() -> Self {
        let home_dir = home_dir().unwrap_or_else(|| {
            std::env::current_dir().expect("Unable to get current working directory")
        });
        Self {
            socket: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            cert: PathBuf::new(),
            pkey: PathBuf::new(),
            db: home_dir.join(CONFIG_DIR),
        }
    }
}

impl BriefsConfig {
    /// Write the config to path
    pub fn save(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
