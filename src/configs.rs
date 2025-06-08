//! HTTP Server Configuration
use std::net::SocketAddr;

#[derive(Debug, Clone)]
/// Configurations for the application.
pub struct Config {
    /// Server bind address
    pub address: SocketAddr,
}

impl Config {
    /// Creates a new configuration with the specified port.
    ///
    /// # Arguments
    ///
    /// * `port` - The address to bind the server to.
    #[must_use]
    pub fn new(port: u16) -> Self {
        Self {
            address: SocketAddr::from(([127, 0, 0, 1], port)),
        }
    }
}

impl Default for Config {
    /// Returns the default configuration with the address set to port 8080.
    fn default() -> Self {
        Self::new(8080)
    }
}
