use crate::{post::Post, StreamError};
use std::{fmt::Display, time::SystemTime};

#[derive(Debug)]
pub struct Stream {
    posts: Vec<Post>,
    size: usize,
    last_updated: SystemTime,
}

impl Default for Stream {
    fn default() -> Self {
        Stream {
            posts: Vec::with_capacity(50),
            size: 1,
            last_updated: SystemTime::now(),
        }
    }
}

impl Stream {
    /// Adds a new post to the current stream
    pub fn add_post(&mut self, post: Post) -> Result<(), StreamError> {
        self.posts.push(post);
        Ok(())
    }

    /// Removes an existing post from the stream.
    pub fn remove_post(&mut self, index: usize) -> Result<(), StreamError> {
        let posts_count = self.posts.len();
        if index > posts_count {
            return Err(StreamError::InvalidIndex {
                posts_count,
                given_index: index,
            });
        };
        self.posts.remove(posts_count - index);
        Ok(())
    }

    /// Get the last time the stream was updated
    pub fn last_updated(&self) -> SystemTime {
        self.last_updated.clone()
    }

    /// Get the number of posts in stream
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn increase_capacity(&mut self) {
        if self.posts.capacity() >= 50 { return };
        self.posts.reserve(50);
    }
}

impl Display for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _post in self.posts.iter() {
            writeln!(f, "{}", _post)?;
        }
        Ok(())
    }
}
