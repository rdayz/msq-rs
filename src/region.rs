use std::io::{Error, ErrorKind, Result};

/// Region enum to restrict the servers region the query searches for
///
/// * Intended to be used with: [`MSQClient`](crate::MSQClient) and
/// [`MSQClientBlock`](crate::MSQClientBlock)
///
/// # Reference
/// | `Region` Enum          | Region            | Byte |
/// | ---------------------- | ----------------- | ---- |
/// | `Region::USEast`       | US East coast     | 0x00 |
/// | `Region::USWest`       | US West coast     | 0x01 |
/// | `Region::SouthAmerica` | South America     | 0x02 |
/// | `Region::Europe`       | Europe            | 0x03 |
/// | `Region::Asia`         | Asia              | 0x04 |
/// | `Region::Australia`    | Australia         | 0x05 |
/// | `Region::MiddleEast`   | Middle East       | 0x06 |
/// | `Region::Africa`       | Africa            | 0x07 |
/// | `Region::All`          | Everyone | 0xFF |
///
#[derive(Debug, PartialEq)]
pub enum Region {
    USEast,
    USWest,
    SouthAmerica,
    Europe,
    Asia,
    Australia,
    MiddleEast,
    Africa,
    All,
}

impl Region {
    /// Return raw u8 byte code of its specified region
    ///
    /// # Example
    /// ```rust
    /// use msq::Region;
    ///
    /// let region_hex_str = format!("{:#04x}", Region::All.as_u8());
    /// assert_eq!(&region_hex_str, "0xff");
    ///
    /// let region_hex_str = format!("{:#04x}", Region::Europe.as_u8());
    /// assert_eq!(&region_hex_str, "0x03");
    /// ```
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::USEast => 0x00,
            Self::USWest => 0x01,
            Self::SouthAmerica => 0x02,
            Self::Europe => 0x03,
            Self::Asia => 0x04,
            Self::Australia => 0x05,
            Self::MiddleEast => 0x06,
            Self::Africa => 0x07,
            Self::All => 0xFF,
        }
    }

    pub fn from_u8(code: u8) -> Result<Self> {
        match code {
            0x00 => Ok(Self::USEast),
            0x01 => Ok(Self::USWest),
            0x02 => Ok(Self::SouthAmerica),
            0x03 => Ok(Self::Europe),
            0x04 => Ok(Self::Asia),
            0x05 => Ok(Self::Australia),
            0x06 => Ok(Self::MiddleEast),
            0x07 => Ok(Self::Africa),
            0xFF => Ok(Self::All),
            _ => Err(Error::new(ErrorKind::Other, "Invalid code")),
        }
    }
}
