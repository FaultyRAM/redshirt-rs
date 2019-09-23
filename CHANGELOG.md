# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to [Semantic Versioning].

## [Unreleased]
### Fixed
- Fixed a formatting error in the documentation for `v2::Writer::into_inner`.
- Fixed a panic in `v2::Writer::write` if the length of the `&[u8]` to be written is greater than
  16384 bytes.

### [0.1.2] - 2019-09-23
### Fixed
- Fixed building with default features disabled.

## [0.1.1] - 2019-09-04
### Changed
- Updated Cargo package information.
- (There are no code changes in this release, this is just to ensure the crate is properly
  categorised on crates.io)

## [0.1.0] - 2019-09-04
### Added
- Initial release.

[Keep a Changelog]: https://keepachangelog.com/en/1.0.0/
[Semantic Versioning]: https://semver.org/spec/v2.0.0.html
[Unreleased]: https://github.com/FaultyRAM/redshirt-rs/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/FaultyRAM/redshirt-rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/FaultyRAM/redshirt-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/FaultyRAM/redshirt-rs/releases/tag/v0.1.0
