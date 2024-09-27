use std::time::Duration;
use tokio::select;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut finished_task =
        tokio::spawn(async { tokio::time::sleep(Duration::from_secs(1)).await });

    let mut forever_task =
        tokio::spawn(async { tokio::time::sleep(Duration::from_secs(1_000_000)).await });

    let mut finished = false;
    loop {
        select! {
            // this should panic because we are polling completed task again
            finished_task_res = &mut finished_task, if !finished => {
                println!("Finished task has finished");
                finished = true;
            }
            forever_task_res = &mut forever_task => {
                println!("WTF");
            }
        }
    }

    Ok(())
}
