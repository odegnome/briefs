use crate::{
    post::{Post, StreamPost},
    StreamError,
};
use std::{cell::RefCell, fmt::Display, rc::Rc, time::SystemTime};

#[derive(Debug, Clone)]
pub struct Stream {
    head: Rc<RefCell<StreamPost>>,
    size: usize,
    last_updated: SystemTime,
}

impl Stream {
    /// Creates a new Stream. This should practically only be used once.
    pub fn new(post: Post) -> Self {
        Stream {
            head: Rc::new(RefCell::new(StreamPost::lone(post))),
            size: 1,
            last_updated: SystemTime::now(),
        }
    }

    /// Adds a new post to the current stream.
    pub fn add_post(&mut self, post: Post) -> Result<(), StreamError> {
        let mut stream_post = StreamPost::lone(post);
        stream_post.next = Some(Rc::clone(&self.head));
        let new_head = Rc::new(RefCell::new(stream_post));
        self.head.borrow_mut().prev = Some(Rc::clone(&new_head));
        self.head = new_head;
        self.size += 1;
        self.last_updated = SystemTime::now();
        Ok(())
    }

    pub fn last_updated(&self) -> SystemTime {
        self.last_updated.clone()
    }

    pub fn size(&self) -> usize {
        self.size.clone()
    }
}

impl Display for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", &self.head.borrow().post)?;
        // Iterate over the remaining posts
        if self.head.borrow().next.is_none() {
            return writeln!(f, "\n{:^54}", "End of Stream");
        };
        let mut ptr = self.head.borrow().next.clone();
        while ptr.is_some() {
            let _post = Rc::clone(&ptr.clone().unwrap());
            writeln!(f, "{}", &_post.borrow().post)?;
            ptr = _post.borrow().next.clone();
        }
        writeln!(f, "\n{:^54}", "End of Stream")
    }
}
