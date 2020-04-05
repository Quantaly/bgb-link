use crate::commands::*;
use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

pub struct BgbStream<T: Read + Write> {
    inner: T,
}

impl<T: Read + Write> BgbStream<T> {
    /// Takes ownership of the given read/writer and uses it for communication.
    ///
    /// For use over TCP, see `connect`.
    pub fn wrap(inner: T) -> BgbStream<T> {
        BgbStream { inner }
    }

    /// Reads 8 bytes from the connection and interprets the raw command data.
    pub fn read_raw(&mut self) -> io::Result<RawBgbCommand> {
        let mut buf = [0u8; 8];
        self.inner.read_exact(&mut buf)?;
        Ok(RawBgbCommand::deserialize(&buf))
    }

    /// Reads 8 bytes from the connection and interprets them as a command.
    ///
    /// If the command is too malformed to interpret, returns an error of
    /// kind `InvalidData`.
    pub fn read(&mut self) -> io::Result<TypedBgbCommand> {
        match TypedBgbCommand::from_raw(&self.read_raw()?) {
            Ok(result) => Ok(result),
            Err(msg) => Err(io::Error::new(io::ErrorKind::InvalidData, msg)),
        }
    }

    /// Serializes the command to an 8-byte packet and writes it to the stream.
    pub fn write(&mut self, command: &impl BgbCommand) -> io::Result<()> {
        self.inner.write_all(&command.serialize())
    }
}

impl BgbStream<TcpStream> {
    /// Establishes a TCP connection to a listening socket over the BGB protocol.
    ///
    /// This method also enables TCP_NODELAY, as recommended in the spec, and waits for the handshake to
    /// complete before returning. If the other party provides an invalid handshake, returns an error
    /// of kind `InvalidData`.
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<BgbStream<TcpStream>> {
        let inner = TcpStream::connect(addr)?;
        inner.set_nodelay(true)?;
        let mut stream = BgbStream { inner };
        stream.write(&TypedBgbCommand::Version { valid: true })?;
        if stream.read()? == (TypedBgbCommand::Version { valid: true }) {
            Ok(stream)
        } else {
            Err(io::Error::new(io::ErrorKind::InvalidData, "bad handshake"))
        }
    }

    /// Uses `TcpStream.peek` to check if 8 bytes are available, and if so, reads them and
    /// interprets the raw data.
    pub fn maybe_read_raw(&mut self) -> io::Result<Option<RawBgbCommand>> {
        let mut buf = [0u8; 8];
        if self.inner.peek(&mut buf)? == 8 {
            self.inner.read_exact(&mut buf)?;
            Ok(Some(RawBgbCommand::deserialize(&buf)))
        } else {
            Ok(None)
        }
    }

    /// As `read` but for `maybe_read_raw` instead of `read_raw`.
    pub fn maybe_read(&mut self) -> io::Result<Option<TypedBgbCommand>> {
        println!("Running maybe_read");
        if let Some(raw) = self.maybe_read_raw()? {
            match TypedBgbCommand::from_raw(&raw) {
                Ok(result) => Ok(Some(result)),
                Err(msg) => Err(io::Error::new(io::ErrorKind::InvalidData, msg)),
            }
        } else {
            println!("There was no command to read");
            Ok(None)
        }
    }
}
