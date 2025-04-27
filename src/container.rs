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
use core::ops::Deref;

use crate::Error;

/// Interface to a variable length storage container.
pub trait Ws6in1Container<T: Clone>: Deref<Target = [T]> + Sized {
    /// Constructs a container from a slice.
    fn from_slice(value: &[T]) -> Self;
    /// Appends the content on a container to another.
    fn append(&mut self, value: &impl Ws6in1Container<T>) -> Result<(), Error>;
    /// Clears the content of a container.
    fn clear(&mut self);
}

#[cfg(feature = "std")]
impl<T: Clone> Ws6in1Container<T> for Vec<T> {
    fn from_slice(value: &[T]) -> Self {
        value.to_vec()
    }

    fn append(&mut self, other: &impl Ws6in1Container<T>) -> Result<(), Error> {
        self.extend_from_slice(other);
        Ok(())
    }

    fn clear(&mut self) {
        self.clear();
    }
}

#[cfg(feature = "heapless")]
impl<T: Clone, const N: usize> Ws6in1Container<T> for heapless::Vec<T, N> {
    fn from_slice(value: &[T]) -> Self {
        // Unwrap is fine because the provided buffers are already length
        // checked.
        heapless::Vec::from_slice(value).unwrap()
    }

    fn append(&mut self, other: &impl Ws6in1Container<T>) -> Result<(), Error> {
        self.extend_from_slice(other)
            .map_err(|()| Error::MessageTooLarge {
                len: self.len() + other.len(),
            })
    }

    fn clear(&mut self) {
        self.clear();
    }
}
