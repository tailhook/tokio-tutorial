extern crate futures;
extern crate rand;

use std::time::Duration;
use std::thread::{spawn, sleep};
use futures::Future;
use rand::{thread_rng, Rng};

fn main() {
    let (tx1, rx1) = futures::oneshot();
    let (tx2, rx2) = futures::oneshot();

    spawn(move || {
        sleep(Duration::from_millis(thread_rng().gen_range(10, 1000)));
        tx1.complete("Alice");
    });
    spawn(move || {
        sleep(Duration::from_millis(thread_rng().gen_range(10, 1000)));
        tx2.complete("Bob");
    });

    rx1.select(rx2)
    .map(|(name, _)| {
        println!("Hello {}!", name);
    }).wait().ok();
}
