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

use super::{Ws6in1Footer, Ws6in1Serde};
use crate::{Error, Result, Ws6in1Container};

/// Ws61in data frame header.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct Ws6in1DataHeader {
    /// Total item count in this stream, usually zero.
    pub item_cnt: u16,
    /// Item index, usually zero.
    pub item_idx: u16,
    /// Total fragment count of this item.
    pub frag_cnt: u8,
    /// One based fragment index.
    pub frag_idx: u8,
}

impl Ws6in1DataHeader {
    pub const LENGTH: usize = 5;
}

impl Ws6in1Serde for Ws6in1DataHeader {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(Self::LENGTH)?;

        buffer.write_u16::<BigEndian>(self.item_cnt);
        buffer.write_u16::<BigEndian>(self.item_idx);
        buffer.write_u8(self.frag_cnt << 4 | self.frag_idx);

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self>
    where
        Self: Sized,
    {
        buffer.check_remaining(Self::LENGTH)?;

        let item_cnt = buffer.read_u16::<BigEndian>();
        let item_idx = buffer.read_u16::<BigEndian>();

        let frag = buffer.read_u8();
        let frag_cnt = (frag & 0xF0) >> 4;
        let frag_idx = frag & 0x0F;

        Ok(Self {
            item_cnt,
            item_idx,
            frag_cnt,
            frag_idx,
        })
    }
}

/// Ws6in1 payload fragment.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Ws6in1PayloadBase<V> {
    pub data: V,
}

impl<V> Ws6in1PayloadBase<V> {
    pub const LENGTH: usize = 55;
    pub const MAX_PAYLOAD_LEN: usize = 54;
}

impl<V: Ws6in1Container<u8>> Ws6in1Serde for Ws6in1PayloadBase<V> {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(Self::LENGTH)?;

        let len = self.data.len();
        buffer.write_u8(
            len.try_into().map_err(|_| Error::PayloadTooLarge { len })?,
        );
        buffer.write_bytes(&self.data);
        for _ in 0..(Self::MAX_PAYLOAD_LEN - len) {
            buffer.write_u8(0);
        }

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self>
    where
        Self: Sized,
    {
        buffer.check_remaining(Self::LENGTH)?;

        let len = buffer.read_u8() as usize;
        let mut payload = [0; Ws6in1PayloadBase::<()>::MAX_PAYLOAD_LEN];
        buffer.read_bytes(&mut payload);

        Ok(Self {
            data: V::from_slice(&payload[0..len]),
        })
    }
}

#[cfg(feature = "std")]
/// A [Ws6in1PayloadBase] using std [Vec] as storage.
pub type Ws6in1PayloadStd = Ws6in1PayloadBase<Vec<u8>>;
#[cfg(feature = "heapless")]
/// A [Ws6in1PayloadBase] using [heapless::Vec] as storage.
pub type Ws6in1PayloadHeapless = Ws6in1PayloadBase<
    heapless::Vec<u8, { Ws6in1PayloadBase::<()>::MAX_PAYLOAD_LEN }>,
>;

#[cfg(feature = "std")]
/// A [Ws6in1PayloadBase] using default storage based on selected features.
pub type Ws6in1Payload = Ws6in1PayloadStd;
#[cfg(not(feature = "std"))]
/// A [Ws6in1PayloadBase] using default storage based on selected features.
pub type Ws6in1Payload = Ws6in1PayloadHeapless;

/// Ws6in1 data frame.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Ws6in1DataFrameBase<V> {
    pub hdr: Ws6in1DataHeader,
    pub payload: Ws6in1PayloadBase<V>,
}

impl<V> Ws6in1DataFrameBase<V> {
    pub const LENGTH: usize = 64;
    pub const FRAME_TYPE: u8 = 0xFE;
}

impl<V: Ws6in1Container<u8>> Ws6in1Serde for Ws6in1DataFrameBase<V> {
    fn serialize(&self, buffer: &mut Cursor<&mut [u8]>) -> Result<()> {
        buffer.check_remaining(Self::LENGTH)?;

        buffer.write_u8(Self::FRAME_TYPE);
        self.hdr.serialize(buffer)?;
        self.payload.serialize(buffer)?;
        Ws6in1Footer::default().serialize(buffer)?;

        Ok(())
    }

    fn deserialize(buffer: &mut Cursor<&[u8]>) -> Result<Self>
    where
        Self: Sized,
    {
        buffer.check_remaining(Self::LENGTH)?;

        let r#type = buffer.read_u8();
        if r#type != Self::FRAME_TYPE {
            return Err(Error::UnsupportedType { r#type });
        }

        let hdr = Ws6in1DataHeader::deserialize(buffer)?;
        let payload = Ws6in1PayloadBase::deserialize(buffer)?;
        Ws6in1Footer::deserialize(buffer)?;

        Ok(Self { hdr, payload })
    }
}

#[cfg(feature = "std")]
/// A [Ws6in1DataFrameBase] using std [Vec] as storage.
pub type Ws6in1DataFrameStd = Ws6in1DataFrameBase<Vec<u8>>;
#[cfg(feature = "heapless")]
/// A [Ws6in1DataFrameBase] using [heapless::Vec] as storage.
pub type Ws6in1DataFrameHeapless = Ws6in1DataFrameBase<
    heapless::Vec<u8, { Ws6in1PayloadBase::<()>::MAX_PAYLOAD_LEN }>,
>;

#[cfg(feature = "std")]
/// A [Ws6in1DataFrameBase] using default storage based on selected features.
pub type Ws6in1DataFrame = Ws6in1DataFrameStd;
#[cfg(not(feature = "std"))]
/// A [Ws6in1DataFrameBase] using default storage based on selected features.
pub type Ws6in1DataFrame = Ws6in1DataFrameHeapless;

#[cfg(test)]
mod tests {
    use heapless::Vec;

