use std::{
    net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs},
    num::NonZeroUsize,
};

/// Default to localhost for IPv4 address.
pub const DEFAULT_IPV4: &str = "127.0.0.1";

/// Magic number for the default port.
pub const DEFAULT_PORT: u16 = 7828;

/// Define a sane maximum payload size for the client.
pub const MAX_PAYLOAD_SIZE: usize = 1024;

/// Helper struct to store an IP address and port to uniquely identify an address.
#[derive(Clone, Debug)]
pub struct Address {
    pub ip: IpAddr,
    pub port: u16,
}

/// Implement trait to display and `Address` in a human-readable format.
impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &Address { ip, port } = self;
        write!(f, "{ip}:{port}")
    }
}

/// Helper for using an `Address` with `std::net` functions.
impl ToSocketAddrs for Address {
    type Iter = std::option::IntoIter<SocketAddr>;
    fn to_socket_addrs(&self) -> std::io::Result<std::option::IntoIter<SocketAddr>> {
        let &Address { ip, port } = self;
        match ip {
            IpAddr::V4(ip) => (ip, port).to_socket_addrs(),
            IpAddr::V6(ip) => (ip, port).to_socket_addrs(),
        }
    }
}

/// The error type when reading from a TCP stream with `read_stream(...)`.
#[derive(Debug)]
pub enum ReadStreamError {
    /// The TCP stream has been closed.
    ConnectionClosed,

    /// Error reading from stream. Guaranteed not to be `WouldBlock`.
    IoError(std::io::Error),
}

/// Helper to read a positive number of bytes from a TCP or safely return an error.
///
/// # Errors
/// * Returns a `ReadStreamError::ConnectionClosed` if the TCP stream has been closed.
/// * Returns a `ReadStreamError::IoError` if an error occurs while reading from the stream. Guaranteed not to be `WouldBlock`.
pub fn read_stream(
    stream: &mut TcpStream,
    buf: &mut [u8],
) -> Result<NonZeroUsize, ReadStreamError> {
    use std::io::Read as _;
    loop {
        let size = match stream.read(buf) {
            // If we successfully got a number of bytes read, check that it is non-zero.
            Ok(size) => {
                match NonZeroUsize::try_from(size) {
                    // Return the valid buffer size.
                    Ok(size) => size,

                    // Zero bytes indicates that the TCP stream has been closed.
                    Err(_) => return Err(ReadStreamError::ConnectionClosed),
                }
            }
            Err(e) => {
                match e.kind() {
                    // If we're not ready to read, just continue.
                    std::io::ErrorKind::WouldBlock => continue,

                    // Otherwise, something went wrong.
                    _ => return Err(ReadStreamError::IoError(e)),
                }
            }
        };

        return Ok(size);
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }