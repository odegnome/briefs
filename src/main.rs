use catchup::{post::Post, stream::Stream};

#[allow(dead_code)]
fn post_demo() {
    let new_post = Post::new(
        String::from("My first post!"), 
        String::from("This is meant to be a small body for my first post. It is quite tiring to think of somehting to write. How did I do?"), 
        String::from("12-05-2023")
    );

    let another_post = Post::new(
        String::from("My second post!"), 
        String::from("This is meant to be a fairly big body for my second post. It is quite tiring to think of somehting to write. How did I do? Hopefully, the answert to that question is quite well. However, if it wasn't, then I have provided you another chance to correct your mistake. So, HOW DID I DO?"), 
        String::from("12-05-2023")
    );

    println!("{}", new_post);
    println!("{}", another_post);
}

fn main() {
    let new_post = Post::new(
        String::from("My first post!"), 
        String::from("This is meant to be a small body for my first post. It is quite tiring to think of somehting to write. How did I do?"), 
        String::from("12-05-2023")
    );

    let new_stream = Stream::new(new_post);

    println!("{:^54}", "Welcome to CatchUp");
    println!("\n{}", &new_stream);
}
