use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bytes::Bytes;
use mini_redis::{Command, Connection, Frame, Result};
use tokio::net::{TcpListener, TcpStream};

type DB = Arc<Mutex<HashMap<String, Bytes>>>;
#[tokio::main]
async fn main() -> Result<()>  {
  let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let db = Arc::new(Mutex::new(HashMap::new()));
   loop{
       let(socket, _) = listener.accept().await?;
       let db = db.clone();
       tokio::spawn(async move{
           process(socket, db).await;
       });
   }
}

async fn process(socket: TcpStream, db: DB) {
    let mut connection = Connection::new(socket);
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Command::Set(command) => {
                println!("Received command: {:?}", command);

                db.lock().unwrap().insert(command.key().to_string(), command.value().clone());
                Frame::Simple("OK".to_string())
            }
            Command::Get(command) => {
                if let Some(v) = db.lock().unwrap().get(command.key()) {
                    println!("Received command: {:?}", command);
                    Frame::Bulk(v.clone().into())
                } else {
                    Frame::Null
                }
            }
            command => panic!("unknown command {:?}", command)
        };
        connection.write_frame(&response).await.unwrap();
    }
}
