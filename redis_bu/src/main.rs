extern crate void;
extern crate futures;
extern crate env_logger;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
#[macro_use] extern crate log;

use std::sync::Mutex;
use std::collections::HashMap;

use futures::Future;
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
    let addr = "0.0.0.0:7001".parse().unwrap();
    let table = Mutex::new(HashMap::<_, Vec<u8>>::new());

    let service = simple_service(move |req: Request| {
        match req.get(0).map(|x| (x.as_slice(), req.len())) {
            Some((b"GET", 2)) => {
                Ok(Response::Bulk(table.lock().unwrap()
                    .get(req[1].as_ref())
                    .map(|vec| vec.clone())
                    .unwrap_or(Vec::new())))
            }
            Some((b"SET", 3)) => {
                table.lock().unwrap()
                    .insert(req[1].as_ref().to_vec(),
                            req[2].as_ref().to_vec());
                Ok(Response::Okay)
            }
            _ => {
                Ok(Response::Err("Invalid command"))
            }
        }
    });

    // Create the new TCP listener
    let listener = TcpListener::bind(&addr, &lp.handle()).unwrap();
    println!("Redis server running on {}", addr);

    let handle = lp.handle();
    let srv = listener.incoming().map(|(socket, _addr)| {
        pipeline::EasyServer::new(service.clone(),
             EasyFramed::new(socket, RedisDeserialize, RedisSerialize))
        .then(|_| Ok(()))  // must be always Ok() or it crashes
    }).buffer_unordered(1000000).for_each(|()| Ok(()));

    lp.run(srv).unwrap();
}
