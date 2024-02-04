use crate::{post::Post, CatchupResult, StreamError};
use std::{fmt::Display, io::Write, time::SystemTime};

/// A Stream struct contains all the posts and some metadata.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Stream {
    posts: Vec<Post>,
    size: usize,
    last_updated: SystemTime,
    date_of_inception: SystemTime,
}

impl Default for Stream {
    fn default() -> Self {
        Stream {
            posts: Vec::with_capacity(50),
            size: 0,
            date_of_inception: SystemTime::now(),
            last_updated: SystemTime::now(),
        }
    }
}

impl Stream {
    /// Adds a new post to the current stream.
    pub fn add_post(&mut self, post: Post) -> CatchupResult<()> {
        self.increase_capacity()?;
        self.posts.push(post);
        self.size += 1;
        self.last_updated = SystemTime::now();
        Ok(())
    }

    /// Removes an existing post from the stream.
    pub fn remove_post(&mut self, index: usize) -> CatchupResult<()> {
        let post_index = self.get_post_index(&index)?;
        self.posts.remove(post_index);
        Ok(())
    }

    /// Update an existing post with the new message.
    pub fn update_msg(&mut self, index: usize, new_msg: String) -> CatchupResult<()> {
        let post_index = self.get_post_index(&index)?;
        let post = self.posts.get_mut(post_index).unwrap();
        Ok(post.update_msg(new_msg)?)
    }

    /// Update an existing post with the new title.
    pub fn update_title(&mut self, id: usize, new_title: String) -> CatchupResult<()> {
        let post_index = self.get_post_index(&id)?;
        let post = self.posts.get_mut(post_index).unwrap();
        Ok(post.update_title(new_title)?)
    }

    /// Get the index of a post in `posts`. The argument specifies
    /// the index of the post from the last post. This return the index from
    /// the start.
    fn get_post_index(&self, index: &usize) -> CatchupResult<usize> {
        let posts_count = self.posts.len();
        if *index > posts_count {
            return Err(StreamError::InvalidIndex {
                posts_count,
                given_index: *index,
            }
            .into());
        }
        Ok(posts_count - index)
    }

    /// Get the last time the stream was updated
    pub fn last_updated(&self) -> SystemTime {
        self.last_updated.clone()
    }

    /// Get the number of posts in stream
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get the date of inception/creation of the stream
    pub fn date_of_inception(&self) -> SystemTime {
        self.date_of_inception
    }

    /// Increase the capacity of the stream by 50
    fn increase_capacity(&mut self) -> CatchupResult<()> {
        if self.posts.capacity() <= 10 {
            return Ok(());
        };
        Ok(self.posts.try_reserve(50)?)
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

impl Stream {
    pub fn catchup(
        &self,
        start_index: usize,
        mut end_index: usize,
        f: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        end_index = if self.size() < end_index {
            self.size()
        } else {
            end_index
        };
        for idx in start_index..end_index {
            writeln!(f, "{}", self.posts[idx])?;
        }
        Ok(())
    }
}
