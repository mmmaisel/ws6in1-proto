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

//! Low-level Ws6in1 protocol implementation.

mod data;
mod frame;

pub use data::{
    Ws6in1DataFrame, Ws6in1DataFrameBase, Ws6in1DataHeader, Ws6in1Payload,
    Ws6in1PayloadBase,
};
#[cfg(feature = "heapless")]
pub use data::{Ws6in1DataFrameHeapless, Ws6in1PayloadHeapless};
#[cfg(feature = "std")]
pub use data::{Ws6in1DataFrameStd, Ws6in1PayloadStd};
use frame::Ws6in1Footer;
pub use frame::Ws6in1Serde;
