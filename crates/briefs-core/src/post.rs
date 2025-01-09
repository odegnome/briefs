//! This module defines the `Post` struct which is the heart of CatchUP!

use crate::{constant, BriefsResult, BriefsError};
use std::fmt::{Display, Formatter};
use std::time::SystemTime;
use textwrap::{self, wrap};

/// Every time a new post is created by the admin,
/// this is the struct that stores all the necessary data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Post {
    id: usize,
    pub title: String,
    pub msg: String,
    pub date: u64,
    pub edited: bool,
}

impl Post {
    /// Create a new post by providing the `title` and the body
    /// of the message in `msg`.
    pub fn new(id: usize, title: String, msg: String) -> BriefsResult<Self> {
        verify_title(&title)?;
        verify_msg(&msg)?;
        Ok(Post {
            id,
            title,
            msg,
            date: time_in_sec(SystemTime::now())?,
            edited: false,
        })
    }

    /// Update the message of an existing post.
    pub fn update_msg(&mut self, new_msg: String) -> BriefsResult<()> {
        verify_msg(&new_msg)?;
        self.msg = new_msg;
        self.edited = true;
        Ok(())
    }

    /// Update the title of an existing post.
    pub fn update_title(&mut self, new_title: String) -> BriefsResult<()> {
        verify_title(&new_title)?;
        self.title = new_title;
        self.edited = true;
        Ok(())
    }

    pub fn id(&self) -> BriefsResult<usize> {
        Ok(self.id)
    }
}

pub(crate) fn time_in_sec(time: SystemTime) -> BriefsResult<u64> {
    Ok(time.duration_since(std::time::UNIX_EPOCH)?.as_secs())
}

/// Some necessary checks for post's title.
fn verify_title(title: &String) -> BriefsResult<()> {
    if title.is_empty() {
        return Err(BriefsError::EmptyTitle.into());
    }
    if title.len() > (constant::MAX_POST_TITLE as usize) {
        return Err(BriefsError::InvalidTitleLength {
            max_size: constant::MAX_POST_TITLE as usize,
            curr_size: title.len(),
        }
        .into());
    }
    Ok(())
}

/// Some necessary checks for post's message.
fn verify_msg(msg: &String) -> BriefsResult<()> {
    // check min/max length of post
    if msg.is_empty() {
        return Err(BriefsError::EmptyPost.into());
    }
    if msg.len() > constant::MAX_POST_LEN as usize {
        return Err(BriefsError::InvalidPostLength {
            max_size: constant::MAX_POST_LEN as usize,
            curr_size: msg.len(),
        }
        .into());
    }
    Ok(())
}

impl Display for Post {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:-<54}\n", "")?;
        write!(f, "\\ {:^50} /\n/ {:50} \\\n", self.title, "")?;
        let mut count = 0u8;
        let wrapping_config = textwrap::Options::new(50).break_words(true);
        for line in wrap(&format!("{}\n", self.msg), wrapping_config) {
            let (left_closure, right_closure) = if count % 2 == 0 {
                ("\\ ", " /")
            } else {
                ("/ ", " \\")
            };
            write!(f, "{left_closure}{:*<50}{right_closure}\n", line)?;
            count += 1;
        }
        write!(f, "{:-<54}", "")
    }
}
