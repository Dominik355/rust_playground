use anyhow::{bail, Context, Result};
use tokio::task::JoinSet;
use tracing::{info, info_span, Instrument};

// What is that name for ? how to show that in logs or even maybe obtain that within some task context ?
#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    let mut tasks: JoinSet<Result<usize>> = JoinSet::new();

    for i in 0..3 {
        let _ = tasks
            .build_task()
            .name(&format!("task-{}", i))
            .spawn(async move {
                async_task()
                    .instrument(info_span!("init-agg", num = ?i))
                    .await?;
                Ok(i)
            });
    }

    while let Some(join_res) = tasks.join_next().await {
        match join_res.context("Task failed to complete")? {
            Ok(dsp) => {
                info!("Finished initial aggregation for {}", dsp);
            }
            Err(err) => {
                bail!("Aggregation task failed: {}", err);
            }
        }
    }

    Ok(())
}

async fn async_task() -> Result<()> {
    info!("Starting async-task");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    bail!("fucked up");
}

pub fn init_logging() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "named_tokio_tasks=trace".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_thread_names(true)
        .with_level(true)
        .try_init()
        .unwrap();
}
