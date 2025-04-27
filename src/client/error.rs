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

use async_hid::HidError;
use time::error::{ComponentRange, IndeterminateOffset};

use crate::Error;

/// Errors returned from Ws6in1 client.
#[derive(Clone, Debug)]
pub enum ClientError {
    /// A Ws6in1 protocol error.
    ProtocolError(Error),
    /// An HID system error.
    HidError(String),
    /// System time error.
    TimeError(String),
    /// No matching device was found.
    DeviceNotFound,
}

impl From<Error> for ClientError {
    fn from(e: Error) -> Self {
        Self::ProtocolError(e)
    }
}

impl From<HidError> for ClientError {
    fn from(e: HidError) -> Self {
        Self::HidError(e.to_string())
    }
}

impl From<IndeterminateOffset> for ClientError {
    fn from(e: IndeterminateOffset) -> Self {
        Self::TimeError(e.to_string())
    }
}

impl From<ComponentRange> for ClientError {
    fn from(e: ComponentRange) -> Self {
        Self::TimeError(e.to_string())
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ProtocolError(e) => write!(f, "{e}"),
            Self::HidError(e) => write!(f, "HID error: {e}"),
            Self::TimeError(e) => write!(f, "System time error: {e}"),
            Self::DeviceNotFound => write!(f, "No matching device was found"),
        }
    }
}
