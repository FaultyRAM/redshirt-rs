# redshirt

[![Travis CI](https://api.travis-ci.com/FaultyRAM/redshirt-rs.svg)](https://travis-ci.com/FaultyRAM/redshirt-rs)
[![Crates.io](https://img.shields.io/crates/v/redshirt.svg)](https://crates.io/crates/redshirt)
[![Docs.rs](https://docs.rs/redshirt/badge.svg)](https://docs.rs/redshirt)

This crate provides utilities for reading and writing Redshirt 1- or Redshirt 2-encoded data. The
Redshirt encoding schemes are used in *Uplink*, a 2001 computer hacking simulation game developed
by Introversion Software.

## Usage

Add redshirt to your `Cargo.toml`

```toml
[dependencies]
redshirt = "^0.1.0"
```

If you're using Rust 2015, you'll also need to add redshirt to your crate root:

```rust
extern crate redshirt;
```

For more details, please refer to [the API documentation](https://docs.rs/redshirt).

### Features

redshirt specifies the following Cargo features, both of which are enabled by default:

* `redshirt1`: toggles Redshirt 1 support.
* `redshirt2`: toggles Redshirt 2 support.

If you only need one or the other, you can specify this in your `Cargo.toml`. For example:

```toml
[dependencies]
redshirt = { version = "^0.1.0", default-features = false, features = ["redshirt1"] }
```

## License

Licensed under either of

* Apache License, Version 2.0,
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
