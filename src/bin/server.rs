#![allow(unused_imports)]
use catchup::{post, stream, Command, StreamCommand};
use std::net::SocketAddr;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(10);

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

        //// !-------SIMULATING REQUEST-------!

        //let (responder, receiver) = oneshot::channel();
        //let cmd = Command::Create {
            //title: String::from("Hello, World!"),
            //msg: String::from(
                //"This is my first post, and as is the tradition, \
                //the post is titled 'Hello, World!'. Hopefully, this works!",
            //),
        //};
        //let wrapped_cmd = StreamCommand {
            //cmd,
            //resp: Some(responder),
        //};
        //tx.send(wrapped_cmd).await.unwrap();

        //let res = receiver.await.unwrap();
        //println!("CONN: {:?}", res);

        //// !-------SIMULATING REQUEST-------!

        //let (responder, receiver) = oneshot::channel();
        //let cmd = Command::Create {
            //title: String::from("Another Hello, World!"),
            //msg: String::from(
                //"Nothing new in this post. It is simply copied from the last \
                //post. This is my first post, and as is the tradition, \
                //the post is titled 'Hello, World!'. Hopefully, this works!",
            //),
        //};
        //let wrapped_cmd = StreamCommand {
            //cmd,
            //resp: Some(responder),
        //};
        //tx.send(wrapped_cmd).await.unwrap();

        //let res = receiver.await.unwrap();
        //println!("CONN: {:?}", res);
        //tokio::time::sleep(tokio::time::Duration::new(5, 0)).await;

        //// !-------SIMULATING REQUEST-------!

        //let (responder, receiver) = oneshot::channel();
        //println!("-------CatchUP-------");
        //let cmd = Command::Catchup {};
        //let wrapped_cmd = StreamCommand {
            //cmd,
            //resp: Some(responder),
        //};
        //tx.send(wrapped_cmd).await.unwrap();

        //let res = receiver.await.unwrap();
        //println!("CONN:\n{}", res);

        // !------- ACCEPT CONNECTIONS ON PORT 8080 -------!

        let listener = TcpListener::bind("localhost:8080").await.unwrap();
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

async fn handle_request(conn: (TcpStream, SocketAddr), _tx: mpsc::Sender<StreamCommand>) {
    println!("Succesfully connected with {:?}", conn.1);

    conn.0.readable().await.unwrap();

    let mut kb_buffer = [0u8; 1024];

    match conn.0.try_read(&mut kb_buffer) {
        Ok(bytes) => {
            println!("Read {bytes} bytes");
            let cmd = serde_json::from_slice::<Command>(&kb_buffer[..bytes]).unwrap();
            println!("{:?}", cmd);
        },
        Err(e) => eprintln!("Error on reading into buffer: {:?}", e),
    }
}
