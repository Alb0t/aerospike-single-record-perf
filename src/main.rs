#[macro_use]
extern crate aerospike;
extern crate chrono;
use chrono::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{env, thread::sleep};

use aerospike::{Client, ClientPolicy, WritePolicy};
use rand::{self, Rng, RngCore};
use std::thread;

fn main() {
    let cpolicy = ClientPolicy {
        use_services_alternate: true,
        ..Default::default()
    };

    let hosts = env::var("AEROSPIKE_HOSTS").unwrap_or_else(|_| String::from("127.0.0.1:3100"));

    // Wrap the Aerospike client in an Arc for thread-safe shared access
    let client = Arc::new(Client::new(&cpolicy, &hosts).expect("Failed to connect to cluster"));

    let thread_count = 64; // Number of threads to spawn

    // Store thread handles in a vector
    let mut handles = Vec::new();

    let start_epoch_time = Utc::now().timestamp();
    for worker in 0..thread_count {
        let client = Arc::clone(&client); // Clone Arc for each thread

        let handle = thread::spawn(move || {
            let mut count = 1; // Initialize the counter outside the loop
            let mut blob_size: usize = 1; // Initial blob size
            let key = as_key!("test", "test", "test");

            loop {
                let elapsed_seconds = Utc::now().timestamp() - start_epoch_time;
                // Create a buffer with the specified size and fill it with random bytes
                let mut data = vec![0u8; blob_size];
                rand::thread_rng().fill_bytes(&mut data);

                let bin = aerospike::Bin::new("name", aerospike::Value::Blob(data.clone()));
                let now = Instant::now();

                // Attempt to write to the Aerospike cluster
                match client.put(&WritePolicy::default(), &key, &[bin]) {
                    Ok(_) => {
                        if count % 100 == 0 {
                            let epoch_time = Utc::now().timestamp();
                            println!(
                                "Result:OK,EpochTime:{},Thread:{},Count:{},BlobSize:{},PutTimeMicroseconds:{:?}",
                                epoch_time,
                                worker,
                                count,
                                blob_size,
                                Duration::as_micros(&now.elapsed())
                            );
                        }
                    }
                    Err(e) => {
                        if count % 100 == 0 {
                            let epoch_time = Utc::now().timestamp();
                            println!("Result:ERR,EpochTime:{},Thread:{},Count:{},BlobSize:{},PutTimeMicroseconds:{:?}",
                                epoch_time,
                                worker,
                                count,
                                blob_size,
                                Duration::as_micros(&now.elapsed())
                            );
                        }
                    }
                }

                // Update counters
                count += 1;
                blob_size = (elapsed_seconds * 1024) as usize; // Increase blob size for the next iteration
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    println!("All threads are done!");
}
