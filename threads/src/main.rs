extern crate futures;

use std::time::Duration;
use std::thread::{spawn, sleep};
use futures::Future;

fn main() {
    let (tx, rx) = futures::oneshot();

    spawn(move || {
        println!("Thread spawned");
        sleep(Duration::new(1, 0));
        println!("Okay, we're ready!");
        tx.complete("Rust Belt");
    });

    rx.and_then(|name| {
        println!("Hello {}!", name);
        Ok(())
    }).wait().ok();
}
