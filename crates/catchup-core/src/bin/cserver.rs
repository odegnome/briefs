#![allow(unused_imports)]
use catchup_core::{
    post,
    stream::{self, Stream},
    Command, StreamCommand,
};
use std::net::SocketAddr;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

/// 10Kb buffer
const BUFFER_SIZE: usize = 10240;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(16);

    let stream_handle = tokio::spawn(async move {
        println!("Stream handle running...");
        let mut stream = stream::Stream::default();
        while let Some(StreamCommand { cmd, resp }) = rx.recv().await {
            match cmd {
                Command::Create { title, msg } => {
                    let post = post::Post::new(stream.size(), title, msg);
                    if post.is_err() {
                        resp.unwrap()
                            .send(format!("ERROR during create: {:?}", post.unwrap_err()))
                            .unwrap();
                        continue;
                    }
                    let result = stream.add_post(post.unwrap());
                    if result.is_err() {
                        resp.unwrap()
                            .send(format!("ERROR during create: {:?}", result.unwrap_err()))
                            .unwrap();
                        continue;
                    }
                    resp.unwrap()
                        .send(format!("Succesfully added a new post"))
                        .unwrap();
                }

                Command::Catchup { last_fetch_id } => {
                    if stream.size() == 0 {
                        resp.unwrap().send(format!("No posts yet")).unwrap();
                        continue;
                    };
                    if last_fetch_id >= stream.size() {
                        resp.unwrap().send(format!("Caught up!")).unwrap();
                        continue;
                    }
                    let uncaught_posts = stream.size() - 1 - last_fetch_id;
                    let limit_index = if uncaught_posts > 10 {
                        last_fetch_id + 11
                    } else {
                        stream.size()
                    };
                    let mut print_buffer = Vec::new();
                    let result = stream.catchup(last_fetch_id, limit_index, &mut print_buffer);
                    if result.is_err() {
                        resp.unwrap()
                            .send(format!("An error occured: {:?}", result.unwrap_err()))
                            .unwrap();
                        continue;
                    }
                    resp.unwrap()
                        .send(format!("{}", String::from_utf8(print_buffer).unwrap()))
                        .unwrap();

                    //resp.unwrap().send(format!("{}", &stream)).unwrap();
                }

                _ => eprintln!("Feature not implemented"),
            }
        }
    });

    let conn_handle = tokio::spawn(async move {
        println!("Connection handle running...");

        // !------- ACCEPT CONNECTIONS ON PORT 8080 -------!
        let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
        println!("Listening on {}...", listener.local_addr().unwrap());

        loop {
            let _tx = tx.clone();
            let conn = listener.accept().await;
            if conn.is_ok() {
                tokio::spawn(async move {
                    handle_request(conn.unwrap(), _tx).await;
                });
            }
        }
    });
    conn_handle.await.unwrap();
    stream_handle.await.unwrap();
}

async fn handle_request(mut conn: (TcpStream, SocketAddr), tx: mpsc::Sender<StreamCommand>) {
    println!("Succesfully connected with {:?}", conn.1);

    conn.0.readable().await.unwrap();

    let mut kb_buffer = [0u8; BUFFER_SIZE];

    match conn.0.try_read(&mut kb_buffer) {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let cmd = serde_json::from_slice::<Command>(&kb_buffer[..bytes]).unwrap();
            println!("{:?}", cmd);
            let (responder, sender) = oneshot::channel();
            let wrapped_cmd = StreamCommand {
                cmd,
                resp: Some(responder),
            };
            tx.send(wrapped_cmd).await.unwrap();
            let result = sender.await.unwrap();
            println!("CONN:\n{}", result);
            conn.0.write(result.as_bytes()).await.unwrap();
        }
        Err(e) => eprintln!("Error reading into buffer: {:?}", e),
    }
}
