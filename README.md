# jxr2uhdr

> **⚠️ Work in Progress** — This is a personal side project, not actively maintained. Use at your own risk.

English | [中文](README.zh-CN.md)

Convert JPEG XR (`.jxr`) HDR images to [Ultra HDR](https://developer.android.com/media/platform/hdr-image-format) JPEG files.

Ultra HDR is a backward-compatible JPEG format developed by Google that embeds an HDR gain map alongside a standard SDR base image. The resulting file can be displayed as normal SDR on legacy devices, while HDR-capable displays render the full high dynamic range.

## Use Case

NVIDIA's in-game screenshot capture tool saves HDR frames as JPEG XR (128bpp RGBA float). This tool converts those captures directly to Ultra HDR JPEG, which is natively supported on Android 14+ and modern displays.

```
input.jxr  (128bpp RGBA f32, HDR)
    ↓  jxr2uhdr
output.jpg  (Ultra HDR JPEG, backward-compatible SDR + HDR gain map)
```

## Installation

```bash
cargo install --path .
```

Or build without installing:

```bash
cargo build --release
# Binary: target/release/jxr2uhdr
```

## Usage

```
jxr2uhdr --input <INPUT> --output <OUTPUT> [--quality <QUALITY>]

Options:
  -i, --input <INPUT>      Input JXR file path
  -o, --output <OUTPUT>    Output Ultra HDR JPG file path
  -q, --quality <QUALITY>  Quality of the output base JPEG (0-100) [default: 90]
  -h, --help               Print help
  -V, --version            Print version
```

**Example:**

```bash
jxr2uhdr -i screenshot.jxr -o output.jpg
jxr2uhdr -i screenshot.jxr -o output.jpg --quality 95
```

Log verbosity can be controlled via the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug jxr2uhdr -i screenshot.jxr -o output.jpg
```

## Build

```bash
# Debug build
cargo build

# Release build (recommended for production use)
cargo build --release
```

## Test

```bash
cargo test
```

## License

MIT
