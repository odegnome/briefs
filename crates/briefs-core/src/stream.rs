use sqlite::Connection;

use crate::{
    constant::{PAGINATION_DEFAULT, PAGINATION_LIMIT, STREAM_CACHE_SIZE},
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
    nposts: u64,
    last_updated: u64,
    date_of_inception: u64,
}

impl Default for Stream {
    fn default() -> Self {
        Stream {
            posts: VecDeque::with_capacity(STREAM_CACHE_SIZE.into()),
            size: 0,
            nposts: 0,
            date_of_inception: time_in_sec(SystemTime::now()).unwrap(),
            last_updated: time_in_sec(SystemTime::now()).unwrap(),
        }
    }
}

impl Stream {
    pub fn assemble(conn: &mut Connection, last_updated: u64, doi: u64) -> BriefsResult<Self> {
        println!("» Assembling existing stream");
        let records = db::query_cache(conn)?;
        let post_iter = db::sqlite_to_post(records)?.into_iter().rev();
        let posts = VecDeque::from_iter(post_iter);
        let size = posts.len();
        println!("» Found {} sqlite rows", size);
        let result = db::query_post_count(conn)?.take("count");

        let nposts = match result {
            sqlite::Value::Integer(val) => val.try_into()?,
            _ => return Err(BriefsError::custom_error("Post count not an integer".into()).into()),
        };

        Ok(Stream {
            posts,
            size,
            nposts,
            last_updated,
            date_of_inception: doi,
        })
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
        self.increment_post_count()?;
        Ok(())
    }

    /// Removes an existing post from the stream.
    pub fn remove_post(&mut self, conn: &mut Connection, id: u32) -> BriefsResult<()> {
        db::delete_post_by_id(conn, id)?;
        self.last_updated = time_in_sec(SystemTime::now())?;
        self.decrement_post_count()?;
        if !self.id_in_cache(id) {
            return Ok(());
        }
        let idx = self.post_id_to_idx(id)?;
        self.posts.remove(idx);
        self.size = self.posts.len();
        Ok(())
    }

    /// Update an existing post with the new message.
    pub fn update_msg(
        &mut self,
        conn: &mut Connection,
        id: u32,
        new_msg: String,
    ) -> BriefsResult<()> {
        db::update_post_msg_by_id(conn, id, new_msg.clone())?;
        self.last_updated = time_in_sec(SystemTime::now())?;
        if !self.id_in_cache(id) {
            return Ok(());
        }
        let idx = self.post_id_to_idx(id)?;
        let post = self
            .posts
            .get_mut(idx)
            .ok_or_else(|| BriefsError::InvalidId {})?;
        post.update_msg(new_msg)
    }

    /// Update an existing post with the new title.
    pub fn update_title(
        &mut self,
        conn: &mut Connection,
        id: u32,
        new_title: String,
    ) -> BriefsResult<()> {
        db::update_post_title_by_id(conn, id, new_title.clone())?;
        self.last_updated = time_in_sec(SystemTime::now())?;
        if !self.id_in_cache(id) {
            return Ok(());
        }
        let post_id = self.post_id_to_idx(id)?;
        let post = self
            .posts
            .get_mut(post_id)
            .ok_or_else(|| BriefsError::InvalidId {})?;
        post.update_title(new_title)
    }

    /// Return latest posts since last fetch.
    pub fn catchup(
        &self,
        conn: &Connection,
        sid: u32,
        limit: Option<u32>,
    ) -> BriefsResult<CatchUpResponse> {
        let mut response = CatchUpResponse {
            posts: Vec::new(),
            caught_up: true,
        };
        if self.posts.is_empty() {
            return Ok(response);
        }

        let mut caught_up = false;
        let last_id = self.posts.back().unwrap().id().unwrap();

        if last_id < sid {
            return Ok(response);
        }

        let mut lmt = limit.unwrap_or(PAGINATION_DEFAULT);
        lmt = std::cmp::min(lmt, PAGINATION_LIMIT);
        let mut eid = sid + lmt;
        eid = if last_id <= eid {
            caught_up = true;
            last_id
        } else {
            eid
        };
        if self.id_in_cache(sid) {
            println!("» Fetching from cache");
            // use cache
            let sidx = self.post_id_to_idx(sid)?;
            let eidx = self.post_id_to_idx(eid)?;
            response.posts = self.posts.range(sidx..=eidx).cloned().collect();
            response.caught_up = caught_up;
            return Ok(response);
        }
        // use db
        let records = db::catchup(conn, sid.try_into()?, eid.try_into()?, lmt)?;
        response.posts = db::sqlite_to_post(records)?;
        response.caught_up = caught_up;
        Ok(response)
    }

    /// Return a specific post.
    pub fn get_post(&self, conn: &Connection, id: u32) -> Option<Post> {
        let result = self.post_id_to_idx(id);
        match result {
            Ok(idx) => self.posts.get(idx).cloned(),
            Err(_) => match db::query_post_by_id(conn, id) {
                Ok(val) => Post::parse_sqlite_row(val).ok(),
                Err(_) => None,
            },
        }
    }

    pub fn stream_metadata(&self) -> BriefsResult<StreamMetadata> {
        Ok(StreamMetadata {
            posts_count: self.nposts as u32,
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

    fn increment_post_count(&mut self) -> BriefsResult<()> {
        self.nposts += 1;
        Ok(())
    }

    fn decrement_post_count(&mut self) -> BriefsResult<()> {
        self.nposts = self.nposts.saturating_sub(1);
        Ok(())
    }

    fn id_in_cache(&self, index: u32) -> bool {
        if self.posts.is_empty() {
            false
        } else if index < self.posts.front().unwrap().id().unwrap_or_default() {
            false
        } else if index > self.posts.back().unwrap().id().unwrap_or_default() {
            false
        } else {
            true
        }
    }

    /// Get the last time the stream was updated
    pub fn last_updated(&self) -> u64 {
        self.last_updated.clone()
    }

    /// Get the number of posts in cache
    pub fn size(&self) -> usize {
        self.size as usize
    }

    /// Get the number of posts in stream
    pub fn nposts(&self) -> usize {
        self.nposts as usize
    }

    /// Get the date of inception/creation of the stream
    pub fn date_of_inception(&self) -> u64 {
        self.date_of_inception
    }

    /// Returns the index of post, with the associated ID, in the posts vector.
    fn post_id_to_idx(&self, id: u32) -> BriefsResult<usize> {
        if self.posts.is_empty() {
            return Err(BriefsError::InvalidId {}.into());
        }
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
