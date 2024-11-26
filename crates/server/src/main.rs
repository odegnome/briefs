use catchup_core::{post, stream, Command, StreamCommand};
use tokio::{net::TcpListener, signal::ctrl_c, sync::mpsc};

use server::{
    database, generate_temp_db, handle_conn_request, interprocess::respond_with_string,
    setup_server,
};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(16);

    let db_path_outer = generate_temp_db();
    let db_path = db_path_outer.to_owned();
    let stream_handle = tokio::spawn(async move {
        //-------
        // Setups
        //-------
        println!("Stream handle running...");
        let mut stream = stream::Stream::default();
        setup_server(Some(db_path.clone().into())).expect("Unable to setup db");
        let mut conn = sqlite::open(db_path).expect("Unable to open connection");
        database::setup_tables(&mut conn).expect("Unable to setup tables");

        //-------
        // Handle requets from conn handler
        //-------
        while let Some(StreamCommand { cmd, resp }) = rx.recv().await {
            match cmd {
                Command::Create { title, msg } => {
                    let new_post = post::Post::new(stream.size(), title, msg);
                    if new_post.is_err() {
                        respond_with_string(
                            resp.unwrap(),
                            format!("ERROR during create: {:?}", new_post.unwrap_err()),
                        );
                        continue;
                    }
                    let new_post = new_post.unwrap();
                    let result = stream.add_post(new_post.clone());
                    if result.is_err() {
                        respond_with_string(
                            resp.unwrap(),
                            format!("ERROR during create: {:?}", result.unwrap_err()),
                        );
                        continue;
                    }

                    // Insert into db
                    let result = database::insert_post(&mut conn, &new_post);
                    if result.is_err() {
                        respond_with_string(
                            resp.unwrap(),
                            format!("ERROR during create: {:?}", result.unwrap_err()),
                        );
                        continue;
                    }

                    respond_with_string(resp.unwrap(), format!("Succesfully added a new post"));
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

                    // Catchup
                    let response = stream.catchup(last_fetch_id, limit_index);
                    if response.is_err() {
                        respond_with_string(
                            resp.unwrap(),
                            format!("An error occured: {:?}", response.unwrap_err()),
                        );
                        continue;
                    }

                    // Serialise the response
                    let response = serde_json::to_string(&response.unwrap());
                    if response.is_err() {
                        respond_with_string(
                            resp.unwrap(),
                            format!("An error occured: {:?}", response.unwrap_err()),
                        );
                        continue;
                    }

                    // Update database
                    let result = database::query_posts(&mut conn, None);
                    if result.is_err() {
                        respond_with_string(
                            resp.unwrap(),
                            format!("An error occured: {:?}", result.unwrap_err()),
                        );
                        continue;
                    }
                    let rows = result.unwrap();
                    for row in rows.iter() {
                        println!("{:?}", row);
                    }

                    resp.unwrap().send(response.unwrap()).unwrap();

                    //resp.unwrap().send(format!("{}", &stream)).unwrap();
                }

                Command::Get { id } => {
                    let result = stream.get_post(id);
                    if result.is_none() {
                        respond_with_string(
                            resp.unwrap(),
                            format!("ERROR during get: Unable to get post"),
                        );
                        continue;
                    }
                    resp.unwrap()
                        .send(format!(
                            "{}",
                            serde_json::to_string(&result.unwrap()).unwrap_or_default()
                        ))
                        .unwrap();
                }

                Command::Delete { id } => {
                    let result = stream.remove_post(id);
                    if result.is_err() {
                        respond_with_string(
                            resp.unwrap(),
                            format!("ERROR during delete: {}", result.unwrap_err()),
                        );
                        continue;
                    }

                    // Delete from db
                    let result = database::delete_post_by_id(&mut conn, id);
                    if result.is_err() {
                        respond_with_string(
                            resp.unwrap(),
                            format!("ERROR during delete: {:?}", result.unwrap_err()),
                        );
                        continue;
                    }

                    respond_with_string(resp.unwrap(), format!("Succesfully deleted post"));
                }

                Command::UpdateMsg { id, msg } => {
                    let result = stream.update_msg(id, msg);
                    if result.is_err() {
                        resp.unwrap()
                            .send(format!(
                                "ERROR during message update: {}",
                                result.unwrap_err()
                            ))
                            .unwrap();
                        continue;
                    }
                    respond_with_string(
                        resp.unwrap(),
                        format!("Succesfully updated post message",),
                    );
                }

                Command::UpdateTitle { id, title } => {
                    let result = stream.update_title(id, title);
                    if result.is_err() {
                        resp.unwrap()
                            .send(format!(
                                "ERROR during title update: {}",
                                result.unwrap_err()
                            ))
                            .unwrap();
                        continue;
                    }
                    respond_with_string(resp.unwrap(), format!("Succesfully updated post title",));
                }

                Command::Metadata {} => {
                    let result = stream.stream_metadata();
                    if result.is_err() {
                        resp.unwrap()
                            .send(format!(
                                "ERROR during title update: {}",
                                result.unwrap_err()
                            ))
                            .unwrap();
                        continue;
                    }
                    resp.unwrap()
                        .send(format!(
                            "{}",
                            serde_json::to_string(&result.unwrap()).unwrap()
                        ))
                        .unwrap();
                }
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
                    handle_conn_request(conn.unwrap(), _tx).await;
                });
            }
        }
    });

    let safe_exit_handle = tokio::spawn(async move {
        ctrl_c().await.unwrap();
        println!("\nCtrl-C");
        std::fs::remove_file(db_path_outer).expect("Unable to remove Db file");
        std::process::exit(0);
    });

    println!("Press Ctrl-C to stop the server; this also deletes the test db");

    //-------
    // Wait for all threads
    //-------
    conn_handle.await.unwrap();
    stream_handle.await.unwrap();
    safe_exit_handle.await.unwrap();
}
