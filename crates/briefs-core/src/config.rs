use home::home_dir;
use std::{
    io::Write,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

const CONFIG_DIR: &str = ".briefs";
const CONFIG_FILE: &str = "briefs.toml";

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
    /// Path of the config file; eg $HOME/.config/
    pub dirpath: PathBuf,
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
            dirpath: home_dir.join(CONFIG_DIR),
        }
    }
}

impl BriefsConfig {
    /// Write the config to path
    pub fn save(&self) -> anyhow::Result<()> {
        // • Make sure filepath exists
        std::fs::create_dir_all(self.dirpath.clone())?;
        // • Create config file
        let filepath = self.dirpath.join(CONFIG_FILE);
        let mut fptr = std::fs::File::create(filepath)?;
        let config = String::from(format!(
            "[config]\
            \nsocket = \"{}\"\
            \ncert = \"{}\"\
            \npkey = \"{}\"\
            \ndb = \"{}\"",
            self.socket.to_string(),
            self.cert.to_str().unwrap_or_default(),
            self.pkey.to_str().unwrap_or_default(),
            self.db.to_str().unwrap_or_default()
        ));
        fptr.write_all(config.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_save() -> anyhow::Result<()> {
        let config = BriefsConfig::default();

        config.save()?;

        Ok(())
    }
}
