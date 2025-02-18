//! This module defines the `Post` struct which is the heart of CatchUP!

use crate::{constant, BriefsError, BriefsResult};
use std::fmt::{Display, Formatter};
use std::time::SystemTime;
use textwrap::core::display_width;
use textwrap::{self, wrap};

/// Every time a new post is created by the admin,
/// this is the struct that stores all the necessary data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Post {
    id: u32,
    pub title: String,
    pub msg: String,
    pub date: u64,
    pub edited: bool,
}

impl Post {
    /// Create a new post by providing the `title` and the body
    /// of the message in `msg`.
    pub fn new(id: u32, title: String, msg: String) -> BriefsResult<Self> {
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

    pub fn id(&self) -> BriefsResult<u32> {
        Ok(self.id)
    }

    pub fn parse_sqlite_row(mut record: sqlite::Row) -> BriefsResult<Self> {
        let mut post = Post {
            id: 0,
            title: String::new(),
            msg: String::new(),
            date: 0,
            edited: false,
        };

        match record.take("id") {
            sqlite::Value::Integer(val) => post.id = val.try_into()?,
            _ => return Err(BriefsError::SqliteValueParseError.into()),
        };

        match record.take("title") {
            sqlite::Value::String(val) => post.title = val,
            _ => return Err(BriefsError::SqliteValueParseError.into()),
        };

        match record.take("msg") {
            sqlite::Value::String(val) => post.msg = val,
            _ => return Err(BriefsError::SqliteValueParseError.into()),
        };

        match record.take("date") {
            sqlite::Value::Integer(val) => post.date = val.try_into()?,
            _ => return Err(BriefsError::SqliteValueParseError.into()),
        };

        match record.take("edited") {
            sqlite::Value::Integer(val) => post.edited = val != 0, // 0: false, 1: true; in sqlite
            _ => return Err(BriefsError::SqliteValueParseError.into()),
        };

        Ok(post)
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
        let content_width = 50;
        let wrapping_config = textwrap::Options::new(content_width).break_words(true);
        for line in wrap(&format!("{}\n", self.msg), wrapping_config) {
            let (left_closure, right_closure) = if count % 2 == 0 {
                ("\\ ", " /")
            } else {
                ("/ ", " \\")
            };
            let text_width = display_width(&line);
            let whitespace = if content_width >= text_width {
                content_width - text_width
            } else {
                0
            };
            write!(
                f,
                "{left_closure}{}{}{right_closure}\n",
                line,
                " ".repeat(whitespace)
            )?;
            count += 1;
        }
        write!(f, "{:-<54}", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_formatting_using_display() {
        let post = Post::new(
            0,
            String::from("First Post"),
            String::from("This is a demo post with emojis to test formatting 😃😃"),
        )
        .unwrap();
        println!("{}", post);

        let post = Post::new(
            0,
            String::from("First Post"),
            String::from("This is a demo post with emojis to test ►→℞+ formatting 😃😃"),
        )
        .unwrap();
        println!("{}", post);
    }
}
