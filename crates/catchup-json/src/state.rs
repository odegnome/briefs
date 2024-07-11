use serde::{Deserialize, Serialize};

use core::post::Post;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Catchup { last_fetch_id: usize },
    NewPost { title: String, msg: String },
    Read { id: isize },
    Update { id: usize },
    Delete { id: usize },
    Subscribe {},
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct CatchUpResponse {
    pub posts: Vec<Post>,
    pub caught_up: bool,
}
