use anyhow::{Context, Result, anyhow};
use std::fs::File;
use std::io::Write;
use ultrahdr::{ColorGamut, ColorRange, ColorTransfer, Encoder, ImgLabel, RawImage};

use crate::tonemap::tonemap_hable_rgba_to_srgb_u8;
use crate::{Image, PixelFormat};

/// Encode [`Image`] to Ultra HDR format and save to file
pub fn encode_ultra_hdr(image: &Image, quality: i32, output_path: &str) -> Result<()> {
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
pub fn encode_ultra_hdr_to_vec(image: &Image, quality: i32) -> Result<Vec<u8>> {
    let mut sdr_pixels = tonemap_hable_rgba_to_srgb_u8(image)?;
    let mut sdr_image = RawImage::rgba8888(
        image.width(),
        image.height(),
        &mut sdr_pixels,
        ColorGamut::UHDR_CG_BT_709,
        ColorTransfer::UHDR_CT_SRGB,
        ColorRange::UHDR_CR_FULL_RANGE,
    )?;

    let mut hdr_pixels = image
        .clone()
        .into_pixel_format(PixelFormat::PixelFormat64bppRGBAHalfFloat)
        .map(Image::into_bytes)
        .context("Failed to prepare RGBA f16 HDR intent pixels")?;
    let mut hdr_image = RawImage::packed(
        PixelFormat::PixelFormat64bppRGBAHalfFloat.try_into()?,
        image.width(),
        image.height(),
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

impl TryFrom<PixelFormat> for ultrahdr::ImgFormat {
    type Error = anyhow::Error;

    fn try_from(format: PixelFormat) -> Result<Self, Self::Error> {
        match format {
            PixelFormat::PixelFormat64bppRGBAHalfFloat => {
                Ok(ultrahdr::ImgFormat::UHDR_IMG_FMT_64bppRGBAHalfFloat)
            }
            unsupported => Err(anyhow!(
                "libultrahdr does not support this pixel format: {unsupported:?}"
            )),
        }
    }
}
