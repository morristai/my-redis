use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    // It is not possible to clone the receiver of an mpsc channel.
    let tx2 = tx.clone();

    tokio::spawn(async move {
        tx.send("sending from first handle").await;
    });

    tokio::spawn(async move {
        tx2.send("sending from second handle").await;
    });

    // When every Sender has gone out of scope or has otherwise been dropped,
    // it is no longer possible to send more messages into the channel. At this point,
    // the recv call on the Receiver will return None, which means that all senders are gone and the channel is closed.
    while let Some(message) = rx.recv().await {
        println!("GOT = {}", message);
    }
}