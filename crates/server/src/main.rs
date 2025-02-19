use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use tokio::{net::TcpListener, signal::ctrl_c, sync::mpsc};
use tokio_rustls::{rustls, TlsAcceptor};

use briefs_core::{
    config,
    db::generate_temp_db,
    post,
    state::CatchUpResponse,
    stream,
    utils::{read_stream_from_disk, save_stream_on_disk},
    Command, StreamCommand, StreamResponse,
};

use server::{handle_conn_request, interprocess::respond_with_bytes, setup_server};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(16);

    // Load config or Generate new config
    let config = match config::fetch_config_from_env() {
        Ok(filepath) => config::BriefsConfig::from_file(filepath).unwrap(),
        Err(e) => {
            eprintln!("✗ Config env variable not set: {:?}", e);
            match config::fallback_config_dir() {
                Ok(filepath) => config::BriefsConfig::from_file(filepath).unwrap(),
                Err(e) => {
                    eprintln!("✗ Fallback config failure: {:?}", e);
                    println!("✓ Creating new default config");
                    let mut config = config::BriefsConfig::default();
                    let db_path = generate_temp_db();
                    config.db = db_path.clone();
                    config.socket = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
                    config.save().unwrap();
                    println!("✓ Saved config to '{}'", config.filepath.display());
                    config
                }
            }
        }
    };
    let socket = config.socket.clone();

    let stream_handle = tokio::spawn(async move {
        println!("✓ Stream handle running...");
        let db_path = config.db.clone();
        setup_server(Some(db_path.clone().into())).expect("Unable to setup db");
        let mut conn = sqlite::open(db_path).expect("Unable to open connection");
        let mut stream = read_stream_from_disk(&mut conn, &config).unwrap_or_else(|_| {
            eprintln!("✗ No Prexisting stream found");
            println!("✓ Creating a new stream");
            let s = stream::Stream::default();
            save_stream_on_disk(&s, &config).expect("✗ Failed to save stream");
            s
        });

        //-------
        // Handle requets from conn handler
        //-------
        while let Some(StreamCommand { cmd, resp }) = rx.recv().await {
            match cmd {
                Command::Create { title, msg } => {
                    let new_post = post::Post::new(stream.nposts() as u32, title, msg);
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

                    respond_with_bytes(
                        resp.unwrap(),
                        serde_json::to_vec(&StreamResponse::new(format!(
                            "Succesfully added a new post"
                        )))
                        .unwrap(),
                    );
                }

                Command::Catchup { last_fetch_id } => {
                    if stream.size() == 0 || last_fetch_id as usize >= stream.nposts() {
                        let empty_stream_response = serde_json::to_vec(&CatchUpResponse {
                            posts: vec![],
                            caught_up: true,
                        })
                        .unwrap();
                        respond_with_bytes(resp.unwrap(), empty_stream_response);
                        continue;
                    };

                    // Catchup
                    let response = stream.catchup(&mut conn, last_fetch_id, None);
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

                    resp.unwrap().send(response.unwrap()).unwrap();
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
        let socket_addr = socket;
        let server_cert = PathBuf::from("/Users/rishabh/project/briefs/auth/keys/cert.pem");
        let private_key = PathBuf::from("/Users/rishabh/project/briefs/auth/keys/key.pem");
        println!("✓ Setting up connection handler...");

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
        println!("✓ Listening on {}...", listener.local_addr().unwrap());

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
        println!("✓ Press Ctrl-C to stop the server");
        ctrl_c().await.unwrap();
        // std::fs::remove_file(db_path_outer).expect("Unable to remove Db file");
        std::process::exit(0);
    });

    //-------
    // Wait for all threads
    //-------
    conn_handle.await.unwrap();
    stream_handle.await.unwrap();
    safe_exit_handle.await.unwrap();
}
