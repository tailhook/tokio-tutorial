use std::io::Write;
use tokio_core::easy::Serialize;

pub struct RedisSerialize;

pub enum Response {
    Okay,
    Bulk(Vec<u8>),
    Err(&'static str),
}


impl Serialize for RedisSerialize {
    type In = Response;
    fn serialize(&mut self, msg: Self::In, buf: &mut Vec<u8>) {
        use self::Response::*;
        match msg {
            Okay => {
                write!(buf, "+OK\r\n").unwrap();
            }
            Err(text) => {
                write!(buf, "-{}\r\n", text).unwrap();
            }
            Bulk(bulk) => {
                write!(buf, "${}\r\n", bulk.len()).unwrap();
                buf.extend(bulk);
                buf.extend(b"\r\n");
            }
        }
    }
}
