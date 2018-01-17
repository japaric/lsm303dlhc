# `lsm303dlhc`

> A platform agnostic driver to interface with the LSM303DLHC (accelerometer + compass)

## What works

- Reading the accelerometer
- Reading the compass and temperature sensor

## TODO

- [ ] Make sure this works with the `i2cdev` crate (i.e. with the Raspberry Pi)
- [ ] Configuration. e.g. selecting the accelerometer / compass sensing range / sensitivity.
- [ ] How to make the API compatible with device specific features like DMA and interrupts?
- ???

## Examples

You should find at least one example in the [f3] crate.

[f3]: https://docs.rs/f3/~0.5

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
