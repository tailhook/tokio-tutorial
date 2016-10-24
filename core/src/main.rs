extern crate futures;
extern crate tokio_core;

use std::io::{stdout, Write};
use std::time::Duration;
use futures::Future;
use tokio_core::reactor::{Core, Timeout};

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let future = Timeout::new(Duration::new(2, 0), &handle).unwrap()
        .and_then(|_| {
            print!("Hello ");
            try!(stdout().flush());
            Ok(())
        })
        .and_then(|_| Timeout::new(Duration::new(2, 0), &handle).unwrap())
        .and_then(|_| {
            println!("world!");
            Ok(())
        });
    core.run(future).unwrap();
}
