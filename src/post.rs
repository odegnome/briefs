use crate::{constant, CatchupResult, StreamError};
use std::fmt::{Display, Formatter};
use std::time::SystemTime;
use textwrap::wrap;

/// Post struct which is the heart of this project.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Post {
    id: usize,
    pub title: String,
    pub msg: String,
    pub date: SystemTime,
    pub edited: bool,
}

impl Post {
    /// Create a new post by providing the `title` and the body
    /// of the message in `msg`.
    pub fn new(id: usize, title: String, msg: String) -> CatchupResult<Self> {
        verify_title(&title)?;
        verify_msg(&msg)?;
        Ok(Post {
            id,
            title,
            msg,
            date: SystemTime::now(),
            edited: false,
        })
    }

    /// Update the message of an existing post.
    pub fn update_msg(&mut self, new_msg: String) -> CatchupResult<()> {
        verify_msg(&new_msg)?;
        self.msg = new_msg;
        self.edited = true;
        Ok(())
    }

    /// Update the title of an existing post.
    pub fn update_title(&mut self, new_title: String) -> CatchupResult<()> {
        verify_title(&new_title)?;
        self.title = new_title;
        self.edited = true;
        Ok(())
    }

}

/// Some necessary checks for post's title.
fn verify_title(title: &String) -> CatchupResult<()> {
    if title.is_empty() {
        return Err(StreamError::EmptyTitle.into());
    }
    if title.len() > (constant::MAX_POST_TITLE as usize) {
        return Err(StreamError::InvalidTitleLength {
            max_size: constant::MAX_POST_TITLE as usize,
            curr_size: title.len(),
        }.into());
    }
    Ok(())
}

/// Some necessary checks for post's message.
fn verify_msg(msg: &String) -> CatchupResult<()> {
    // check min/max length of post
    if msg.is_empty() {
        return Err(StreamError::EmptyPost.into());
    }
    if msg.len() > constant::MAX_POST_LEN as usize {
        return Err(StreamError::InvalidPostLength {
            max_size: constant::MAX_POST_LEN as usize,
            curr_size: msg.len(),
        }.into());
    }
    Ok(())
}

impl Display for Post {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:-<54}\n", "")?;
        write!(f, "\\ {:^50} /\n/ {:50} \\\n", self.title, "")?;
        let mut count = 0u8;
        for line in wrap(&format!("{}\n", self.msg), 50) {
            let (left_closure, right_closure) = if count % 2 == 0 {
                ("\\ ", " /")
            } else {
                ("/ ", " \\")
            };
            write!(f, "{left_closure}{: <50}{right_closure}\n", line)?;
            count += 1;
        }
        write!(f, "{:-<54}", "")
    }
}
