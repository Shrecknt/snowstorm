use super::{SyncVarintRead, SyncVarintWrite};
use std::io::{Read, Write};

impl<T: Read + ?Sized> SyncVarintRead for T {
    type Error = tokio::io::Error;

    fn read_varint(&mut self) -> Result<i32, Self::Error> {
        Ok(self.read_varint_len()?.1)
    }

    fn read_varint_len(&mut self) -> Result<(u32, i32), Self::Error> {
        let mut buf = [0u8];
        let mut res = 0;
        let mut count = 0u32;

        loop {
            self.read_exact(&mut buf)?;
            res |= (buf[0] as i32 & (0b0111_1111_i32))
                .checked_shl(7 * count)
                .ok_or(tokio::io::ErrorKind::Other)?;

            count += 1;
            if count > 5 {
                break Err(tokio::io::ErrorKind::Other.into());
            } else if (buf[0] & (0b1000_0000_u8)) == 0 {
                break Ok((count, res));
            }
        }
    }
}

impl<T: Write + ?Sized> SyncVarintWrite for T {
    type Error = tokio::io::Error;

    fn write_varint(&mut self, varint: i32) -> Result<(), Self::Error> {
        let mut buffer = [0];
        let mut value = varint;

        if value == 0 {
            self.write_all(&buffer)?;
        }

        while value != 0 {
            buffer[0] = (value & 0b0111_1111) as u8;
            value = (value >> 7) & (i32::max_value() >> 6);
            if value != 0 {
                buffer[0] |= 0b1000_0000;
            }
            self.write_all(&buffer)?;
        }

        Ok(())
    }
}
