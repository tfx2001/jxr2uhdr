use anyhow::{Context, Result, anyhow};
use std::fs::File;
use std::io::Write;
use ultrahdr::{ColorGamut, ColorRange, ColorTransfer, Encoder, ImgLabel, RawImage};

use crate::tonemap::tonemap_hable_rgba_to_srgb_u8;
use crate::{Image, PixelFormat};

const SCRGB_REFERENCE_WHITE_NITS: f32 = 80.0;
const UHDR_LINEAR_REFERENCE_WHITE_NITS: f32 = 203.0;
const SCRGB_TO_UHDR_LINEAR_SCALE: f32 =
    SCRGB_REFERENCE_WHITE_NITS / UHDR_LINEAR_REFERENCE_WHITE_NITS;
const UHDR_MAX_TARGET_DISPLAY_PEAK_NITS: f32 = 10_000.0;
const TARGET_DISPLAY_PEAK_PERCENTILE: f32 = 0.999;

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
/// 64bpp RGBA f16 only for libultrahdr gain-map generation. HDR intent RGB
/// values are scaled from scRGB's 80-nit reference white to libultrahdr's
/// 203-nit linear reference white before they are passed to libultrahdr.
/// Target display peak brightness is estimated from the HDR content peak and
/// written to the gain-map metadata so decoders do not assume a 10,000-nit
/// linear HDR intent for every image.
/// 64bpp RGBA f16 input is accepted for compatibility with callers that already
/// own half-float pixels.
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

    let mut hdr_image = image.clone();
    hdr_image
        .scale_rgb(SCRGB_TO_UHDR_LINEAR_SCALE)
        .context("Failed to scale HDR intent pixels")?;
    let mut hdr_pixels = hdr_image
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
    encoder.set_target_display_peak_brightness(target_display_peak_nits(image)?)?;

    encoder.encode().context("Failed to encode Ultra HDR")?;

    Ok(encoder
        .encoded_stream()
        .context("Encoder did not produce an output stream")?
        .bytes()?
        .to_vec())
}

fn target_display_peak_nits(image: &Image) -> Result<f32> {
    let mut peaks = image
        .positive_max_rgb_values()
        .context("Failed to read HDR image peak RGB values")?;
    if peaks.is_empty() {
        return Ok(UHDR_LINEAR_REFERENCE_WHITE_NITS);
    }

    let index = ((peaks.len() as f32 * TARGET_DISPLAY_PEAK_PERCENTILE).ceil() as usize)
        .saturating_sub(1)
        .min(peaks.len() - 1);
    peaks.select_nth_unstable_by(index, |left, right| {
        left.partial_cmp(right)
            .expect("finite HDR peak values should be comparable")
    });

    let peak_nits = peaks[index] * SCRGB_REFERENCE_WHITE_NITS;
    Ok(peak_nits.clamp(
        UHDR_LINEAR_REFERENCE_WHITE_NITS,
        UHDR_MAX_TARGET_DISPLAY_PEAK_NITS,
    ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_display_peak_clamps_sdr_range_content_to_ultrahdr_minimum() {
        let image = Image::f32_image(1, 1, &[1.0, 0.5, 0.25, 1.0]);

        let peak_nits = target_display_peak_nits(&image).expect("valid image should have a peak");

        assert_eq!(peak_nits, UHDR_LINEAR_REFERENCE_WHITE_NITS);
    }

    #[test]
    fn target_display_peak_uses_high_percentile_to_ignore_single_outlier() {
        let mut values = Vec::new();
        for _ in 0..999 {
            values.extend_from_slice(&[10.0, 2.0, 1.0, 1.0]);
        }
        values.extend_from_slice(&[1000.0, 2.0, 1.0, 1.0]);
        let image = Image::f32_image((values.len() / 4) as u32, 1, &values);

        let peak_nits = target_display_peak_nits(&image).expect("valid image should have a peak");

        assert_eq!(peak_nits, 800.0);
    }

    #[test]
    fn target_display_peak_clamps_to_ultrahdr_maximum() {
        let image = Image::f16_image(1, 1, &[200.0, 20.0, 10.0, 1.0]);

        let peak_nits = target_display_peak_nits(&image).expect("valid image should have a peak");

        assert_eq!(peak_nits, UHDR_MAX_TARGET_DISPLAY_PEAK_NITS);
    }
}
