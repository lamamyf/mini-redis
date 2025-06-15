use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>
    },
    Set {
        key: String,
        value: Bytes,
        resp: Responder<()>
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    let manger = tokio::spawn(async move {
        use Command::*;

        let mut client = client::connect("127.0.0.1:6379").await.unwrap();
        while let Some(command) = rx.recv().await {
            match command {
                Get { key, resp } => {
                    let response = client.get(&key).await;
                    let _ = resp.send(response);
                }
                Set { key, value, resp  } => {
                    let response = client.set(&key, value).await;
                    let _ = resp.send(response);
                }
            }
        }
    });

    let tx2 = tx.clone();
    let get_key_task = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let command  = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };
        tx.send(command).await.unwrap();
        let response = resp_rx.await;
        println!("GOT = {:?}", response);
    });

    let set_key_task = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let command = Command::Set {
            key: "foo".to_string(),
            value: "bar".into(),
            resp: resp_tx
        };
        tx2.send(command).await.unwrap();
        let response = resp_rx.await;
        println!("GOT = {:?}", response);
    });

    get_key_task.await.unwrap();
    set_key_task.await.unwrap();
    manger.await.unwrap();
}