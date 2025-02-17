use std::io::Write;

use anyhow::ensure;
use sqlite::Connection;

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

    let mut content = Vec::with_capacity(16);
    content.extend(stream.last_updated().to_be_bytes());
    content.extend(stream.date_of_inception().to_be_bytes());

    // Write data
    fileptr.write_all(content.as_slice())?;

    Ok(())
}

pub fn read_stream_from_disk(
    conn: &mut Connection,
    config: &BriefsConfig,
) -> anyhow::Result<Stream> {
    let data_dir = config.dirpath.join(DATA_DIR);
    if !std::fs::exists(&data_dir)? {
        return Err(BriefsError::utils_error("Data directory does not exist".into()).into());
    }
    let stream_file = data_dir.join(DATA_FILE);
    if !std::fs::exists(&stream_file)? {
        return Err(BriefsError::utils_error("Stream data does not exist".into()).into());
    }

    let cache = std::fs::read(stream_file)?;

    ensure!(
        cache.len() == 16,
        BriefsError::utils_error("Incorrect cache len".into())
    );

    let mut u64_chunks = cache.chunks_exact(8);
    let mut u64_barray = [0u8; 8];
    u64_barray.copy_from_slice(
        u64_chunks
            .next()
            .ok_or(BriefsError::utils_error("Cannot parse doi chunk".into()))?,
    );
    let last_updated = u64::from_be_bytes(u64_barray);
    u64_barray.copy_from_slice(
        u64_chunks
            .next()
            .ok_or(BriefsError::utils_error("Cannot parse doi chunk".into()))?,
    );
    let doi = u64::from_be_bytes(u64_barray);

    Ok(Stream::assemble(conn, last_updated, doi)?)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{constant::CONFIG_FILE, db::test::setup_mock_db};
    use rand::{prelude::Distribution, thread_rng};

    const CONFIG_DIR: &str = "briefs";

    // Create default stream and config. Also, dirpath in briefsconfig
    // will certainly exist. The dirpath will be random, in order to allow
    // test cases to be run concurrently.
    fn get_mocks() -> (Stream, BriefsConfig) {
        let tmp_dir = std::env::temp_dir();
        let stream = Stream::default();
        let mut config = BriefsConfig::default();

        let mut dirname = String::from(CONFIG_DIR);
        let gibberish: Vec<String> = rand::distributions::uniform::Uniform::<u8>::new(0, 10)
            .sample_iter(thread_rng())
            .map(|val| val.to_string())
            .take(5)
            .collect();
        dirname.extend(gibberish);

        let dirpath = tmp_dir.join(dirname);
        println!("{:?}", dirpath);
        let filepath = dirpath.join(CONFIG_FILE);
        println!("{:?}", filepath);

        let error = "Error creating mocks";
        std::fs::create_dir_all(&dirpath).expect(error);
        assert!(std::fs::exists(&dirpath).expect("WTF?"));
        std::fs::File::create(&filepath).expect(error);

        let error = "Error setting filepath";
        config.set_filepath(filepath).expect(error);

        (stream, config)
    }

    fn cleanup(dirpath: PathBuf) {
        assert!(dirpath.is_dir());

        std::fs::remove_dir_all(dirpath).unwrap();
    }

    /// Returns last_updated & date_of_inception, which are u64 in stream.
    fn read_stream_byte_cache(file: PathBuf) -> (u64, u64) {
        let cache = std::fs::read(file).unwrap();

        assert!(
            cache.len() == 16,
            "{:?}",
            BriefsError::utils_error("Incorrect cache len".into())
        );

        let mut u64_chunks = cache.chunks_exact(8);
        let mut u64_barray = [0u8; 8];
        u64_barray.copy_from_slice(
            u64_chunks
                .next()
                .ok_or(BriefsError::utils_error("Cannot parse doi chunk".into()))
                .unwrap(),
        );
        let last_updated = u64::from_be_bytes(u64_barray);
        u64_barray.copy_from_slice(
            u64_chunks
                .next()
                .ok_or(BriefsError::utils_error("Cannot parse doi chunk".into()))
                .unwrap(),
        );
        let doi = u64::from_be_bytes(u64_barray);

        (last_updated, doi)
    }

    #[test]
    fn test_save_stream_on_disk() {
        let (stream, config) = get_mocks();

        save_stream_on_disk(&stream, &config).unwrap();

        let stream_dir = config.dirpath.join(DATA_DIR);
        let stream_file = stream_dir.join(DATA_FILE);
        assert!(stream_dir.exists());
        assert!(stream_file.exists());

        let (last_updated, doi) = read_stream_byte_cache(stream_file.clone());
        assert_eq!(stream.last_updated(), last_updated);
        assert_eq!(stream.date_of_inception(), doi);

        cleanup(config.dirpath);
    }

    #[test]
    fn test_basic_read_stream_from_disk() {
        let (stream, config) = get_mocks();

        save_stream_on_disk(&stream, &config).unwrap();

        let stream_dir = config.dirpath.join(DATA_DIR);
        let stream_file = stream_dir.join(DATA_FILE);
        assert!(stream_dir.exists());
        assert!(stream_file.exists());

        let mock_db = setup_mock_db();
        let mut conn = sqlite::open(&mock_db).unwrap();

        let dskstream = read_stream_from_disk(&mut conn, &config).unwrap();

        assert_eq!(dskstream.last_updated(), stream.last_updated());
        assert_eq!(dskstream.date_of_inception(), stream.date_of_inception());
        assert_eq!(dskstream.size(), stream.size());
        assert_eq!(dskstream.nposts(), stream.nposts());

        cleanup(config.dirpath);

    }
}
