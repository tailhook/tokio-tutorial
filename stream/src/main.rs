extern crate futures;
extern crate tokio_core;

use std::io::{stdout, Write};
use std::time::Duration;
use futures::Future;
use futures::stream::{iter, Stream};
use tokio_core::reactor::{Core, Timeout};

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let future = iter("Hello world!".chars().map(|x| Ok(x)))
        .and_then(|letter| {
            Timeout::new(Duration::from_millis(100), &handle).unwrap()
            .map(move |()| letter)
        })
        .for_each(|letter| {
            print!("{}", letter);
            try!(stdout().flush());
            Ok(())
        })
        .map(|_| {
            println!("");
        });
    core.run(future).unwrap();
}
