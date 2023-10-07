#![allow(unused_imports)]
use catchup::{post, stream, Command, StreamCommand};
use std::net::SocketAddr;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(16);

    let stream_handle = tokio::spawn(async move {
        println!("Stream handle running...");
        let mut post_count = 0usize;
        let mut new_stream = stream::Stream::default();
        while let Some(StreamCommand { cmd, resp }) = rx.recv().await {
            match cmd {
                Command::Create { title, msg } => {
                    let post = post::Post::new(post_count + 1, title, msg);
                    if post.is_err() {
                        resp.unwrap()
                            .send(format!("ERROR during create: {:?}", post.unwrap_err()))
                            .unwrap();
                        continue;
                    }
                    let result = new_stream.add_post(post.unwrap());
                    if result.is_err() {
                        resp.unwrap()
                            .send(format!("ERROR during create: {:?}", result.unwrap_err()))
                            .unwrap();
                        continue;
                    }
                    resp.unwrap()
                        .send(format!("Succesfully added a new post"))
                        .unwrap();
                    post_count += 1;
                }

                Command::Catchup {} => {
                    resp.unwrap().send(format!("{}", &new_stream)).unwrap();
                }

                _ => eprintln!("Feature not implemented"),
            }
        }
    });

    let conn_handle = tokio::spawn(async move {
        println!("Connection handle running...");

        // !------- ACCEPT CONNECTIONS ON PORT 8080 -------!

        let listener = TcpListener::bind("192.168.1.16:8080").await.unwrap();
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

    let mut kb_buffer = [0u8; 1024];

    match conn.0.try_read(&mut kb_buffer) {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let cmd = serde_json::from_slice::<Command>(&kb_buffer[..bytes]).unwrap();
            println!("{:?}", cmd);
            let (responder, receiver) = oneshot::channel();
            let wrapped_cmd = StreamCommand {
                cmd,
                resp: Some(responder),
            };
            tx.send(wrapped_cmd).await.unwrap();
            let result = receiver.await.unwrap();
            println!("CONN:\n{}", result);
            conn.0.write(result.as_bytes()).await.unwrap();
        }
        Err(e) => eprintln!("Error reading into buffer: {:?}", e),
    }
}
