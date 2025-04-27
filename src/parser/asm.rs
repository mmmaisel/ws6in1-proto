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

use super::Ws6in1Data;
use crate::{protocol::Ws6in1DataFrameBase, Error, Ws6in1Container};

/// Holds state for message fragment assembly.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Ws6in1AssemblerBase<V> {
    /// Current fragment number.
    frag_idx: u8,
    /// Message assembly buffer.
    buffer: V,
}

impl<V> Ws6in1AssemblerBase<V> {
    /// Maximum supported assembled message length.
    pub const MAX_MESSAGE_LEN: usize = 256;
}

impl<V: Ws6in1Container<u8>> Ws6in1AssemblerBase<V> {
    /// Adds a message fragment to the internal buffer.
    /// If the full message was received, is parses the data and returns it.
    /// If the full message is not received yet, [None] is returned.
    /// If the new fragment does not belong to a partially received message,
    /// the partial message is discarded and and error is returned.
    pub fn parse<W: Ws6in1Container<u8>>(
        &mut self,
        packet: Ws6in1DataFrameBase<W>,
    ) -> Result<Option<Ws6in1Data>, Error> {
        if self.frag_idx == packet.hdr.frag_idx - 1 {
            self.buffer.append(&packet.payload.data)?;
            self.frag_idx += 1;
        } else {
            let idx = self.frag_idx;
            self.frag_idx = 0;
            self.buffer.clear();
            return Err(Error::FragmentDiscarded { idx });
        }

        if self.frag_idx == packet.hdr.frag_cnt {
            let data = core::str::from_utf8(&self.buffer)
                .map_err(|e| Error::InvalidCharacter {
                    idx: e.valid_up_to(),
                })?
                .try_into()?;
            self.frag_idx = 0;
            self.buffer.clear();

            return Ok(Some(data));
        }

        Ok(None)
    }
}

#[cfg(feature = "std")]
/// A [Ws6in1AssemblerBase] using std [Vec] as storage.
pub type Ws6in1AssemblerStd = Ws6in1AssemblerBase<Vec<u8>>;
#[cfg(feature = "heapless")]
/// A [Ws6in1AssemblerBase] using [heapless::Vec] as storage.
pub type Ws6in1AssemblerHeapless = Ws6in1AssemblerBase<
    heapless::Vec<u8, { Ws6in1AssemblerBase::<()>::MAX_MESSAGE_LEN }>,
>;

#[cfg(feature = "std")]
/// A [Ws6in1AssemblerBase] using default storage based on selected features.
pub type Ws6in1Assembler = Ws6in1AssemblerStd;
#[cfg(not(feature = "std"))]
/// A [Ws6in1AssemblerBase] using default storage based on selected features.
pub type Ws6in1Assembler = Ws6in1AssemblerHeapless;

#[cfg(test)]
mod tests {
    use heapless::Vec;

    use super::*;
    use crate::{
        parser::{Ws6in1ExtData, Ws6in1IndoorData, Ws6in1OutdoorData},
        protocol::{
            Ws6in1DataFrameHeapless, Ws6in1DataHeader, Ws6in1PayloadHeapless,
        },
    };

    #[test]
    fn test_good_data_assembly() {
        let mut asm = Ws6in1AssemblerHeapless::default();

        let frame1 = Ws6in1DataFrameHeapless {
            hdr: Ws6in1DataHeader {
                frag_cnt: 3,
                frag_idx: 1,
                ..Default::default()
            },
            payload: Ws6in1PayloadHeapless {
                data: Vec::from_slice(
                    b"3 2020-01-17 17:30 20.4 49 6.0 60 0.0 0.0 0.0 0.0 129 ",
                )
                .unwrap(),
            },
        };
        let frame2 = Ws6in1DataFrameHeapless {
            hdr: Ws6in1DataHeader {
                frag_cnt: 3,
                frag_idx: 2,
                ..Default::default()
            },
            payload: Ws6in1PayloadHeapless {
                data: Vec::from_slice(
                    b"SE 1017 954 0 -1.2 --.- 27.3 57 33.4 40 --.- -- --.- -",
                )
                .unwrap(),
            },
        };
        let frame3 = Ws6in1DataFrameHeapless {
            hdr: Ws6in1DataHeader {
                frag_cnt: 3,
                frag_idx: 3,
                ..Default::default()
            },
            payload: Ws6in1PayloadHeapless {
                data: Vec::from_slice(b"- --.- -- --.- -- --.- --").unwrap(),
            },
        };

        let expected = Ws6in1Data {
            local_timestamp: 1579282200,
            indoor: Ws6in1IndoorData {
                temperature: 20.4,
                humidity: 49,
                baro_sea: 1017,
                baro_absolute: 954,
            },
            outdoor: Some(Ws6in1OutdoorData {
                temperature: 6.0,
                humidity: 60,
                rain_day: 0.0,
                rain_actual: 0.0,
                wind_actual: 0.0,
                wind_gust: 0.0,
                wind_dir: 129,
                uv_index: 0.0,
                dew_point: -1.2,
            }),
            ext: [
                Some(Ws6in1ExtData {
                    temperature: 27.3,
                    humidity: 57,
                }),
                Some(Ws6in1ExtData {
                    temperature: 33.4,
                    humidity: 40,
                }),
                None,
                None,
                None,
                None,
                None,
            ],
        };

        assert!(asm.parse(frame1).unwrap().is_none());
        assert!(asm.parse(frame2).unwrap().is_none());
        let received = asm.parse(frame3).unwrap().unwrap();

        assert_eq!(expected, received);
        assert_eq!(asm.frag_idx, 0);
        assert!(asm.buffer.is_empty());
    }

    #[test]
    fn test_bad_data_assembly() {
        let mut asm = Ws6in1AssemblerHeapless::default();

        let frame1 = Ws6in1DataFrameHeapless {
            hdr: Ws6in1DataHeader {
                frag_cnt: 3,
                frag_idx: 1,
                ..Default::default()
            },
            payload: Ws6in1PayloadHeapless {
                data: Vec::from_slice(b"foo ").unwrap(),
            },
        };
        let frame2 = Ws6in1DataFrameHeapless {
            hdr: Ws6in1DataHeader {
                frag_cnt: 3,
                frag_idx: 1,
                ..Default::default()
            },
            payload: Ws6in1PayloadHeapless {
                data: Vec::from_slice(b"bar").unwrap(),
            },
        };

        assert!(asm.parse(frame1).unwrap().is_none());
        assert_eq!(asm.frag_idx, 1);

        asm.parse(frame2)
            .expect_err("Invalid packet sequence did not trigger restart");
    }
}
