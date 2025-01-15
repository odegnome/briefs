use tokio::sync::mpsc;
use tokio::sync::oneshot;

type Responder<T> = oneshot::Sender<T>;

#[derive(Debug)]
enum Command {
    Set {
        msg: String,
        resp: Responder<String>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(2);
    //let cmd = Command::Set { resp: trans };

    let pr2 = tokio::spawn(async move {
        //println!("- Receiving over oneshot");
        let mut msg_count = 1u8;
        while let Some(res) = rx.recv().await {
            match res {
                Command::Set { msg, resp } => {
                    println!("HANDLE: {:?}", msg);
                    //println!("- Sending number as response");
                    resp.send(format!("Ack #{}", msg_count)).unwrap();
                }
            }
            msg_count += 1;
        }
    });

    let pr1 = tokio::spawn(async move {
        for index in 1..=10 {
            let (responder, receiver) = oneshot::channel();
            //println!("- Initiating send from new task");
            let cmd = Command::Set {
                msg: format!("Message #{}", index),
                resp: responder,
            };
            tx.send(cmd).await.unwrap();

            let res = receiver.await.unwrap();
            //println!("- Spawned task received message");
            println!("TASK: {:?}", res);
            tokio::time::sleep(tokio::time::Duration::new(5, 0)).await;
        }
    });

    pr1.await.unwrap();
    pr2.await.unwrap();
}
