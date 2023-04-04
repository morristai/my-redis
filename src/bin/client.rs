use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
    },
    Set {
        key: String,
        val: Bytes,
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    // The `Sender` handles are moved into the tasks. As there are two
    // tasks, we need a second `Sender`.
    let tx2 = tx.clone();

    // Spawn two tasks, one gets a key, the other sets a key
    let t1 = tokio::spawn(async move {
        let cmd = Command::Get {
            key: "foo".to_string(),
        };

        tx.send(cmd).await.unwrap();
    });

    let t2 = tokio::spawn(async move {
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
        };

        tx2.send(cmd).await.unwrap();
    });

    // spawn a task that processes messages from the channel. First, a client connection is established to Redis.
    // Then, received commands are issued via the Redis connection.
    let manager = tokio::spawn(async move {
        // Establish a connection to the server
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        // Start receiving messages
        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key } => {
                    client.get(&key).await;
                }
                Set { key, val } => {
                    client.set(&key, val).await;
                }
            }
        }
    });

    // .await the join handles to ensure the commands fully complete before the process exits
    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}