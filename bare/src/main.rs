extern crate futures;

use futures::Future;

fn main() {
    futures::lazy(|| {
        println!("Hello world!");
        Ok::<(), ()>(())
    }).wait().ok();
}
