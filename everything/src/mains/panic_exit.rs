use std::panic;
use std::process;
use std::thread;

fn main() {
    // take_hook() returns the default hook in case when a custom one is not set
    let orig_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // invoke the default handler and exit the process
        orig_hook(panic_info);
        process::exit(1);
    }));

    thread::spawn(move || {
        panic!("something bad happened");
    })
    .join();

    // this line won't ever be invoked because of process::exit()
    println!("Won't be printed");
}
