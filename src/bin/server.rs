use tokio::sync::{oneshot, mpsc};
use catchup::{post, stream};

/// Used to send acknowledgements to the connection handler.
type Responder<T> = oneshot::Sender<T>;

pub enum Command {
    Catchup,
    Create { title: String, msg: String, resp: Responder<String> },
    Read { index: usize, resp: Responder<String> },
    Update,
    Delete { index: usize, resp: Responder<String> },
    Get,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(2);

    let stream_handle = tokio::spawn(async move {
        println!("Stream handle running...");
        let mut post_count = 0usize;
        let mut new_stream = stream::Stream::default();
        while let Some(request) = rx.recv().await {
            match request {
                Command::Create { title, msg, resp } => {
                    let post = post::Post::new(post_count + 1, title, msg);
                    if post.is_err() {
                        resp.send(
                            format!("ERROR during create: {:?}", post.unwrap_err())
                        )
                        .unwrap();
                        continue;
                    }
                    let result = new_stream.add_post(post.unwrap());
                    if result.is_err() {
                        resp.send(
                            format!("ERROR during create: {:?}", result.unwrap_err())
                        )
                        .unwrap();
                        continue;
                    }
                    resp.send(
                        format!("Succesfully added a new post")
                    ).unwrap();
                    post_count += 1;
                }

                Command::Catchup => {
                    println!("{}\n", &new_stream);
                }

                _ => eprintln!("Feature not implemented"),
            }

        }
    });

    let conn_handle = tokio::spawn(async move {
        println!("Connection handle running...");
        let (responder, receiver) = oneshot::channel();
        let cmd = Command::Create {
            title: String::from("Hello, World!"),
            msg: String::from(
                "This is my first post, and as is the tradition, \
                the post is titled 'Hello, World!'. Hopefully, this works!"),
            resp: responder
        };
        tx.send(cmd).await.unwrap();

        let res = receiver.await.unwrap();
        println!("CONN: {:?}", res);
        tokio::time::sleep(tokio::time::Duration::new(5,0)).await;

        let (responder, receiver) = oneshot::channel();
        let cmd = Command::Create {
            title: String::from("Another Hello, World!"),
            msg: String::from(
                "Nothing new in this post. It is simply copied from the last \
                post. This is my first post, and as is the tradition, \
                the post is titled 'Hello, World!'. Hopefully, this works!"),
            resp: responder
        };
        tx.send(cmd).await.unwrap();

        let res = receiver.await.unwrap();
        println!("CONN: {:?}", res);
        tokio::time::sleep(tokio::time::Duration::new(5,0)).await;

        println!("-------CatchUP-------");
        let cmd = Command::Catchup;
        tx.send(cmd).await.unwrap();
    });
    conn_handle.await.unwrap();
    stream_handle.await.unwrap();
}

