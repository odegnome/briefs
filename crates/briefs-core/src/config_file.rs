use std::{net::SocketAddr, path::PathBuf};

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
