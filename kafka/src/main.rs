use anyhow::{anyhow, Context, Result};
use rdkafka::admin::{AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::{FromClientConfig, FromClientConfigAndContext};
use rdkafka::consumer::stream_consumer::StreamPartitionQueue;
use rdkafka::consumer::{Consumer, DefaultConsumerContext, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::message::BorrowedMessage;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::TokioRuntime;
use rdkafka::{ClientConfig, Message, Offset, TopicPartitionList};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::task::JoinSet;
use tracing::{error, info};

const TOPIC_NAME: &str = "test-repartition";

#[tokio::main]
async fn main() -> Result<()> {
    info!("Starting kafka test");
    init_logging();

    let mut client_config = ClientConfig::new();
    client_config.set("bootstrap.servers", "localhost:29092");
    client_config.set("group.id", "test-group-id");
    let client_config = Arc::new(client_config);
    let mut tasks: JoinSet<Result<()>> = JoinSet::new();

    let admin_client = rdkafka::admin::AdminClient::from_config_and_context(
        &*client_config,
        DefaultClientContext,
    )?;

    admin_client
        .create_topics(
            [&NewTopic::new(TOPIC_NAME, 3, TopicReplication::Fixed(1))],
            &AdminOptions::new(),
        )
        .await
        .context("Error occurred when creating topic")?;

    let consumer =
        Arc::new(StreamConsumer::from_config(&*client_config).context("can not create consumer")?);
    consumer
        .assign(&TopicPartitionList::from_topic_map(&HashMap::from([
            ((TOPIC_NAME.to_owned(), 0), Offset::End),
            ((TOPIC_NAME.to_owned(), 1), Offset::End),
        ]))?)
        .context("subscription problem")?;

    tasks.spawn(run_producer(client_config.clone()));
    tasks.spawn(run_consumers(consumer.clone()));

    // sleep and reassign
    tokio::time::sleep(Duration::from_secs(7)).await;
    info!("Reassigning");
    consumer
        .assign(&TopicPartitionList::from_topic_map(&HashMap::from([
            ((TOPIC_NAME.to_owned(), 0), Offset::Stored),
            ((TOPIC_NAME.to_owned(), 1), Offset::Stored),
            ((TOPIC_NAME.to_owned(), 2), Offset::Stored),
        ]))?)
        .context("subscription problem")?;

    tasks.spawn(async move {
        let split: StreamPartitionQueue<DefaultConsumerContext, TokioRuntime> = consumer
            .split_partition_queue(TOPIC_NAME, 2)
            .ok_or(anyhow!("could not split partition queue"))?;
        info!("Created split reader for partition 2");

        loop {
            match split.recv().await {
                Ok(message) => {
                    let payload = String::from_utf8_lossy(
                        message.payload().ok_or(anyhow!("deser string error"))?,
                    )
                    .to_string();
                    info!("2 | Obtained message: {}", payload)
                }
                Err(err) => {
                    error!("Split consumer error: {:?}", err)
                }
            }
        }

        Ok(())
    });

    while let Some(res) = tasks.join_next().await {
        info!("Ended main task: {:?}", res);
        break;
    }

    info!("Finished kafka test");
    Ok(())
}

async fn run_producer(config: Arc<ClientConfig>) -> Result<()> {
    let producer: FutureProducer = config.create()?;

    let mut i = 0;
    loop {
        let payload = format!("Message-{}", i);
        match producer.send_result(
            FutureRecord::<(), [u8]>::to(TOPIC_NAME)
                .timestamp(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64)
                .partition(i % 3)
                .payload(payload.as_bytes()),
        ) {
            Ok(_) => {}
            Err(err) => {
                error!("Could send message {}: {:?}", i, err)
            }
        }
        i += 1;
        tokio::time::sleep(Duration::from_millis(0_876)).await;
    }

    Ok(())
}

async fn run_consumers(consumer: Arc<StreamConsumer>) -> Result<()> {
    let mut tasks: JoinSet<Result<()>> = JoinSet::new();

    for i in 0..=1 {
        let consumer_c = consumer.clone();
        tasks.spawn(async move { consumer_task(consumer_c, i).await });
    }

    while let Some(res) = tasks.join_next().await {
        info!("Ended task: {:?}", res);
        break;
    }

    Ok(())
}

async fn consumer_task(consumer: Arc<StreamConsumer>, partition: i32) -> Result<()> {
    let split: StreamPartitionQueue<DefaultConsumerContext, TokioRuntime> = consumer
        .split_partition_queue(TOPIC_NAME, partition)
        .ok_or(anyhow!("could not split partition queue"))?;
    info!("Created split reader for partition {partition}");

    loop {
        match split.recv().await {
            Ok(message) => {
                let payload = String::from_utf8_lossy(
                    message.payload().ok_or(anyhow!("deser string error"))?,
                )
                .to_string();
                info!("{partition} | Obtained message: {}", payload)
            }
            Err(err) => {
                error!("Split consumer error: {:?}", err)
            }
        }
    }
}

pub fn init_logging() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "kafka=info".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_line_number(true)
        .with_file(true)
        .with_level(true)
        .try_init()
        .unwrap();
}
