use std::path::PathBuf;
use std::{net::ToSocketAddrs, sync::Arc};

use briefs_core::state::CatchUpResponse;
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use tokio::{net::TcpListener, signal::ctrl_c, sync::mpsc};
use tokio_rustls::{rustls, TlsAcceptor};

use briefs_core::{post, stream, Command, StreamCommand, StreamResponse, db::generate_temp_db};

use server::{
    handle_conn_request,
    interprocess::respond_with_bytes,
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
        briefs_core::db::setup_tables(&mut conn).expect("Unable to setup tables");

        //-------
        // Handle requets from conn handler
        //-------
        while let Some(StreamCommand { cmd, resp }) = rx.recv().await {
            match cmd {
                Command::Create { title, msg } => {
                    let new_post = post::Post::new(stream.size(), title, msg);
                    if new_post.is_err() {
                        respond_with_bytes(
                            resp.unwrap(),
                            serde_json::to_vec(&StreamResponse::new(format!(
                                "ERROR during create: {:?}",
                                new_post.unwrap_err()
                            )))
                            .unwrap(),
                        );
                        continue;
                    }
                    let new_post = new_post.unwrap();
                    let result = stream.add_post(&mut conn, new_post.clone());
                    if result.is_err() {
                        respond_with_bytes(
                            resp.unwrap(),
                            serde_json::to_vec(&StreamResponse::new(format!(
                                "ERROR during create: {:?}",
                                result.unwrap_err()
                            )))
                            .unwrap(),
                        );
                        continue;
                    }

                    // Insert into db
                    let result = briefs_core::db::insert_post(&mut conn, &new_post);
                    if result.is_err() {
                        respond_with_bytes(
                            resp.unwrap(),
                            serde_json::to_vec(&StreamResponse::new(format!(
                                "ERROR during create: {:?}",
                                result.unwrap_err()
                            )))
                            .unwrap(),
                        );
                        continue;
                    }

                    respond_with_bytes(
                        resp.unwrap(),
                        serde_json::to_vec(&StreamResponse::new(format!(
                            "Succesfully added a new post"
                        )))
                        .unwrap(),
                    );
                }

                Command::Catchup { last_fetch_id } => {
                    if stream.size() == 0 || last_fetch_id >= stream.size() {
                        let empty_stream_response = serde_json::to_vec(&CatchUpResponse {
                            posts: vec![],
                            caught_up: true,
                        })
                        .unwrap();
                        respond_with_bytes(resp.unwrap(), empty_stream_response);
                        continue;
                    };

                    let uncaught_posts = stream.size() - 1 - last_fetch_id;
                    let limit_index = if uncaught_posts > 10 {
                        last_fetch_id + 11
                    } else {
                        stream.size()
                    };

                    // Catchup
                    let response = stream.catchup(&mut conn, last_fetch_id, limit_index);
                    if response.is_err() {
                        respond_with_bytes(
                            resp.unwrap(),
                            serde_json::to_vec(&StreamResponse::new(format!(
                                "An error occured: {:?}",
                                response.unwrap_err()
                            )))
                            .unwrap(),
                        );
                        continue;
                    }

                    // Serialise the response
                    let response = serde_json::to_vec(&response.unwrap());
                    if response.is_err() {
                        respond_with_bytes(
                            resp.unwrap(),
                            serde_json::to_vec(&StreamResponse::new(format!(
                                "An error occured: {:?}",
                                response.unwrap_err()
                            )))
                            .unwrap(),
                        );
                        continue;
                    }

                    // Update database
                    let result = briefs_core::db::query_posts(&mut conn, None);
                    if result.is_err() {
                        respond_with_bytes(
                            resp.unwrap(),
                            serde_json::to_vec(&StreamResponse::new(format!(
                                "An error occured: {:?}",
                                result.unwrap_err()
                            )))
                            .unwrap(),
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
                    let result = stream.get_post(&mut conn, id);
                    if result.is_none() {
                        respond_with_bytes(
                            resp.unwrap(),
                            serde_json::to_vec(&StreamResponse::new(format!(
                                "ERROR during get: Unable to get post"
                            )))
                            .unwrap(),
                        );
                        continue;
                    }
                    resp.unwrap()
                        .send(serde_json::to_vec(&result.unwrap()).unwrap_or_default())
                        .unwrap();
                }

                Command::Delete { id } => {
                    let result = stream.remove_post(&mut conn, id);
                    if result.is_err() {
                        respond_with_bytes(
                            resp.unwrap(),
                            serde_json::to_vec(&StreamResponse::new(format!(
                                "ERROR during delete: {}",
                                result.unwrap_err()
                            )))
                            .unwrap(),
                        );
                        continue;
                    }

                    // Delete from db
                    let result = briefs_core::db::delete_post_by_id(&mut conn, id);
                    if result.is_err() {
                        respond_with_bytes(
                            resp.unwrap(),
                            serde_json::to_vec(&StreamResponse::new(format!(
                                "ERROR during delete: {:?}",
                                result.unwrap_err()
                            )))
                            .unwrap(),
                        );
                        continue;
                    }

                    respond_with_bytes(resp.unwrap(), format!("Succesfully deleted post").into());
                }

                Command::UpdateMsg { id, msg } => {
                    let result = stream.update_msg(&mut conn, id, msg);
                    if result.is_err() {
                        respond_with_bytes(
                            resp.unwrap(),
                            serde_json::to_vec(&StreamResponse::new(format!(
                                "ERROR during message update: {}",
                                result.unwrap_err()
                            )))
                            .unwrap(),
                        );
                        continue;
                    }
                    respond_with_bytes(
                        resp.unwrap(),
                        serde_json::to_vec(&StreamResponse::new(format!(
                            "Succesfully updated post message"
                        )))
                        .unwrap(),
                    );
                }

                Command::UpdateTitle { id, title } => {
                    let result = stream.update_title(&mut conn, id, title);
                    if result.is_err() {
                        respond_with_bytes(
                            resp.unwrap(),
                            serde_json::to_vec(&StreamResponse::new(format!(
                                "ERROR during title update: {}",
                                result.unwrap_err()
                            )))
                            .unwrap(),
                        );
                        continue;
                    }
                    respond_with_bytes(
                        resp.unwrap(),
                        serde_json::to_vec(&StreamResponse::new(format!(
                            "Succesfully updated post title"
                        )))
                        .unwrap(),
                    );
                }

                Command::Metadata {} => {
                    let result = stream.stream_metadata();
                    if result.is_err() {
                        respond_with_bytes(
                            resp.unwrap(),
                            serde_json::to_vec(&StreamResponse::new(format!(
                                "ERROR during title update: {}",
                                result.unwrap_err()
                            )))
                            .unwrap(),
                        );
                        continue;
                    }
                    resp.unwrap()
                        .send(serde_json::to_vec(&result.unwrap()).unwrap_or_default())
                        .unwrap();
                }
            }
        }
    });

    let conn_handle = tokio::spawn(async move {
        let socket_addr = "0.0.0.0:8080".to_socket_addrs().unwrap().next().unwrap();
        let server_cert = PathBuf::from("/Users/rishabh/project/briefs/auth/keys/cert.pem");
        let private_key = PathBuf::from("/Users/rishabh/project/briefs/auth/keys/key.pem");
        println!("Setting up connection handler...");

        let certs = CertificateDer::pem_file_iter(server_cert)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let key = PrivateKeyDer::from_pem_file(private_key).unwrap();

        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .unwrap();
        let acceptor = TlsAcceptor::from(Arc::new(config));

        // !------- ACCEPT CONNECTIONS ON PORT 8080 -------!
        let listener = TcpListener::bind(socket_addr).await.unwrap();
        println!("Listening on {}...", listener.local_addr().unwrap());

        loop {
            let _tx = tx.clone();
            let conn = listener.accept().await;
            let acceptor = acceptor.clone();

            if conn.is_ok() {
                tokio::spawn(async move {
                    let stream = acceptor.accept(conn.unwrap().0).await.unwrap();
                    // function signature needs to change for this
                    handle_conn_request(stream, _tx).await;
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
