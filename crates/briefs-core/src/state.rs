use serde::{Deserialize, Serialize};

use crate::post::Post;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct CatchUpResponse {
    pub posts: Vec<Post>,
    pub caught_up: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct StreamMetadata {
    pub latest_post_id: Option<usize>,
    pub last_updated: u64,
    pub posts_count: usize,
}
