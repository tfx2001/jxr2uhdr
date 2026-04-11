use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use ultrahdr::{ColorGamut, ColorRange, ColorTransfer, Encoder, ImgLabel, RawImage};

use crate::Image;

/// Encode [`Image`] to Ultra HDR format and save to file
///
/// **Note:** The input image must be in 64bppRGBAHalfFloat format
pub fn encode_ultra_hdr(image: &mut Image, quality: i32, output_path: &str) -> Result<()> {
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

    let mut out_file = File::create(output_path)?;
    out_file.write_all(encoder.encoded_stream().unwrap().bytes()?)?;

    Ok(())
}
