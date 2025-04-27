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

use byteorder_cursor::Cursor;

use super::{Ws6in1Footer, Ws6in1Serde};
use crate::{Error, Result};

pub(crate) const CMD_LENGTH: usize = 8;
pub(crate) const CMD_TYPE: u8 = 0xFC;

/// Ws6in1 set date command.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct Ws6in1SetDate {
    pub day: u8,
    pub month: u8,
    pub year: u8,
}

impl Ws6in1SetDate {
    pub const OPCODE: u8 = 0x08;
}

impl Ws6in1Serde for Ws6in1SetDate {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(CMD_LENGTH)?;

        buffer.write_u8(CMD_TYPE);
        buffer.write_u8(Self::OPCODE);
        buffer.write_u8(self.year);
        buffer.write_u8(self.month);
        buffer.write_u8(self.day);

        Ws6in1Footer::default().serialize(buffer)
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self>
    where
        Self: Sized,
    {
        buffer.check_remaining(CMD_LENGTH)?;

        let r#type = buffer.read_u8();
        if r#type != CMD_TYPE {
            return Err(Error::UnsupportedType { r#type });
        }

        let opcode = buffer.read_u8();
        if opcode != Self::OPCODE {
            return Err(Error::UnsupportedType { r#type });
        }

        let year = buffer.read_u8();
        let month = buffer.read_u8();
        let day = buffer.read_u8();
        Ws6in1Footer::deserialize(buffer)?;

        Ok(Self { day, month, year })
    }
}

/// Ws6in1 set time command.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct Ws6in1SetTime {
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
}

impl Ws6in1SetTime {
    pub const OPCODE: u8 = 0x09;
}

impl Ws6in1Serde for Ws6in1SetTime {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(CMD_LENGTH)?;

        buffer.write_u8(CMD_TYPE);
        buffer.write_u8(Self::OPCODE);
        buffer.write_u8(self.hour);
        buffer.write_u8(self.min);
        buffer.write_u8(self.sec);

        Ws6in1Footer::default().serialize(buffer)
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self>
    where
        Self: Sized,
    {
        buffer.check_remaining(CMD_LENGTH)?;

        let r#type = buffer.read_u8();
        if r#type != CMD_TYPE {
            return Err(Error::UnsupportedType { r#type });
        }

        let opcode = buffer.read_u8();
        if opcode != Self::OPCODE {
            return Err(Error::UnsupportedType { r#type });
        }

        let hour = buffer.read_u8();
        let min = buffer.read_u8();
        let sec = buffer.read_u8();
        Ws6in1Footer::deserialize(buffer)?;

        Ok(Self { hour, min, sec })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_set_date() {
        let cmd = Ws6in1SetDate {
            day: 17,
            month: 3,
            year: 25,
        };

        let mut buffer = [0u8; 8];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = cmd.serialize(&mut cursor) {
            panic!("Ws6in1SetDate serialization failed: {e:?}");
        }

        let expected = [0xFC, 0x08, 0x19, 0x03, 0x11, 0x00, 0x00, 0xFD];
        assert_eq!(CMD_LENGTH, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_deserialize_set_date() {
        let serialized = [0xFC, 0x08, 0x19, 0x03, 0x11, 0x00, 0x00, 0xFD];

        let expected = Ws6in1SetDate {
            day: 17,
            month: 3,
            year: 25,
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match Ws6in1SetDate::deserialize(&mut cursor) {
            Err(e) => panic!("Ws6in1SetDate deserialization failed: {e:?}"),
            Ok(cmd) => {
                assert_eq!(expected, cmd);
                assert_eq!(CMD_LENGTH, cursor.position());
            }
        };
    }

    #[test]
    fn test_serialize_set_time() {
        let cmd = Ws6in1SetTime {
            hour: 17,
            min: 10,
            sec: 20,
        };

        let mut buffer = [0u8; 8];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = cmd.serialize(&mut cursor) {
            panic!("Ws6in1SetTime serialization failed: {e:?}");
        }

        let expected = [0xFC, 0x09, 0x11, 0x0A, 0x14, 0x00, 0x00, 0xFD];
        assert_eq!(CMD_LENGTH, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_deserialize_set_time() {
        let serialized = [0xFC, 0x09, 0x11, 0x0A, 0x14, 0x00, 0x00, 0xFD];

        let expected = Ws6in1SetTime {
            hour: 17,
            min: 10,
            sec: 20,
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match Ws6in1SetTime::deserialize(&mut cursor) {
            Err(e) => panic!("Ws6in1SetTime deserialization failed: {e:?}"),
            Ok(cmd) => {
                assert_eq!(expected, cmd);
                assert_eq!(CMD_LENGTH, cursor.position());
            }
        };
    }
}
