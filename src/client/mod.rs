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

//! High level async-hid based SMA speedwire client.

use async_hid::{AsyncHidRead, AsyncHidWrite, Device, HidBackend};
use byteorder_cursor::Cursor;
use futures_lite::stream::StreamExt;
use time::{UtcDateTime, UtcOffset};

use crate::{
    parser::{Ws6in1Assembler, Ws6in1Data},
    protocol::{AnyWs6in1Message, Ws6in1Serde, Ws6in1SetDate, Ws6in1SetTime},
};

mod error;
pub use error::ClientError;

/// Ws6in1 client instance for communication with devices.
/// This object holds the connection independent communication state.
pub struct Ws6in1Client {
    /// Enumerated device to be opened.
    device: Device,
    /// Packet assembler
    asm: Ws6in1Assembler,
}

impl Ws6in1Client {
    const BUFFER_SIZE: usize = 128;
    const USAGE_PAGE: u16 = 0xFF00;
    const USAGE_ID: u16 = 0x0001;

    pub const VENDOR_ID: u16 = 0x1941;
    pub const PRODUCT_ID: u16 = 0x8021;

    /// Creates a new Ws6in1Client.
    pub async fn new() -> Result<Self, ClientError> {
        let device = HidBackend::default()
            .enumerate()
            .await?
            .find(|info| {
                info.matches(
                    Self::USAGE_PAGE,
                    Self::USAGE_ID,
                    Self::VENDOR_ID,
                    Self::PRODUCT_ID,
                )
            })
            .await
            .ok_or(ClientError::DeviceNotFound)?;

        Ok(Self {
            device,
            asm: Ws6in1Assembler::default(),
        })
    }

    /// Opens a connection to the device and reads messages.
    /// The received weather data fragments are assembled and parsed.
    /// This function has no internal timeout and can take up to 20 seconds.
    pub async fn read_weather_data(
        &mut self,
    ) -> Result<Ws6in1Data, ClientError> {
        let mut hid = self.device.open_readable().await?;
        let mut buffer = [0u8; Self::BUFFER_SIZE];

        loop {
            let len = hid.read_input_report(&mut buffer).await?;
            let mut cursor = Cursor::new(&buffer[..len]);
            let data = match AnyWs6in1Message::deserialize(&mut cursor)
                .map_err(|e| e.to_string())
            {
                Ok(AnyWs6in1Message::DataFrame(frame)) => {
                    match self.asm.parse(frame) {
                        Ok(x) => x,
                        Err(e) => {
                            eprintln!("Parser error: {e:?}");
                            None
                        }
                    }
                }
                Ok(_) => {
                    eprintln!("Received invalid message type");
                    None
                }
                Err(e) => {
                    eprintln!("Other error: {e:?}");
                    None
                }
            };

            if let Some(data) = data {
                return Ok(data);
            }
        }
    }

    /// Opens a connection to the device and writes the given
    /// [AnyWs6in1Message] to it.
    pub async fn write(
        &mut self,
        msg: AnyWs6in1Message,
    ) -> Result<(), ClientError> {
        let mut hid = self.device.open_writeable().await?;

        let mut buffer = [0u8; Self::BUFFER_SIZE];
        let mut cursor = Cursor::new(&mut buffer[..]);
        cursor.write_u8(0x00); // report ID
        msg.serialize(&mut cursor)?;

        let len = cursor.position();
        hid.write_output_report(&buffer[..len]).await?;

        Ok(())
    }

    /// Opens a connection to the device and sets the devices date and time
    /// to the given unix timestamp. The timestamp is in UTC but is converted
    /// to local time before being sent.
    pub async fn write_datetime(
        &mut self,
        timestamp: i64,
    ) -> Result<(), ClientError> {
        let datetime = UtcDateTime::from_unix_timestamp(timestamp)?;
        let offset = UtcOffset::local_offset_at(datetime.into())?;
        let now = datetime.to_offset(offset);

        self.write(AnyWs6in1Message::SetDate(Ws6in1SetDate {
            day: now.day(),
            month: now.month() as u8,
            year: (now.year() % 100) as u8,
        }))
        .await?;
        self.write(AnyWs6in1Message::SetTime(Ws6in1SetTime {
            hour: now.hour(),
            min: now.minute(),
            sec: now.second(),
        }))
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::time::{sleep, timeout, Duration};

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_read_weather_data() {
        let mut client = Ws6in1Client::new().await.unwrap();

        match timeout(Duration::from_secs(15), client.read_weather_data()).await
        {
            Ok(Ok(data)) => eprintln!("Received: {data:?}"),
            Ok(Err(e)) => panic!("Reading data failed: {e:?}"),
            Err(_e) => panic!("Timed out"),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_write_datetime() {
        let mut client = Ws6in1Client::new().await.unwrap();

        // 1. Jan 2003 01:02:03 in UTC.
        let mut now = 1044061323;
        client.write_datetime(now).await.unwrap();

        sleep(Duration::from_secs(10)).await;

        now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        client.write_datetime(now).await.unwrap();
    }
}
