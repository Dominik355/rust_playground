use core::time::Duration;
use futures_util::future::OptionFuture;
use std::time::Instant;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    let mut timer2 = Box::pin(conditional_sleeper(Some(tokio::time::sleep(
        Duration::from_millis(1_000),
    ))));
    let mut timer3 = Box::pin(OptionFuture::from(Some(tokio::time::sleep(
        Duration::from_millis(1_400),
    ))));

    let mut timer4 = Some(tokio::spawn(tokio::time::sleep(Duration::from_millis(
        1_400,
    ))));

    let start = Instant::now();
    let duration = Duration::from_millis(2_000);

    loop {
        let timer1 = tokio::time::sleep(Duration::from_millis(300));
        tokio::select! {
            _ = timer1 => {
                println!("hello from timer 1");
                if start.elapsed() >= duration {
                    println!("Terminating");
                    break;
                }
            },
            res = async { timer4.as_mut().expect("has to be Some").await }, if timer4.is_some() => {
                println!("hello from timer 4");
                timer4 = None;
            }
            // Some(_) = &mut timer2 => {
            //     println!("timer 2 finished");
            // }
            // Some(_) = &mut timer3 => {
            //     println!("timer 3 finished");
            // }
        }
    }

    println!("Done");
}

/// This is the same as OptionFuture
async fn conditional_sleeper(t: Option<tokio::time::Sleep>) -> Option<()> {
    match t {
        Some(timer) => Some(timer.await),
        None => None,
    }
}
