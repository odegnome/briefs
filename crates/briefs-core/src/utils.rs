use regex::Regex;
use std::io::Write;

use crate::{
    config::BriefsConfig,
    constant::{DATA_DIR, DATA_FILE},
    stream::Stream,
    BriefsError,
};

pub fn save_stream_on_disk(stream: &Stream, config: &BriefsConfig) -> anyhow::Result<()> {
    let data_dir = config.dirpath.join(DATA_DIR);
    if !std::fs::exists(&data_dir)? {
        std::fs::create_dir_all(data_dir.clone())?;
    }
    let stream_file = data_dir.join(DATA_FILE);
    let mut fileptr = std::fs::File::create(&stream_file)?;

    // Write data
    let content = format!(
        "!! Don't modify this file manually !!\
        \nsize={}\
        \nnposts={}\
        \nupdated={}\
        \ndoi={}\n",
        stream.size(),
        stream.nposts(),
        stream.last_updated(),
        stream.date_of_inception()
    );
    fileptr.write_all(content.as_bytes())?;

    Ok(())
}

pub fn read_stream_from_disk(config: &BriefsConfig) -> anyhow::Result<Stream> {
    let data_dir = config.dirpath.join(DATA_DIR);
    if !std::fs::exists(&data_dir)? {
        return Err(BriefsError::utils_error("Data directory does not exist".into()).into());
    }
    let stream_file = data_dir.join(DATA_FILE);
    if !std::fs::exists(&stream_file)? {
        return Err(BriefsError::utils_error("Stream data does not exist".into()).into());
    }

    let buf = std::fs::read_to_string(stream_file)?;
    let buf: Vec<&str> = buf.split('\n').collect();

    let mut doi: Option<u64> = None;
    let mut updated: Option<u64> = None;

    let pattern = Regex::new(r#"^(?<key>\w+) ?= ?['"]?(?<val>[0-9a-zA-Z.:/]*)['"]?.*$"#)?;
    for text in buf.into_iter().skip(1) {
        if let Some(matches) = pattern.captures(text) {
            match matches.name("key").map_or("", |val| val.as_str()) {
                "doi" => {
                    doi = matches
                        .name("val")
                        .map(|val| val.as_str().parse::<u64>().expect("Cannot parse 'doi'"))
                }
                "updated" => {
                    updated = matches
                        .name("val")
                        .map(|val| val.as_str().parse::<u64>().expect("Cannot parse 'updated'"))
                }

                _ => {
                    return Err(
                        BriefsError::config_error("Parsing error: key not found".into()).into(),
                    )
                }
            }
        }
    }

    if doi.is_none() || updated.is_none() {
        return Err(
            BriefsError::utils_error("Required values not found in stream file".into()).into(),
        );
    }

    Ok(Stream::assemble(updated.unwrap(), doi.unwrap())?)
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use regex::Regex;

    use super::*;
    use crate::constant::{CONFIG_DIR, CONFIG_FILE};

    // Create default stream and config. Also, dirpath in briefsconfig
    // will certainly exist.
    fn get_mocks() -> (Stream, BriefsConfig) {
        let tmp_dir = std::env::temp_dir();
        let stream = Stream::default();
        let mut config = BriefsConfig::default();

        let error = "Error creating mocks for testing";
        config
            .set_filepath(tmp_dir.join(CONFIG_DIR).join(CONFIG_FILE))
            .expect(error);

        std::fs::create_dir_all(&config.dirpath).expect(error);
        std::fs::File::create(&config.filepath).expect(error);

        (stream, config)
    }

    fn get_regex_pattern() -> Regex {
        Regex::new(r#"^(?<key>\w+) ?= ?['"]?(?<val>[0-9a-zA-Z.:/]*)['"]?.*$"#).unwrap()
    }

    #[test]
    fn test_save_stream_on_disk() {
        let (stream, config) = get_mocks();

        save_stream_on_disk(&stream, &config).unwrap();

        let stream_dir = config.dirpath.join(DATA_DIR);
        let stream_file = stream_dir.join(DATA_FILE);
        assert!(stream_dir.exists());
        assert!(stream_file.exists());

        let mut fptr = std::fs::File::open(stream_file).unwrap();
        let mut buf = String::new();
        fptr.read_to_string(&mut buf).unwrap();

        panic!("Incomplete")
    }
}
