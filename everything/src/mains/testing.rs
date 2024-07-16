use futures::task::SpawnExt;
use std::time::Duration;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let mut set: JoinSet<Result<(), anyhow::Error>> = JoinSet::new();

    for i in 1..6 {
        set.spawn(async move {
            tokio::time::sleep(Duration::from_millis(rand::random::<u64>() % 3_000)).await;
            println!("{i} finished");
            Ok(())
        });
    }

    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        println!("set len: {:?}", set.len());
    }
}
