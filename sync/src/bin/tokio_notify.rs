use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task::JoinSet;

#[tokio::main]
pub async fn main() {
    let notify = Arc::new(Notify::new());

    let mut set = JoinSet::new();
    for i in 0..10 {
        let notify_c = notify.clone();
        set.spawn(async move {
            notify_c.notified().await;
            println!("{} | received notification", i);
        });
    }

    println!("Sleeping for 2 secs");
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("sending notification");
    notify.notify_waiters();

    while let Some(res) = set.join_next().await {
        println!("task done: {:?}", res)
    }
}
