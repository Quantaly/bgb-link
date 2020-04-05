use super::stream::BgbStream;
use crate::commands::*;
use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};

#[derive(Debug)]
pub struct BgbListener {
    inner: TcpListener,
}

impl BgbListener {
    /// Wraps the given `TcpListener` and listens for BGB connections.
    pub fn wrap(inner: TcpListener) -> BgbListener {
        BgbListener { inner }
    }

    /// Accepts a connection and performs the BGB handshake before returning.
    /// Additionally sets TCP_NODELAY as recommended by the spec.
    /// If a bad handshake is received, returns an error of kind `InvalidData`.
    pub fn accept(&self) -> io::Result<(BgbStream<TcpStream>, SocketAddr)> {
        let (stream, addr) = self.inner.accept()?;
        stream.set_nodelay(true)?;
        let mut stream = BgbStream::wrap(stream);
        stream.write(&TypedBgbCommand::Version { valid: true })?;
        if stream.read()? == (TypedBgbCommand::Version { valid: true }) {
            Ok((stream, addr))
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, "bad handshake"))
        }
    }

    /// Returns an `Iterator` equivalent to calling `accept` in a loop, but without
    /// the `SocketAddr` information. (idk why the standard library just did it like that)
    pub fn incoming(&self) -> BgbIncoming {
        BgbIncoming { inner: self }
    }
}

/// Like `std::net::Incoming` but for the `BgbListener`.
pub struct BgbIncoming<'a> {
    inner: &'a BgbListener,
}

impl<'a> Iterator for BgbIncoming<'a> {
    type Item = io::Result<BgbStream<TcpStream>>;

    fn next(&mut self) -> Option<io::Result<BgbStream<TcpStream>>> {
        Some(self.inner.accept().map(|p| p.0))
    }
}
