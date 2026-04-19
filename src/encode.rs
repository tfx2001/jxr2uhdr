use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use ultrahdr::{ColorGamut, ColorRange, ColorTransfer, Encoder, ImgLabel, RawImage};

use crate::types::Image;

/// Encode [`Image`] to Ultra HDR format and save to file
///
/// **Note:** The input image must be in 64bppRGBAHalfFloat format
pub fn encode_ultra_hdr(image: &mut Image, quality: i32, output_path: &str) -> Result<()> {
    let encoded = encode_ultra_hdr_to_vec(image, quality)?;
    let mut out_file = File::create(output_path)?;
    out_file.write_all(&encoded)?;

    Ok(())
}

/// Encode [`Image`] to Ultra HDR bytes.
///
/// **Note:** The input image must be in 64bppRGBAHalfFloat format
pub fn encode_ultra_hdr_to_vec(image: &mut Image, quality: i32) -> Result<Vec<u8>> {
    let mut hdr_image = RawImage::packed(
        image.format.into(),
        image.width,
        image.height,
        &mut image.pixels,
        ColorGamut::UHDR_CG_BT_709,
        ColorTransfer::UHDR_CT_LINEAR,
        ColorRange::UHDR_CR_FULL_RANGE,
    )?;

    let mut encoder = Encoder::new()?;

    encoder.set_raw_image(&mut hdr_image, ImgLabel::UHDR_HDR_IMG)?;

    encoder.set_quality(quality, ImgLabel::UHDR_HDR_IMG)?;

    encoder.encode().context("Failed to encode Ultra HDR")?;

    Ok(encoder
        .encoded_stream()
        .context("Encoder did not produce an output stream")?
        .bytes()?
        .to_vec())
}
