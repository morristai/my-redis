use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

type ShardedDb = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    println!("Listening");

    fn new_sharded_db(num_shards: usize) -> ShardedDb {
        let mut db = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            db.push(Mutex::new(HashMap::new()));
        }
        Arc::new(db)
    }

    let db = new_sharded_db(4);

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // Clone the handle to the hash map.
        let db = db.clone();

        println!("Accepted");
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

fn calculate_hash<T: Hash + ?Sized>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

async fn process(socket: TcpStream, db: ShardedDb) {
    use mini_redis::Command::{self, Get, Set};

    // Connection, provided by `mini-redis`, handles parsing frames from
    // the socket
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                // https://doc.rust-lang.org/std/hash/index.html
                let hash = calculate_hash(cmd.key());
                let mut shard = db[hash as usize % db.len()].lock().unwrap();
                // TODO: IDK why Set's key is String, but calling key() returns &str
                // https://github.com/tokio-rs/mini-redis/blob/b1e365b62fd056653f5a883798317df3fdbfcf49/src/cmd/set.rs#L46
                shard.insert(cmd.key().to_string(), cmd.value().clone());

                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let hash = calculate_hash(cmd.key());
                let shard = db[hash as usize % db.len()].lock().unwrap();
                if let Some(value) = shard.get(cmd.key()) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
}