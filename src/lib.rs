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
#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
#![forbid(unsafe_code)]

#[cfg(feature = "client")]
pub mod client;
mod container;
mod error;
pub mod parser;
pub mod protocol;

pub use container::Ws6in1Container;
pub use error::{Error, Result};
