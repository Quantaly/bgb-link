use crate::commands::*;
use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

pub struct BgbStream<T: Read + Write> {
    inner: T,
}

impl<T: Read + Write> BgbStream<T> {
    pub fn wrap(inner: T) -> BgbStream<T> {
        BgbStream { inner }
    }

    pub fn read_raw(&mut self) -> io::Result<RawBgbCommand> {
        let mut buf = [0u8; 8];
        self.inner.read_exact(&mut buf)?;
        Ok(RawBgbCommand::deserialize(&buf))
    }

    pub fn read(&mut self) -> io::Result<TypedBgbCommand> {
        match TypedBgbCommand::from_raw(&self.read_raw()?) {
            Ok(result) => Ok(result),
            Err(msg) => Err(io::Error::new(io::ErrorKind::InvalidData, msg)),
        }
    }

    pub fn write(&mut self, command: &impl BgbCommand) -> io::Result<()> {
        self.inner.write_all(&command.serialize())
    }
}

impl BgbStream<TcpStream> {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<BgbStream<TcpStream>> {
        let inner = TcpStream::connect(addr)?;
        inner.set_nodelay(true)?;
        Ok(BgbStream { inner })
    }
}