    use super::*;

    #[test]
    fn test_serialize_data_header() {
        let cmd = Ws6in1DataHeader {
            item_cnt: 2,
            item_idx: 1,
            frag_cnt: 3,
            frag_idx: 1,
        };

        let mut buffer = [0u8; 5];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = cmd.serialize(&mut cursor) {
            panic!("Ws6in1DataHeader serialization failed: {e:?}");
        }

        let expected = [0x00, 0x02, 0x00, 0x01, 0x31];
        assert_eq!(5, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_deserialize_data_header() {
        let serialized = [0x00, 0x02, 0x00, 0x01, 0x31];

        let expected = Ws6in1DataHeader {
            item_cnt: 2,
            item_idx: 1,
            frag_cnt: 3,
            frag_idx: 1,
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match Ws6in1DataHeader::deserialize(&mut cursor) {
            Err(e) => panic!("Ws6in1DataHeader deserialization failed: {e:?}"),
            Ok(cmd) => {
                assert_eq!(expected, cmd);
                assert_eq!(Ws6in1DataHeader::LENGTH, cursor.position());
            }
        };
    }

    #[test]
    fn test_serialize_payload() {
        let cmd = Ws6in1PayloadHeapless {
            data: Vec::from_slice(&[1, 2, 3, 4]).unwrap(),
        };

        let mut buffer = [0u8; 55];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = cmd.serialize(&mut cursor) {
            panic!("Ws6in1Payload serialization failed: {e:?}");
        }

        #[rustfmt::skip]
        let expected = [
            0x04, 0x01, 0x02, 0x03, 0x04,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(55, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_deserialize_payload() {
        #[rustfmt::skip]
        let serialized = [
            0x04, 0x01, 0x02, 0x03, 0x04,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let expected = Ws6in1PayloadHeapless {
            data: Vec::from_slice(&[1, 2, 3, 4]).unwrap(),
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match Ws6in1PayloadHeapless::deserialize(&mut cursor) {
            Err(e) => panic!("Ws6in1Payload deserialization failed: {e:?}"),
            Ok(payload) => {
                assert_eq!(expected, payload);
                assert_eq!(Ws6in1Payload::LENGTH, cursor.position());
            }
        };
    }

    #[test]
    fn test_serialize_data() {
        let cmd = Ws6in1DataFrameHeapless {
            hdr: Ws6in1DataHeader {
                frag_cnt: 3,
                frag_idx: 1,
                ..Default::default()
            },
            payload: Ws6in1PayloadHeapless {
                data: Vec::from_slice(
                    b"3 2020-01-17 17:30 20.4 49 6.0 60 \
                        0.0 0.0 0.0 0.0 129 ",
                )
                .unwrap(),
            },
        };

        let mut buffer = [0u8; 64];
        let mut cursor = Cursor::new(&mut buffer[..]);

        if let Err(e) = cmd.serialize(&mut cursor) {
            panic!("Ws6in1DataFrame serialization failed: {e:?}");
        }

        let expected = Vec::<u8, 64>::from_slice(
            b"\xfe\0\0\0\x00163 2020-01-17 17:30 20.4 49 6.0 60 \
                0.0 0.0 0.0 0.0 129 \0\0\xfd",
        )
        .unwrap();
        assert_eq!(64, cursor.position());
        assert_eq!(expected, buffer);
    }

    #[test]
    fn test_deserialize_data() {
        #[rustfmt::skip]
        let serialized = b"\xfe\0\0\0\x00163 2020-01-17 17:30 20.4 49 6.0 60 \
            0.0 0.0 0.0 0.0 129 \0\0\xfd";

        let expected = Ws6in1DataFrameHeapless {
            hdr: Ws6in1DataHeader {
                frag_cnt: 3,
                frag_idx: 1,
                ..Default::default()
            },
            payload: Ws6in1PayloadHeapless {
                data: Vec::from_slice(
                    b"3 2020-01-17 17:30 20.4 49 6.0 60 \
                        0.0 0.0 0.0 0.0 129 ",
                )
                .unwrap(),
            },
        };

        let mut cursor = Cursor::new(&serialized[..]);
        match Ws6in1DataFrameHeapless::deserialize(&mut cursor) {
            Err(e) => panic!("Ws6in1DataFrame deserialization failed: {e:?}"),
            Ok(payload) => {
                assert_eq!(expected, payload);
                assert_eq!(Ws6in1DataFrame::LENGTH, cursor.position());
            }
        };
    }
}
