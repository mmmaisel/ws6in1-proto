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

//! Ws6in1 message parser. Data is parsed through [TryFrom].

use core::str::SplitWhitespace;

use time::{
    format_description::BorrowedFormatItem, macros::format_description, Date,
    PrimitiveDateTime, Time,
};

use super::{Error, Result};

mod asm;
#[cfg(feature = "heapless")]
pub use asm::Ws6in1AssemblerHeapless;
#[cfg(feature = "std")]
pub use asm::Ws6in1AssemblerStd;
pub use asm::{Ws6in1Assembler, Ws6in1AssemblerBase};

/// Data from the indoor console.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ws6in1IndoorData {
    pub temperature: f32,
    pub humidity: u8,
    pub baro_sea: u16,
    pub baro_absolute: u16,
}

/// Data from main outdoor sensor.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ws6in1OutdoorData {
    pub temperature: f32,
    pub humidity: u8,
    pub rain_day: f32,
    pub rain_actual: f32,
    pub wind_actual: f32,
    pub wind_gust: f32,
    pub wind_dir: u16,
    pub uv_index: f32,
    pub dew_point: f32,
}

/// Data from an extra sensor.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ws6in1ExtData {
    pub temperature: f32,
    pub humidity: u8,
}

/// Parsed weather data from a Ws6in1 compatible weather station.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ws6in1Data {
    /// Unix timestamp in local timezone and with 60 seconds resolution.
    pub local_timestamp: i64,
    /// Data measured by the indoor unit. This data is always available.
    pub indoor: Ws6in1IndoorData,
    /// Data measured by the outdoor unit. This data may be unavailable.
    pub outdoor: Option<Ws6in1OutdoorData>,
    /// Data measured by additional sensors.
    pub ext: [Option<Ws6in1ExtData>; Self::EXT_SENSOR_COUNT],
}

impl Ws6in1Data {
    /// Maximum amount of additional sensors.
    pub const EXT_SENSOR_COUNT: usize = 7;
    const DATE_FORMAT: &'static [BorrowedFormatItem<'_>] =
        format_description!("[year]-[month]-[day]");
    const TIME_FORMAT: &'static [BorrowedFormatItem<'_>] =
        format_description!("[hour]:[minute]");
}

struct TokenIterator<'a> {
    iter: SplitWhitespace<'a>,
    tpos: usize,
}

impl<'a> TokenIterator<'a> {
    fn new(iter: SplitWhitespace<'a>) -> Self {
        Self { iter, tpos: 0 }
    }

    fn pos(&self) -> usize {
        self.tpos
    }

    fn next(&mut self) -> Result<&str> {
        let str = self
            .iter
            .next()
            .ok_or(Error::UnexpectedEnd { tpos: self.tpos })?;
        self.tpos += 1;

        Ok(str)
    }

    fn end(&mut self) -> Result<()> {
        match self.iter.next() {
            None => Ok(()),
            Some(x) => Err(Error::GarbageEnd {
                char: x.as_bytes()[0],
            }),
        }
    }
}

impl TryFrom<&str> for Ws6in1Data {
    type Error = super::Error;

