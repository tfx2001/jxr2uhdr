use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use ultrahdr::{ColorGamut, ColorRange, ColorTransfer, Encoder, ImgLabel, RawImage};

use crate::convert::convert_128bpp_f32_to_64bpp_f16;
use crate::tonemap::tonemap_hable_rgba_to_srgb_u8;
use crate::types::{Image, PixelFormat};

/// Encode [`Image`] to Ultra HDR format and save to file
pub fn encode_ultra_hdr(image: &mut Image, quality: i32, output_path: &str) -> Result<()> {
    let encoded = encode_ultra_hdr_to_vec(image, quality)?;
    let mut out_file = File::create(output_path)?;
    out_file.write_all(&encoded)?;

    Ok(())
}

/// Encode [`Image`] to Ultra HDR bytes.
///
/// 128bpp RGBA f32 input is tonemapped directly from f32 and converted to
/// 64bpp RGBA f16 only for libultrahdr gain-map generation. 64bpp RGBA f16
/// input is accepted for compatibility with callers that already own half-float
/// pixels.
pub fn encode_ultra_hdr_to_vec(image: &mut Image, quality: i32) -> Result<Vec<u8>> {
    let mut sdr_pixels = tonemap_hable_rgba_to_srgb_u8(image)?;
    let mut sdr_image = RawImage::rgba8888(
        image.width,
        image.height,
        &mut sdr_pixels,
        ColorGamut::UHDR_CG_BT_709,
        ColorTransfer::UHDR_CT_SRGB,
        ColorRange::UHDR_CR_FULL_RANGE,
    )?;

    let mut hdr_pixels = hdr_pixels_as_f16(image)?;
    let mut hdr_image = RawImage::packed(
        PixelFormat::PixelFormat64bppRGBAHalfFloat.into(),
        image.width,
        image.height,
        &mut hdr_pixels,
        ColorGamut::UHDR_CG_BT_709,
        ColorTransfer::UHDR_CT_LINEAR,
        ColorRange::UHDR_CR_FULL_RANGE,
    )?;

    let mut encoder = Encoder::new()?;

    encoder.set_raw_image(&mut hdr_image, ImgLabel::UHDR_HDR_IMG)?;
    encoder.set_raw_image(&mut sdr_image, ImgLabel::UHDR_SDR_IMG)?;

    encoder.set_quality(quality, ImgLabel::UHDR_BASE_IMG)?;
    encoder.set_quality(quality, ImgLabel::UHDR_GAIN_MAP_IMG)?;

    encoder.encode().context("Failed to encode Ultra HDR")?;

    Ok(encoder
        .encoded_stream()
        .context("Encoder did not produce an output stream")?
        .bytes()?
        .to_vec())
}

fn hdr_pixels_as_f16(image: &Image) -> Result<Vec<u8>> {
    match image.format {
        PixelFormat::PixelFormat128bppRGBAFloat => {
            convert_128bpp_f32_to_64bpp_f16(image.pixels.as_slice())
        }
        PixelFormat::PixelFormat64bppRGBAHalfFloat => Ok(image.pixels.clone()),
        PixelFormat::Unknown => Err(anyhow::anyhow!(
            "Unsupported pixel format for Ultra HDR encoding: Unknown"
        )),
    }
}
