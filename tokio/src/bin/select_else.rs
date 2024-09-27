#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, rx) = flume::unbounded();
    let select_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Ok(message) = rx.recv_async() => {
                    println!("Received message: {:?}", message);
                }
                else => {
                    println!("Channel dropped");
                    break;
                }
            }
        }
    });

    tx.send("first message")?;
    tx.send("Second message")?;
    std::mem::drop(tx);

    let task_res = select_task.await;
    println!("Task res: {:?}", task_res);

    Ok(())
}
