use std::path::Path;
use std::time::Duration;
use sysinfo::{System, MINIMUM_CPU_UPDATE_INTERVAL};

fn main() {
    // CPU
    let cpu_thread = std::thread::spawn(|| {
        let mut sys = System::new();
        loop {
            sys.refresh_cpu();
            let v: Vec<_> = sys
                .cpus()
                .iter()
                .map(|cpu| (cpu.name(), cpu.cpu_usage()))
                .collect();
            println!("        CPU");
            v.into_iter()
                .for_each(|(name, usage)| println!("            {} | {:.2} %", name, usage));

            sys.refresh_processes();
            println!("        PROCESSES");
            sys.processes()
                .iter()
                .for_each(|p| println!("           {} | {} - {:?}", p.0, p.1.name(), p.1.exe()));

            std::thread::sleep(Duration::from_millis(2_000));
        }
    });

    let net_thread = std::thread::spawn(|| {
        let mut networks = sysinfo::Networks::new_with_refreshed_list();
        loop {
            networks.refresh();
            for (interface_name, network) in &networks {
                println!("[{interface_name}] {:?}", network.received());
            }

            std::thread::sleep(Duration::from_millis(1_700));
        }
    });

    cpu_thread.join().unwrap();
}
