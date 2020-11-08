# bsx [![cargo-badge][]][cargo] [![license-badge][]][license] [![rust-version-badge][]][rust-version]

Another Rust arbitrary base codec implementation, using min-const-generics.

## Rust Version Policy

This crate is currently nightly only.

## Developing

This project uses [clippy][] and denies warnings in CI builds. To ensure your
changes will be accepted please check them with `cargo clippy` (available via
`rustup component add clippy`) before submitting a pull request (along with
`cargo test` as usual).

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.

[cargo-badge]: https://img.shields.io/crates/v/bsx.svg?style=flat-square
[cargo]: https://crates.io/crates/bsx
[license-badge]: https://img.shields.io/badge/license-MIT/Apache--2.0-lightgray.svg?style=flat-square
[license]: #license
[rust-version-badge]: https://img.shields.io/badge/rust-nightly-red.svg?style=flat-square
[rust-version]: #rust-version-policy

[clippy]: https://github.com/rust-lang-nursery/rust-clippy
