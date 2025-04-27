/******************************************************************************\
    ws6in1-proto - A protocol library for CC8488 compatible weather stations
    Copyright (C) 2025 Max Maisel

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
\******************************************************************************/

use byteorder_cursor::{BigEndian, Cursor};

use crate::{Error, Result};

/// Interface for (de)serialization of Ws6in1 messages.
pub trait Ws6in1Serde {
    /// Serialize given object into buffer.
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()>;
    /// Deserialize buffer into object.
    /// The supplied slice must contain exactly one packet.
    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self>
    where
        Self: Sized;
}

/// Footer marker at the and of an Ws6in1 frame.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub(crate) struct Ws6in1Footer {}

impl Ws6in1Footer {
    pub const LENGTH: usize = 3;
    pub const MAGIC: u8 = 0xFD;

    pub fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(Self::LENGTH)?;
        // Checksum is ignored by device.
        buffer.write_u16::<BigEndian>(0);
        buffer.write_u8(Self::MAGIC);

        Ok(())
    }

    pub fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(Self::LENGTH)?;

        let _crc = buffer.read_u16::<BigEndian>();
        let magic = buffer.read_u8();
        if magic != Self::MAGIC {
            return Err(Error::InvalidMagic { magic });
        }

        let trailing = buffer.remaining();
        if trailing != 0 {
            return Err(Error::BufferNotConsumed { trailing });
        }

        Ok(Self {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_footer() {
        let cmd = Ws6in1Footer {};

        let mut buffer = [0u8; 3];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = cmd.serialize(&mut cursor) {
            panic!("Ws6in1Footer serialization failed: {e:?}");
        }

        let expected = [0x00, 0x00, 0xFD];
        assert_eq!(Ws6in1Footer::LENGTH, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn deserialize_footer() {
        let serialized = [0x10u8, 0x20, 0xfd];

        let expected = Ws6in1Footer {};

        let mut cursor = Cursor::new(&serialized[..]);
        match Ws6in1Footer::deserialize(&mut cursor) {
            Err(e) => panic!("Ws6in1Footer deserialization failed: {e:?}"),
            Ok(cmd) => {
                assert_eq!(expected, cmd);
                assert_eq!(Ws6in1Footer::LENGTH, cursor.position());
            }
        };
    }
}
