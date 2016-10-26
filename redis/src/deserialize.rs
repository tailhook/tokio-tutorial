use std::io;
use std::str::from_utf8;

use futures::{Async, Poll};
use tokio_core::easy::{EasyBuf, Parse};


pub type Request = Vec<EasyBuf>;

pub struct RedisDeserialize;

fn error<V>(reason: &'static str) -> Result<V, io::Error> {
     Err(io::Error::new(io::ErrorKind::Other, reason))
}

impl RedisDeserialize {
    /// Parses input buffer and returns each redis bulk (string) as a slice
    /// of start and stop index in the original buffer
    ///
    /// Returns None if we can't read the whole request yet
    fn parse_slices(&self, data: &[u8])
        -> Result<Option<Vec<(usize, usize)>>, io::Error>
    {
        if data[0] != b'*' {
            return error("request must be bulk");
        }
        let endline = data.iter().position(|&x| x == b'\n');
        let (array_size, mut offset) = if let Some(end) = endline {
            let strdata = from_utf8(&data[1..end]).ok();
            match strdata.and_then(|x| x.trim_right().parse().ok()) {
                Some(array_size) => (array_size, end+1),
                None => return error("invalid array size"),
            }
        } else {
            return Ok(None);
        };
        // TODO(tailhook) we should limit the array size or our server is
        // open for a DoS attack
        let mut slices = Vec::with_capacity(array_size);

        for _ in 0..array_size {
            if data[offset] != b'$' {
                return error("array item is not a string");
            }
            let endline = data[offset..].iter().position(|&x| x == b'\n');
            let (bulk_size, noff) = if let Some(end) = endline {
                let strdata = from_utf8(&data[offset+1..offset+end]).ok();
                match strdata.and_then(|x| x.trim_right().parse().ok()) {
                    Some(bulk_size) => (bulk_size, offset+end+1),
                    None => return error("invalid bulk size"),
                }
            } else {
                return Ok(None);
            };
            if data.len() < noff + bulk_size + 2 {
                return Ok(None);
            }
            offset = noff + bulk_size + 2;
            if &data[offset-2..offset] != b"\r\n" {
                return error("bulk must be followed by CR LF");
            }
            slices.push((noff, bulk_size));
        }
        return Ok(Some(slices));
    }
}

impl Parse for RedisDeserialize {
    type Out = Request;
    fn parse(&mut self, buf: &mut EasyBuf) -> Poll<Request, io::Error> {
        if buf.len() == 0 {
            return Ok(Async::NotReady);
        }

        // Firstly we find out if we read all the parts of a request,
        // while storing only offsets of the data in the buffer
        let slices = match self.parse_slices(buf.as_slice()) {
            Ok(Some(slices)) => slices,
            Ok(None) => return Ok(Async::NotReady),
            Err(e) => return Err(e),
        };

        // All slices are successfully parsed, let's make a request with
        // the real slices
        let mut result = Vec::with_capacity(slices.len());
        let mut cur = 0;
        for (start, size) in slices {
            if cur < start {
                buf.drain_to(start - cur);
            }
            result.push(buf.drain_to(size));
            cur = start + size;
        }
        buf.drain_to(2); // remove first two bytes
        return Ok(Async::Ready(result));
    }
    fn done(&mut self, _buf: &mut EasyBuf) -> Result<Request, io::Error> {
        error("data left in the stream on EOF")
    }
}
