# ws6in1-proto

A Rust crate that provides a CC8488 compatible weather station protocol
implementation.

## Crate Features and Goals

* [x] High level client for easy integration into applications.
* [x] Optional **`no_std`** support for embedded devices.
* [x] Verify messages during de-serialization.
* [x] Being efficient if possible.

## Rust Feature Flags
* **`std`** (default) — Remove this feature to make the library
  `no_std` compatible.
* **`client`** — Enables an async-hid based high level client.
* **`heapless`** - Enables support for heapless vectors.

## Device access

To access the USB hidraw device on Linux it may be required to install the
supplied udev rules and add the user to the `dialout` group.

## Specification

* ws6in1 protocol based on: [weewx-ws6in1](https://github.com/BobAtchley/weewx-ws6in1/blob/a969571c2e59ff8a739f16a95ff7404f00e822d2/bin/user/ws6in1.py)

## License

**ws6in1-proto** is licensed under the GNU Affero General Public License as published
by the Free Software Foundation, either version 3 of the License, or (at your
option) any later version.
