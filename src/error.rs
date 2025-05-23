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
#[cfg(not(feature = "std"))]
use core::{fmt::Debug, prelude::rust_2021::derive};

use byteorder_cursor::BufferTooSmall;

/// Errors returned from Ws6in1 protocol processing.
#[derive(Clone, Debug)]
pub enum Error {
    /// The provided buffer is too small.
    BufferTooSmall(BufferTooSmall),
    /// The provided buffer contained unexpected trailing bytes and
    /// was not completely deserialized.
    BufferNotConsumed { trailing: usize },
    /// The magic value was incorrect.
    InvalidMagic { magic: u8 },
    /// The given frame type is unsupported.
    UnsupportedType { r#type: u8 },
    /// The opcode of this message has an unsupported value.
    UnsupportedOpcode { opcode: u8 },
    /// The payload of a packet exceeds the maximum supported length.
    PayloadTooLarge { len: usize },
    /// The parsed unexpectedly encountered end of input after this token number.
    UnexpectedEnd { tpos: usize },
    /// The message contained a non UTF8 character.
    InvalidCharacter { idx: usize },
    /// Parsing number from string token with given number failed.
    InvalidToken { tpos: usize },
    /// Parser encountered "garbage" characters at the end of the message.
    GarbageEnd { char: u8 },
    /// A fragment was discarded during message assembly.
    FragmentDiscarded { idx: u8 },
    /// A message exceeded maximum length during assembly.
    MessageTooLarge { len: usize },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::BufferTooSmall(e) => {
                write!(f, "{e}")
            }
            Self::BufferNotConsumed { trailing } => {
                write!(
                    f,
                    "The supplied buffer contained {trailing} trailing bytes"
                )
            }
            Self::InvalidMagic { magic } => {
                write!(f, "Found invalid magic value {magic:X}")
            }
            Self::UnsupportedType { r#type: typ } => {
                write!(f, "Found unsupported frame type {typ:X}")
            }
            Self::UnsupportedOpcode { opcode } => {
                write!(f, "Found unsupported opcode {opcode:X}")
            }
            Self::PayloadTooLarge { len } => {
                write!(
                    f,
                    "The messages payload length {len} exceeds \
                    the supported maximum"
                )
            }
            Self::UnexpectedEnd { tpos } => {
                write!(f, "Unexpected end found after token number {tpos}")
            }
            Self::InvalidCharacter { idx } => {
                write!(f, "Non UTF8 character was found at index {idx}")
            }
            Self::InvalidToken { tpos } => {
                write!(f, "Parsing token number {tpos} failed",)
            }
            Self::GarbageEnd { char } => {
                write!(
                    f,
                    "Parser found garbage at end of message: 0x{char:02X}",
                )
            }
            Self::FragmentDiscarded { idx } => {
                write!(
                    f,
                    "Fragment {idx} was discarded during message assembly",
                )
            }
            Self::MessageTooLarge { len } => {
                write!(
                    f,
                    "The assembled message length {len} exceeds \
                    the supported maximum",
                )
            }
        }
    }
}

impl From<BufferTooSmall> for Error {
    fn from(e: BufferTooSmall) -> Self {
        Self::BufferTooSmall(e)
    }
}

/// A specialized Result type for Ws6in1 operations.
pub type Result<T> = core::result::Result<T, Error>;
