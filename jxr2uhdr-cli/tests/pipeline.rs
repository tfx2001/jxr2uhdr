use std::path::PathBuf;

use jxr2uhdr::decode::decode_jxr;
use jxr2uhdr::encode::encode_ultra_hdr_to_vec;
use jxr2uhdr::{Image, PixelFormat};

fn fixture_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/data/{file_name}"))
}

fn decode_fixture(file_name: &str) -> Image {
    let input = fixture_path(file_name);
    decode_jxr(input.to_str().expect("fixture path should be valid UTF-8"))
        .unwrap_or_else(|error| panic!("{file_name} should decode successfully: {error:#}"))
}

fn assert_ultra_hdr_jpeg(encoded: &[u8]) {
    assert!(encoded.len() > 1024, "encoded JPEG should not be tiny");
    assert_eq!(&encoded[..2], &[0xFF, 0xD8], "output should be a JPEG");
    assert!(
        encoded
            .windows(b"urn:iso:std:iso:ts:21496:-1".len())
            .any(|window| window == b"urn:iso:std:iso:ts:21496:-1"),
        "output should contain the Ultra HDR ISO marker"
    );
}

#[test]
fn converts_sample_jxr_to_ultra_hdr_bytes() {
    let image = decode_fixture("sunrise-hdr.jxr");

    assert_eq!(image.format(), PixelFormat::PixelFormat128bppRGBAFloat);

    let encoded = encode_ultra_hdr_to_vec(&image, 90)
        .expect("decoded image should encode to Ultra HDR bytes");

    assert_ultra_hdr_jpeg(&encoded);
}

#[test]
fn converts_64bpp_sample_jxr_to_ultra_hdr_bytes() {
    let image = decode_fixture("blue_colorballs.jxr");

    assert_eq!(image.format(), PixelFormat::PixelFormat64bppRGBAHalfFloat);

    let encoded = encode_ultra_hdr_to_vec(&image, 90)
        .expect("decoded 64bpp image should encode to Ultra HDR bytes");

    assert_ultra_hdr_jpeg(&encoded);
}

#[test]
fn lower_quality_produces_smaller_output() {
    // Decode separately for each encode to avoid shared mutable state
    let image_low = decode_fixture("sunrise-hdr.jxr");
    let image_high = decode_fixture("sunrise-hdr.jxr");

    let encoded_low =
        encode_ultra_hdr_to_vec(&image_low, 10).expect("should encode with low quality");
    let encoded_high =
        encode_ultra_hdr_to_vec(&image_high, 95).expect("should encode with high quality");

    assert!(
        encoded_low.len() < encoded_high.len(),
        "quality=10 ({} bytes) should produce smaller output than quality=95 ({} bytes)",
        encoded_low.len(),
        encoded_high.len(),
    );
}
