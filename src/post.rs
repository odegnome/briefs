use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
    rc::Rc,
};
use textwrap::wrap;

/// A wrapper for Post object. Used to hold the actual post and pointers to the
/// adjoining posts.
#[derive(Debug, Clone)]
pub struct StreamPost {
    /// pointer to the post object stored in heap
    pub post: Box<Post>,
    /// next here actually refers to the previous post
    pub next: Option<Rc<RefCell<StreamPost>>>,
    /// prev here actually refers to the next post
    pub prev: Option<Rc<RefCell<StreamPost>>>,
}

impl StreamPost {
    pub fn lone(post: Post) -> StreamPost {
        StreamPost {
            post: Box::new(post),
            next: None,
            prev: None,
        }
    }
}

/// Post struct which is the heart of this project.
#[derive(Debug, Clone)]
pub struct Post {
    pub title: String,
    pub msg: String,
    pub date: String,
}

impl Post {
    pub fn new(title: String, msg: String, date: String) -> Self {
        Post { title, msg, date }
    }
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
