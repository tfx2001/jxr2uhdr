# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add local Hable tonemapping for SDR base image generation.
- Add WebAssembly SIMD build flags for Emscripten targets.

### Changed

- Preserve decoded 128bpp RGBA f32 pixels until encoding, converting to f16 only for Ultra HDR gain map generation.
- Replace infallible f32-to-f16 byte casting with checked `bytemuck::try_cast_slice` error handling.

## [0.2.1] - 2026-04-30

### Fixed

- Improve compatibility of generated Ultra HDR images across apps and platforms.

## [0.2.0] - 2026-04-19

### Added

- Support 64bpp RGBA half float pixel format (#10)
- Web frontend for JXR to Ultra HDR conversion (#7)

## [0.1.1] - 2026-04-19

### Fixed

- Fix encoding quality setting not being applied correctly (#6)

## [0.1.0] - 2026-04-19

- Initial release.

[unreleased]: https://github.com/tfx2001/jxr2uhdr/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/tfx2001/jxr2uhdr/releases/tag/v0.2.1
[0.2.0]: https://github.com/tfx2001/jxr2uhdr/releases/tag/v0.2.0
[0.1.1]: https://github.com/tfx2001/jxr2uhdr/releases/tag/v0.1.1
[0.1.0]: https://github.com/tfx2001/jxr2uhdr/releases/tag/v0.1.0
