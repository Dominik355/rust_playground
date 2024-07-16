use std::panic;
use std::process;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        let p = PoisonPill;
        tokio::time::sleep(Duration::from_secs(3)).await;
        panic!("something bad happened");
    });

    loop {
        tokio::time::sleep(Duration::from_millis(400)).await;
        println!("here we go again");
    }
}

struct PoisonPill;

impl Drop for PoisonPill {
    fn drop(&mut self) {
        if thread::panicking() {
            println!("dropped while unwinding");
            process::exit(1);
        }
    }
}