    fn try_from(msg: &str) -> Result<Self> {
        let mut iter = TokenIterator::new(msg.split_whitespace());
        let _history_pct = iter.next()?;

        let date = Date::parse(iter.next()?, Self::DATE_FORMAT)
            .map_err(|_| Error::InvalidToken { tpos: iter.pos() })?;
        let time = Time::parse(iter.next()?, Self::TIME_FORMAT)
            .map_err(|_| Error::InvalidToken { tpos: iter.pos() })?;
        let local_timestamp =
            PrimitiveDateTime::new(date, time).as_utc().unix_timestamp();

        let temperature_in = iter
            .next()?
            .parse::<f32>()
            .map_err(|_| Error::InvalidToken { tpos: iter.pos() })?;
        let humidity_in = iter
            .next()?
            .parse::<u8>()
            .map_err(|_| Error::InvalidToken { tpos: iter.pos() })?;

        let temperature_out = iter.next()?.parse::<f32>().ok();
        let humidity_out = iter.next()?.parse::<u8>().ok();

        let rain_day = iter.next()?.parse::<f32>().ok();
        let rain_actual = iter.next()?.parse::<f32>().ok();

        let wind_actual = iter.next()?.parse::<f32>().ok();
        let wind_gust = iter.next()?.parse::<f32>().ok();
        let wind_dir = iter.next()?.parse::<u16>().ok();
        let _wind_octant = iter.next()?;

        let baro_sea = iter
            .next()?
            .parse::<u16>()
            .map_err(|_| Error::InvalidToken { tpos: iter.pos() })?;
        let baro_absolute = iter
            .next()?
            .parse::<u16>()
            .map_err(|_| Error::InvalidToken { tpos: iter.pos() })?;

        let uv_index = iter.next()?.parse::<f32>().ok();
        let dew_point = iter.next()?.parse::<f32>().ok();
        let _unknown = iter.next()?;

        let indoor = Ws6in1IndoorData {
            temperature: temperature_in,
            humidity: humidity_in,
            baro_absolute,
            baro_sea,
        };

        let outdoor = if let (
            Some(temperature),
            Some(humidity),
            Some(rain_day),
            Some(rain_actual),
            Some(wind_actual),
            Some(wind_gust),
            Some(wind_dir),
            Some(uv_index),
            Some(dew_point),
        ) = (
            temperature_out,
            humidity_out,
            rain_day,
            rain_actual,
            wind_actual,
            wind_gust,
            wind_dir,
            uv_index,
            dew_point,
        ) {
            Some(Ws6in1OutdoorData {
                temperature,
                humidity,
                rain_day,
                rain_actual,
                wind_actual,
                wind_gust,
                wind_dir,
                uv_index,
                dew_point,
            })
        } else {
            None
        };

        let mut ext = [None; Self::EXT_SENSOR_COUNT];
        for i in ext.iter_mut() {
            let temperature = iter.next()?.parse::<f32>().ok();
            let humidity = iter.next()?.parse::<u8>().ok();

            if let (Some(temperature), Some(humidity)) = (temperature, humidity)
            {
                *i = Some(Ws6in1ExtData {
                    temperature,
                    humidity,
                })
            }
        }

        iter.end()?;

        Ok(Self {
            local_timestamp,
            indoor,
            outdoor,
            ext,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_data1() {
        let str = "3 2020-01-17 17:30 20.4 49 6.0 60 0.0 0.0 0.0 0.0 129 \
            SE 1017 954 0 -1.2 --.- 27.3 57 33.4 40 --.- -- --.- -- --.- \
            -- --.- -- --.- --";

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

        match TryInto::<Ws6in1Data>::try_into(str) {
            Ok(parsed) => assert_eq!(expected, parsed),
            Err(e) => panic!("Parsing Ws6in1Data failed: {e}"),
        }
    }

    #[test]
    fn test_parse_data2() {
        let str = "100 2025-01-20 00:19 19.5 38 --.- -- 0.0 0.0 --.- --.- \
            --- --- 1014 954 -- --.- --.- 18.6 52 2.3 82 20.9 35 19.1 38 \
            22.3 41 --.- -- --.- --";

        let expected = Ws6in1Data {
            local_timestamp: 1737332340,
            indoor: Ws6in1IndoorData {
                temperature: 19.5,
                humidity: 38,
                baro_sea: 1014,
                baro_absolute: 954,
            },
            outdoor: None,
            ext: [
                Some(Ws6in1ExtData {
                    temperature: 18.6,
                    humidity: 52,
                }),
                Some(Ws6in1ExtData {
                    temperature: 2.3,
                    humidity: 82,
                }),
                Some(Ws6in1ExtData {
                    temperature: 20.9,
                    humidity: 35,
                }),
                Some(Ws6in1ExtData {
                    temperature: 19.1,
                    humidity: 38,
                }),
                Some(Ws6in1ExtData {
                    temperature: 22.3,
                    humidity: 41,
                }),
                None,
                None,
            ],
        };

        match TryInto::<Ws6in1Data>::try_into(str) {
            Ok(parsed) => assert_eq!(expected, parsed),
            Err(e) => panic!("Parsing Ws6in1Data failed: {e}"),
        }
    }
}
