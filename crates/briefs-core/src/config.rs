use home::home_dir;
use regex::Regex;
use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

use crate::BriefsError;

const CONFIG_DIR: &str = ".briefs";
const CONFIG_FILE: &str = "briefs.toml";

#[derive(Debug)]
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
    /// Path of the config file directory; eg $HOME/.config/
    pub dirpath: PathBuf,
    /// Path of the config file; eg $HOME/.config/briefs.toml
    pub filepath: PathBuf,
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
            filepath: home_dir.join(CONFIG_DIR).join(CONFIG_FILE),
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
            \ndb = \"{}\"\n",
            self.socket.to_string(),
            self.cert.to_str().unwrap_or_default(),
            self.pkey.to_str().unwrap_or_default(),
            self.db.to_str().unwrap_or_default()
        ));
        fptr.write_all(config.as_bytes())?;
        Ok(())
    }

    pub fn from_file(file: PathBuf) -> anyhow::Result<Self> {
        let mut buf = String::new();
        let mut fptr = std::fs::File::open(file.clone())?;
        let _ = fptr.read_to_string(&mut buf)?;
        let mut config = BriefsConfig::default();
        config.dirpath = file
            .parent()
            .ok_or(BriefsError::config_error("filepath has no parent".into()))?
            .to_path_buf();
        config.filepath = file;

        let buf: Vec<&str> = buf.split("\n").collect();
        if buf.len() == 0 {
            return Err(BriefsError::config_error("Config file is empty".to_string()).into());
        }
        if !buf.first().unwrap().trim().contains("[config]") {
            return Err(
                BriefsError::config_error("Parsing error: header not found".to_string()).into(),
            );
        };
        let pattern = Regex::new(r#"^(?<key>\w+) ?= ?['"]?(?<val>[0-9a-zA-Z.:/]*)['"]?.*$"#)?;
        // let pattern = Regex::new(r#"^(?<key>\w+) ?= ?"?(?<val>\w+)"? *$"#)?;
        for text in buf.into_iter().skip(1) {
            if let Some(matches) = pattern.captures(text) {
                match matches.name("key").map_or("", |val| val.as_str()) {
                    "socket" => {
                        config.socket = SocketAddr::from_str(
                            matches.name("val").map_or("", |val| val.as_str()),
                        )?
                    }
                    "cert" => {
                        config.cert = matches.name("val").map_or("", |val| val.as_str()).into()
                    }
                    "pkey" => {
                        config.pkey = matches.name("val").map_or("", |val| val.as_str()).into()
                    }
                    "db" => config.db = matches.name("val").map_or("", |val| val.as_str()).into(),
                    _ => {
                        return Err(BriefsError::config_error(
                            "Parsing error: key not found".into(),
                        )
                        .into())
                    }
                }
            }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save() {
        let config = BriefsConfig::default();
        config.save().unwrap();

        let saved_config = BriefsConfig::from_file(config.filepath.clone()).unwrap();
        assert_eq!(config.socket, saved_config.socket);
        assert_eq!(config.cert, saved_config.cert);
        assert_eq!(config.pkey, saved_config.pkey);
        assert_eq!(config.db, saved_config.db);
        assert_eq!(config.filepath, saved_config.filepath);
        assert_eq!(config.dirpath, saved_config.dirpath);
    }

    #[test]
    fn test_new_from_file() {
        let mut config = BriefsConfig::default();
        config.save().unwrap();

        let saved_config = BriefsConfig::from_file(config.filepath.clone()).unwrap();
        assert_eq!(config.socket, saved_config.socket);
        assert_eq!(config.cert, saved_config.cert);
        assert_eq!(config.pkey, saved_config.pkey);
        assert_eq!(config.db, saved_config.db);
        assert_eq!(config.filepath, saved_config.filepath);
        assert_eq!(config.dirpath, saved_config.dirpath);

        config.cert = home_dir().unwrap().join(CONFIG_DIR);
        config.db = home_dir().unwrap().join(CONFIG_DIR);
        config.save().unwrap();

        let saved_config = BriefsConfig::from_file(config.filepath.clone()).unwrap();
        assert_eq!(config.socket, saved_config.socket);
        assert_eq!(config.cert, saved_config.cert);
        assert_eq!(config.pkey, saved_config.pkey);
        assert_eq!(config.db, saved_config.db);
        assert_eq!(config.filepath, saved_config.filepath);
        assert_eq!(config.dirpath, saved_config.dirpath);
    }

    #[test]
    fn test_regex() {
        // let pattern = Regex::new(r#"^(?<key>\w+) ?= ?['"]??(?<val>\w+)['"]?$"#).unwrap();
        let pattern = Regex::new(r#"^(?<key>\w+) ?= ?['"]?(?<val>[0-9a-zA-Z.:/]*)['"]?$"#).unwrap();

        let data = "socket = '0.0.0.0:80'";
        let cpt = pattern.captures(data).unwrap();
        assert_eq!(cpt.name("key").unwrap().as_str(), "socket");
        assert_eq!(cpt.name("val").unwrap().as_str(), "0.0.0.0:80");


        let data = "socket = ''";
        let cpt = pattern.captures(data).unwrap();
        assert_eq!(cpt.name("key").unwrap().as_str(), "socket");
        assert_eq!(cpt.name("val").unwrap().as_str(), "");

        let data = r#"cert = "/Users/rishabh""#;
        let cpt = pattern.captures(data).unwrap();
        assert_eq!(cpt.name("key").unwrap().as_str(), "cert");
        assert_eq!(cpt.name("val").unwrap().as_str(), "/Users/rishabh");

        let data = r#"db = "/Users/rishabh/.briefs""#;
        let cpt = pattern.captures(data).unwrap();
        assert_eq!(cpt.name("key").unwrap().as_str(), "db");
        assert_eq!(cpt.name("val").unwrap().as_str(), "/Users/rishabh/.briefs");
    }
}
