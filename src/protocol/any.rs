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

#[cfg(feature = "heapless")]
use super::Ws6in1PayloadBase;
use super::{
    cmd, Ws6in1DataFrameBase, Ws6in1Serde, Ws6in1SetDate, Ws6in1SetTime,
};
use crate::{Error, Result, Ws6in1Container};

/// Container that can hold any supported Ws6in1 message.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnyWs6in1MessageBase<V> {
    DataFrame(Ws6in1DataFrameBase<V>),
    SetDate(Ws6in1SetDate),
    SetTime(Ws6in1SetTime),
}

impl<V: Ws6in1Container<u8>> Ws6in1Serde for AnyWs6in1MessageBase<V> {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        match self {
            Self::DataFrame(x) => x.serialize(buffer),
            Self::SetDate(x) => x.serialize(buffer),
            Self::SetTime(x) => x.serialize(buffer),
        }
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self> {
        buffer.check_remaining(1)?;

        let r#type = buffer.peek_u8(0);
        let message = match r#type {
            cmd::CMD_TYPE => {
                buffer.check_remaining(cmd::CMD_LENGTH)?;
                let opcode = buffer.peek_u8(1);
                match opcode {
                    Ws6in1SetDate::OPCODE => {
                        Self::SetDate(Ws6in1SetDate::deserialize(buffer)?)
                    }
                    Ws6in1SetTime::OPCODE => {
                        Self::SetTime(Ws6in1SetTime::deserialize(buffer)?)
                    }
                    opcode => return Err(Error::UnsupportedOpcode { opcode }),
                }
            }
            Ws6in1DataFrameBase::<()>::FRAME_TYPE => {
                Self::DataFrame(Ws6in1DataFrameBase::deserialize(buffer)?)
            }
            r#type => return Err(Error::UnsupportedType { r#type }),
        };

        Ok(message)
    }
}

#[cfg(feature = "std")]
/// An [AnyWs6in1MessageBase] using std [Vec] as storage.
pub type AnyWs6in1MessageStd = AnyWs6in1MessageBase<Vec<u8>>;
#[cfg(feature = "heapless")]
/// An [AnyWs6in1MessageBase] using [heapless::Vec] as storage.
pub type AnyWs6in1MessageHeapless = AnyWs6in1MessageBase<
    heapless::Vec<u8, { Ws6in1PayloadBase::<()>::MAX_PAYLOAD_LEN }>,
>;

#[cfg(feature = "std")]
/// An [AnyWs6in1MessageBase] using default storage based on selected features.
pub type AnyWs6in1Message = AnyWs6in1MessageStd;
#[cfg(not(feature = "std"))]
/// An [AnyWs6in1MessageBase] using default storage based on selected features.
pub type AnyWs6in1Message = AnyWs6in1MessageHeapless;

#[cfg(test)]
mod tests {
    use heapless::Vec;

    use super::*;
    use crate::protocol::{
        Ws6in1DataFrameHeapless, Ws6in1DataHeader, Ws6in1PayloadHeapless,
    };

    #[test]
    fn test_any_set_time_deserialization() {
        #[rustfmt::skip]
        let serialized = [0xFC, 0x09, 0x11, 0x0A, 0x14, 0x00, 0x00, 0xFD];

        let expected = AnyWs6in1MessageHeapless::SetTime(Ws6in1SetTime {
            hour: 17,
            min: 10,
            sec: 20,
        });

        let mut cursor = Cursor::new(&serialized[..]);
        match AnyWs6in1MessageHeapless::deserialize(&mut cursor) {
            Err(e) => panic!("AnyWs6in1Message deserialization failed: {e:?}"),
            Ok(message) => {
                assert_eq!(expected, message);
                assert_eq!(8, cursor.position());
            }
        }
    }

    #[test]
    fn test_any_date_frame_deserialization() {
        #[rustfmt::skip]
        let serialized = b"\xfe\0\0\0\x00163 2020-01-17 17:30 20.4 49 6.0 60 \
            0.0 0.0 0.0 0.0 129 \0\0\xfd";

        let expected =
            AnyWs6in1MessageHeapless::DataFrame(Ws6in1DataFrameHeapless {
                hdr: Ws6in1DataHeader {
                    frag_cnt: 3,
                    frag_idx: 1,
                    ..Default::default()
                },
                payload: Ws6in1PayloadHeapless {
                    data: Vec::from_slice(
                        "3 2020-01-17 17:30 20.4 49 6.0 60 \
                        0.0 0.0 0.0 0.0 129 "
                            .as_bytes(),
                    )
                    .unwrap(),
                },
            });

        let mut cursor = Cursor::new(&serialized[..]);
        match AnyWs6in1MessageHeapless::deserialize(&mut cursor) {
            Err(e) => panic!("AnyWs6in1Message deserialization failed: {e:?}"),
            Ok(message) => {
                assert_eq!(expected, message);
                assert_eq!(64, cursor.position());
            }
        }
    }

    #[test]
    fn test_any_set_time_serialization() {
        let cmd = AnyWs6in1MessageHeapless::SetTime(Ws6in1SetTime {
            hour: 17,
            min: 10,
            sec: 20,
        });

        let mut buffer = [0u8; 8];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = cmd.serialize(&mut cursor) {
            panic!("AnyWs6in1Message serialization failed: {e:?}");
        }

        let expected = [0xFC, 0x09, 0x11, 0x0A, 0x14, 0x00, 0x00, 0xFD];
        assert_eq!(8, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn reject_random_junk() {
        let serialized = [
            0xBA, 0x5B, 0x8C, 0xD9, 0x7A, 0xB5, 0x5D, 0x98, 0x87, 0x99, 0x90,
            0xC3, 0x56, 0x51, 0xFA, 0x16,
        ];

        let mut cursor = Cursor::new(&serialized[..]);
        if let Ok(x) = AnyWs6in1MessageHeapless::deserialize(&mut cursor) {
            panic!("Deserialized junk as {x:?}");
        }
    }

    #[test]
    fn serialize_into_too_small_buffer() {
        let cmd = AnyWs6in1MessageHeapless::SetTime(Ws6in1SetTime {
            hour: 17,
            min: 10,
            sec: 20,
        });

        let mut buffer = [0u8; 7];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Ok(x) = cmd.serialize(&mut cursor) {
            panic!("Serialized message into too small buffer {x:?}");
        }
    }
}
