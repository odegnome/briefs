use anyhow;

pub trait DbInsertString {
    fn db_insert_string(&self) -> anyhow::Result<String>;
}
