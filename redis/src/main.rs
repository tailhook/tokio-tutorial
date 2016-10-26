extern crate void;
extern crate futures;
extern crate env_logger;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
#[macro_use] extern crate log;

use std::io;

use futures::{Future, finished};
use futures::stream::Stream;
use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;
use tokio_core::easy::EasyFramed;
use tokio_proto::easy::pipeline;
use tokio_service::simple_service;

mod serialize;
mod deserialize;

use serialize::{Response, RedisSerialize};
use deserialize::{Request, RedisDeserialize};


pub fn main() {
    env_logger::init().unwrap();

    let mut lp = Core::new().unwrap();

    // The address to bind the listener socket to
    let addr = "127.0.0.1:12345".parse().unwrap();

    let service = simple_service(move |req: Request| {
        Ok(Response::Okay)
    });

    // Create the new TCP listener
    let listener = TcpListener::bind(&addr, &lp.handle()).unwrap();
    println!("Redis server running on {}", addr);

    let handle = lp.handle();
    let srv = listener.incoming().for_each(|(socket, _addr)| {
        handle.spawn(
            pipeline::EasyServer::new(service.clone(),
                 EasyFramed::new(socket, RedisDeserialize, RedisSerialize))
            .map_err(|e| debug!("Connection error: {}", e)));
        Ok(())
    });

    lp.run(srv).unwrap();
}
