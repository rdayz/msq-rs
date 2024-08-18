use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Result};

pub trait ReadPacketExt: ReadBytesExt {
    fn read_u8_veccheck(&mut self, src: &[u8]) -> Result<bool>;
}

impl ReadPacketExt for Cursor<Vec<u8>> {
    fn read_u8_veccheck(&mut self, cmp: &[u8]) -> Result<bool> {
        for cch in cmp {
            let sch = self.read_u8()?;
            if *cch != sch {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

pub trait WritePacketExt: WriteBytesExt {
    fn write_cstring(&mut self, src: &str) -> Result<()>;
}

impl WritePacketExt for Cursor<Vec<u8>> {
    fn write_cstring(&mut self, src: &str) -> Result<()> {
        for ch in src.chars() {
            // increase buffer size to allow encoding CJK or emoji characters
            let mut chu8 = [0; 8];
            let codes = ch.encode_utf8(&mut chu8);
            for code in codes.as_bytes() {
                self.write_u8(*code)?;
            }
        }
        self.write_u8(0x00)?; // 0x00 Terminated
        Ok(())
    }
}
