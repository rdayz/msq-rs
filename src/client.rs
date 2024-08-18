use crate::filter::Filter;
use crate::region::Region;

use crate::packet_ext::{ReadPacketExt, WritePacketExt};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use tokio::time::sleep;
use std::io::{Cursor, Error, ErrorKind, Result};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

/// The primary MSQ client driver (async)
///
/// * Requires feature: `async` (Turned **on** by default)
/// * Intended to be used with [`Filter`] and [`Region`].
/// * This uses the [`tokio`] asynchronous UDP Socket to achieve an
/// async MSQ client driver.
/// * The non-async/blocking version of this: [`MSQClientBlock`](crate::MSQClientBlock)
///
/// ## Quick Start
/// ```rust
/// use msq::{MSQClient, Region, Filter};
/// use std::io::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let mut client = MSQClient::new().await?;
///     client.connect("hl2master.steampowered.com:27011").await?;
///
///     let servers = client
///         .query(Region::Europe,  // Restrict query to Europe region
///             Filter::new()       // Create a Filter builder
///                 .appid(240)     // appid of 240 (CS:S)
///                 .nand()         // Start of NAND special filter
///                     .map("de_dust2")     // Map is de_dust2
///                     .empty(true)         // Server is empty
///                 .end()          // End of NAND special filter
///                 .gametype(&vec!["friendlyfire", "alltalk"])).await?;
///     Ok(())
/// }
/// ```
pub struct MSQClient {
    sock: UdpSocket,
}

#[derive(PartialEq, Default, Clone)]
pub struct Address {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
}

const EMPTY_ADRESS: Address = Address {
    a: 0,
    b: 0,
    c: 0,
    d: 0,
};

impl MSQClient {
    /// Create a new MSQClient variable and binds the UDP socket to `0.0.0.0:0`
    pub async fn new() -> Result<MSQClient> {
        let sock = UdpSocket::bind("0.0.0.0:0").await?;
        Ok(MSQClient { sock })
    }

    /// Connect the client to the given master server address/hostname
    ///
    /// # Arguments
    /// * `master_server_addr` - The master server's hostname/ip address
    ///
    /// # Example
    /// ```
    /// use msq::MSQClient;
    /// use std::io::Result;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let mut client = MSQClient::new().await?;
    ///     client.connect("hl2master.steampowered.com:27011").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect(&mut self, master_server_addr: &str) -> Result<()> {
        self.sock.connect(master_server_addr).await?;
        Ok(())
    }

    /// Query with raw bytes
    ///
    /// # Arguments
    /// * `region_code` - Region code in u8 (`0x00 - 0x07 / 0xFF`)
    /// * `filter_str` - Filter in plain string (EX: `\\appid\\240\\map\\de_dust2`)
    pub async fn query_raw(
        &mut self,
        region_code: u8,
        filter_str: &str,
        sender: Sender<(Address, u16)>,
    ) -> Result<()> {
        self.send(region_code, filter_str, EMPTY_ADRESS, 0).await?; // First Packet
        self.recv(region_code, filter_str, sender).await
    }

    /// Query with specified Region and Filter
    ///
    /// Returns a Vec list of IP addresses in strings
    ///
    /// # Arguments
    /// * `region` - [`Region`] enum (`Region::USEast` - `Region::Africa` / `Region::All`)
    /// * `filter` - [`Filter`] builder (EX: `Filter::new().appid(240).map("de_dust2")`)
    pub async fn query(
        &mut self,
        region: Region,
        filter: Filter,
        sender: Sender<(Address, u16)>,
    ) -> Result<()> {
        self.query_raw(region.as_u8(), &filter.as_string(), sender)
            .await
    }

    async fn send(
        &mut self,
        region_code: u8,
        filter_str: &str,
        address: Address,
        port: u16,
    ) -> Result<()> {
        let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::default());
        cursor.write_u8(0x31)?;
        cursor.write_u8(region_code)?;
        cursor.write_cstring(&format!(
            "{}.{}.{}.{}:{}",
            address.a, address.b, address.c, address.d, port
        ))?;
        cursor.write_cstring(filter_str)?;
        self.sock.send(cursor.get_ref()).await?;
        Ok(())
    }

    async fn recv(
        &mut self,
        region_code: u8,
        filter_str: &str,
        sender: Sender<(Address, u16)>,
    ) -> Result<()> {
        let mut buf: [u8; 2048] = [0x00; 2048];
        let mut last_address: Address = Address::default();
        let mut last_port: u16 = 0;
        let mut end_of_list = false;
        while !end_of_list {
            let len = self.sock.recv(&mut buf).await?;
            let mut cursor = Cursor::new(buf[..len].to_vec());
            if cursor.read_u8_veccheck(&[0xFF, 0xFF, 0xFF, 0xFF, 0x66, 0x0A])? {
                while let Ok(a) = cursor.read_u8() {
                    let address = Address {
                        a,
                        b: cursor.read_u8()?,
                        c: cursor.read_u8()?,
                        d: cursor.read_u8()?,
                    };

                    if address == EMPTY_ADRESS {
                        end_of_list = true;
                        break;
                    }

                    let port = cursor.read_u16::<BigEndian>()?;
                    sender.send((address.clone(), port)).await.unwrap();

                    last_address = address;
                    last_port = port;
                }
            } else {
                return Err(Error::new(ErrorKind::Other, "Mismatched starting sequence"));
            }

            sleep(Duration::from_secs(6)).await;

            self.send(region_code, filter_str, last_address.clone(), last_port)
                .await?;
        }

        Ok(())
    }
}
