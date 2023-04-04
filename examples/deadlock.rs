use std::sync::Mutex;

use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let mtx = Mutex::new(0);

    tokio::join!(work(&mtx), work(&mtx));

    println!("{}", *mtx.lock().unwrap());
}

async fn work(mtx: &Mutex<i32>) {
    println!("lock");
    {
        let mut v = mtx.lock().unwrap();
        println!("locked");
        // 1. task1 got suspended on sleep but still holds the Mutex lock
        // 2. task2 will block synchronously on obtaining the Mutex that owned by task 1
        // 3. Since task 2 is blocked, this also means the runtime thread is fully blocked. It can not actually go into task1 timer handling state
        sleep(Duration::from_millis(100)).await;
        *v += 1;
    } // lock goes out of scope here, so this can be compiled even std MutexGuard doesn't impl Send
    println!("unlock")
}

// stdout:
// lock
// locked
// lock
