use sqlite::Connection;

use crate::{
    constant::STREAM_CACHE_SIZE,
    db,
    post::{time_in_sec, Post},
    state::{CatchUpResponse, StreamMetadata},
    BriefsError, BriefsResult,
};
use std::{collections::VecDeque, fmt::Display, time::SystemTime};

/// A Stream contains all the posts and some metadata.
#[derive(Debug)]
pub struct Stream {
    posts: VecDeque<Post>,
    size: usize,
    last_updated: u64,
    date_of_inception: u64,
}

impl Default for Stream {
    fn default() -> Self {
        Stream {
            posts: VecDeque::with_capacity(STREAM_CACHE_SIZE.into()),
            size: 0,
            date_of_inception: time_in_sec(SystemTime::now()).unwrap(),
            last_updated: time_in_sec(SystemTime::now()).unwrap(),
        }
    }
}

impl Stream {
    // ***
    // Command handlers
    // ***

    /// Adds a new post to the current stream.
    pub fn add_post(&mut self, conn: &mut Connection, post: Post) -> BriefsResult<()> {
        db::insert_post(conn, &post)?;
        if self.posts.len() == STREAM_CACHE_SIZE as usize {
            self.posts.pop_front();
            self.size -= 1;
        }
        self.posts.push_back(post);
        self.size += 1;
        self.last_updated = time_in_sec(SystemTime::now())?;
        Ok(())
    }

    /// Removes an existing post from the stream.
    pub fn remove_post(&mut self, conn: &mut Connection, id: usize) -> BriefsResult<()> {
        db::delete_post_by_id(conn, id)?;
        let idx = self.post_id_to_idx(id)?;
        self.posts.remove(idx);
        self.size = self.posts.len();
        self.last_updated = time_in_sec(SystemTime::now())?;
        Ok(())
    }

    /// Update an existing post with the new message.
    pub fn update_msg(
        &mut self,
        conn: &mut Connection,
        id: usize,
        new_msg: String,
    ) -> BriefsResult<()> {
        db::update_post_msg_by_id(conn, id, new_msg.clone())?;
        let post_id = self.post_id_to_idx(id)?;
        let post = self
            .posts
            .get_mut(post_id)
            .ok_or_else(|| BriefsError::InvalidId {})?;
        self.last_updated = time_in_sec(SystemTime::now())?;
        post.update_msg(new_msg)
    }

    /// Update an existing post with the new title.
    pub fn update_title(
        &mut self,
        conn: &mut Connection,
        id: usize,
        new_title: String,
    ) -> BriefsResult<()> {
        db::update_post_title_by_id(conn, id, new_title.clone())?;
        let post_id = self.post_id_to_idx(id)?;
        let post = self
            .posts
            .get_mut(post_id)
            .ok_or_else(|| BriefsError::InvalidId {})?;
        self.last_updated = time_in_sec(SystemTime::now())?;
        post.update_title(new_title)
    }

    /// Return the latest posts.
    #[allow(unused_assignments)]
    pub fn catchup(
        &self,
        conn: &Connection,
        start_index: usize,
        mut end_index: usize,
    ) -> BriefsResult<CatchUpResponse> {
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
    pub fn get_post(&self, conn: &Connection, id: usize) -> Option<&Post> {
        let result = self.post_id_to_idx(id);
        match result {
            Ok(post_idx) => self.posts.get(post_idx),
            Err(_) => None,
        }
    }

    pub fn stream_metadata(&self) -> BriefsResult<StreamMetadata> {
        Ok(StreamMetadata {
            posts_count: self.size(),
            last_updated: self.last_updated,
            latest_post_id: self.posts.back().map(|val| val.id().unwrap()),
        })
    }

    /// Refresh the internal cache from Db.
    pub fn refresh_cache(&mut self, conn: &mut Connection) -> BriefsResult<()> {
        self.posts.clear();
        let n_posts = db::query_last_n(conn, self.size.try_into()?)?;
        self.posts = db::sqlite_to_post(n_posts)?.into();
        Ok(())
    }

    // ***
    // Helpers
    // ***

    /// Get the index of a post in `posts`. The argument specifies
    /// the index of the post from the last post. This return the index from
    /// the start.
    #[allow(dead_code)]
    fn get_post_index(&self, index: &usize) -> BriefsResult<usize> {
        let posts_count = self.posts.len();
        if *index > posts_count {
            return Err(BriefsError::InvalidIndex {
                posts_count,
                given_index: *index,
            }
            .into());
        }
        Ok(posts_count - index)
    }

    /// Get the last time the stream was updated
    pub fn last_updated(&self) -> u64 {
        self.last_updated.clone()
    }

    /// Get the number of posts in stream
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get the date of inception/creation of the stream
    pub fn date_of_inception(&self) -> u64 {
        self.date_of_inception
    }

    /// Returns the index of post, with the associated ID, in the posts vector.
    fn post_id_to_idx(&self, id: usize) -> BriefsResult<usize> {
        let mut start = 0;
        let mut end = self.posts.len() - 1;
        let mut mid = (end + start) / 2;
        let mut post_id = self
            .posts
            .get(mid)
            .ok_or_else(|| BriefsError::InvalidId {})?
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
                .ok_or_else(|| BriefsError::InvalidId {})?
                .id()?;
        }

        if self
            .posts
            .get(mid)
            .ok_or_else(|| BriefsError::InvalidId {})?
            .id()?
            != id
        {
            return Err(BriefsError::InvalidId {}.into());
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
