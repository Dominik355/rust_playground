mod panic_exit;

use flume::Sender;
use std::any::TypeId;

fn main() {
    let (tx, rx) = flume::bounded(10);

    let mut supplier: Box<dyn FnMut() -> Sender<i64>> = Box::new(|| tx.clone());

    for i in 0..4 {
        let tx = supplier();
        std::thread::spawn(move || {
            tx.send(i).unwrap();
        });
    }

    while let Ok(v) = rx.recv() {
        println!("obtained: {:?}", v);
    }
}

// fn main() {
//     println!("String: {:?}", TypeId::of::<String>());
//     println!("u32: {:?}", TypeId::of::<u32>());
//     println!("u64: {:?}", TypeId::of::<u64>());
//     println!("f64: {:?}", TypeId::of::<f64>());
//     println!("bool: {:?}", TypeId::of::<bool>());
//     println!("i128: {:?}", TypeId::of::<i128>());
//     println!("CustomEnum: {:?}", TypeId::of::<CustomEnum>());
// }
//
// enum CustomEnum {
//     First,
//     Second,
// }
