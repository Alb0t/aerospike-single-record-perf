#[macro_use]
extern crate aerospike;
extern crate chrono;
use chrono::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{
    env,
    io::{BufWriter, Write},
    thread,
    time::Duration,
};

use aerospike::{Client, ClientPolicy, WritePolicy};
use rand::{thread_rng, RngCore};

fn main() {
    let thread_count: usize = env::var("THREAD_COUNT")
        .unwrap_or_else(|_| "64".to_string())
        .parse()
        .expect("Invalid THREAD_COUNT value");

    let use_shared_client: bool = env::var("SHARED_CLIENT")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .expect("Invalid SHARED_CLIENT value");

    let hosts = env::var("AEROSPIKE_HOSTS").unwrap_or_else(|_| "127.0.0.1:3100".to_string());

    let client_policy = ClientPolicy {
        use_services_alternate: true,
        ..Default::default()
    };

    // Shared client instance if enabled
    let shared_client = if use_shared_client {
        Some(Arc::new(
            Client::new(&client_policy, &hosts).expect("Failed to connect to cluster"),
        ))
    } else {
        None
    };

    let start_epoch_time = Utc::now().timestamp();
    let mut handles = Vec::new();

    // Shared buffered writer for logging
    let log_writer = Arc::new(Mutex::new(BufWriter::new(std::io::stdout())));

    for worker in 0..thread_count {
        let client = if use_shared_client {
            Arc::clone(shared_client.as_ref().unwrap())
        } else {
            Arc::new(Client::new(&client_policy, &hosts).expect("Failed to connect to cluster"))
        };

        let log_writer = Arc::clone(&log_writer);

        let handle = thread::spawn(move || {
            let mut count = 1;
            let mut blob_size: usize = 1;
            let key = as_key!("test", "test", "test");

            // Pre-allocate a large buffer to reduce per-iteration RNG overhead
            let mut rng = thread_rng();
            let mut data = vec![0u8; 1024 * 1024]; // 1MB buffer
            rng.fill_bytes(&mut data);

            loop {
                let elapsed_seconds = Utc::now().timestamp() - start_epoch_time;
                blob_size = (elapsed_seconds * 1024) as usize;

                // Ensure we don't exceed preallocated buffer
                let blob = &data[..blob_size.min(data.len())];

                let bin = aerospike::Bin::new("name", aerospike::Value::Blob(blob.to_vec()));

                let now = Instant::now();
                let result = client.put(&WritePolicy::default(), &key, &[bin]);

                if count % 100 == 0 {
                    let log_message = format!(
                        "Result:{},EpochTime:{},Thread:{},Count:{},BlobSize:{},PutTimeMicroseconds:{}\n",
                        if result.is_ok() { "OK" } else { "ERR" },
                        Utc::now().timestamp(),
                        worker,
                        count,
                        blob_size,
                        now.elapsed().as_micros()
                    );

                    let mut writer = log_writer.lock().unwrap();
                    writer.write_all(log_message.as_bytes()).unwrap();
                }

                count += 1;

                // Yield to avoid CPU starvation
                thread::yield_now();
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    println!("All threads are done!");
}
