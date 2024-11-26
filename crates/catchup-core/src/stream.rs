use crate::{
    constant::STREAM_CACHE_SIZE,
    post::Post,
    state::{CatchUpResponse, StreamMetadata},
    CatchupResult, StreamError,
};
use std::{
    collections::VecDeque,
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

/// A Stream contains all the posts and some metadata.
#[derive(Debug)]
pub struct Stream {
    posts: VecDeque<Post>,
    size: usize,
    last_updated: SystemTime,
    date_of_inception: SystemTime,
}

fn systime_to_u64(time: &SystemTime) -> CatchupResult<u64> {
    Ok(time.duration_since(UNIX_EPOCH)?.as_secs())
}

impl Default for Stream {
    fn default() -> Self {
        Stream {
            posts: VecDeque::with_capacity(STREAM_CACHE_SIZE.into()),
            size: 0,
            date_of_inception: SystemTime::now(),
            last_updated: SystemTime::now(),
        }
    }
}

impl Stream {
    // ***
    // Command handlers
    // ***

    /// Adds a new post to the current stream.
    pub fn add_post(&mut self, post: Post) -> CatchupResult<()> {
        self.increase_capacity()?;
        if self.posts.len() == STREAM_CACHE_SIZE.into() {
            self.posts.pop_front();
            self.size -= 1;
        }
        self.posts.push_back(post);
        self.size += 1;
        self.last_updated = SystemTime::now();
        Ok(())
    }

    /// Removes an existing post from the stream.
    pub fn remove_post(&mut self, id: usize) -> CatchupResult<()> {
        let idx = self.post_id_to_idx(id)?;
        self.posts.remove(idx);
        self.size = self.posts.len();
        self.last_updated = SystemTime::now();
        Ok(())
    }

    /// Update an existing post with the new message.
    pub fn update_msg(&mut self, id: usize, new_msg: String) -> CatchupResult<()> {
        let post_id = self.post_id_to_idx(id)?;
        let post = self
            .posts
            .get_mut(post_id)
            .ok_or_else(|| StreamError::InvalidId {})?;
        self.last_updated = SystemTime::now();
        post.update_msg(new_msg)
    }

    /// Update an existing post with the new title.
    pub fn update_title(&mut self, id: usize, new_title: String) -> CatchupResult<()> {
        let post_id = self.post_id_to_idx(id)?;
        let post = self
            .posts
            .get_mut(post_id)
            .ok_or_else(|| StreamError::InvalidId {})?;
        self.last_updated = SystemTime::now();
        post.update_title(new_title)
    }

    /// Return the latest posts.
    #[allow(unused_assignments)]
    pub fn catchup(
        &self,
        start_index: usize,
        mut end_index: usize,
    ) -> CatchupResult<CatchUpResponse> {
        let mut caught_up = false;
        end_index = if self.size() <= end_index {
            caught_up = true;
            self.size()
        } else {
            end_index
        };
        if self
            .posts
            .binary_search_by_key(&start_index, |val| val.id().unwrap())
            .is_err()
        {
            // !-------
            // Fetch result from the db
            // -------!
            todo!()
        }
        let response = CatchUpResponse {
            posts: self.posts.clone().into(),
            caught_up,
        };
        Ok(response)
    }

    /// Return a specific post.
    pub fn get_post(&self, id: usize) -> Option<&Post> {
        let result = self.post_id_to_idx(id);
        match result {
            Ok(post_idx) => self.posts.get(post_idx),
            Err(_) => None,
        }
    }

    pub fn stream_metadata(&self) -> CatchupResult<StreamMetadata> {
        Ok(StreamMetadata {
            posts_count: self.size(),
            last_updated: systime_to_u64(&self.last_updated)?,
            latest_post_id: self.posts.back().map(|val| val.id().unwrap()),
        })
    }

    // ***
    // Helpers
    // ***
    
    pub fn refresh_cache(&mut self) -> CatchupResult<()> {
        self.posts.clear();
        todo!();
        Ok(())
    }

    /// Get the index of a post in `posts`. The argument specifies
    /// the index of the post from the last post. This return the index from
    /// the start.
    #[allow(dead_code)]
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
        Ok(self.posts.try_reserve(STREAM_CACHE_SIZE.into())?)
    }

    /// Returns the index of post, with the associated ID, in the posts vector.
    fn post_id_to_idx(&self, id: usize) -> CatchupResult<usize> {
        let mut start = 0;
        let mut end = self.posts.len() - 1;
        let mut mid = (end + start) / 2;
        let mut post_id = self
            .posts
            .get(mid)
            .ok_or_else(|| StreamError::InvalidId {})?
            .id()?;

        while start < end {
            if id < post_id {
                end = mid - 1;
            } else if id > post_id {
                start = mid + 1;
            } else {
                break;
            }
            mid = (end + start) / 2;
            post_id = self
                .posts
                .get(mid)
                .ok_or_else(|| StreamError::InvalidId {})?
                .id()?;
        }

        if self
            .posts
            .get(mid)
            .ok_or_else(|| StreamError::InvalidId {})?
            .id()?
            != id
        {
            return Err(StreamError::InvalidId {}.into());
        }

        Ok(mid)
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
