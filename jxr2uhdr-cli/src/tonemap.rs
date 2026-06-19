use anyhow::{Result, bail, ensure};
use half::f16;
use linear_srgb;

use crate::types::{Image, PixelFormat};

/// Tonemap a linear RGBA [`Image`] into sRGB RGBA bytes with Hable filmic tonemapping.
///
/// The image must contain interleaved RGBA pixels in either
/// [`PixelFormat::PixelFormat128bppRGBAFloat`] or
/// [`PixelFormat::PixelFormat64bppRGBAHalfFloat`] format. 128bpp f32 input is
/// used directly as the tonemapping working buffer; 64bpp f16 input is expanded
/// to f32 for compatibility with callers that already hold half-float pixels.
///
/// RGB values are tonemapped with a max-RGB Hable filmic curve, then encoded to
/// sRGB `u8`. Alpha is not tonemapped or gamma-encoded; it is linearly clamped
/// to `[0.0, 1.0]` and scaled to `0..=255`.
pub fn tonemap_hable_rgba_to_srgb_u8(image: &Image) -> Result<Vec<u8>> {
    let mut f32_pixels = image_rgba_to_f32(image)?;
    hable_max_rgb_tonemap_row(f32_pixels.as_mut_slice(), 4);

    let mut sdr_u8_pixels = vec![0u8; f32_pixels.len()];
    linear_srgb::default::linear_to_srgb_u8_rgba_slice(&f32_pixels, sdr_u8_pixels.as_mut_slice());

    Ok(sdr_u8_pixels)
}

fn image_rgba_to_f32(image: &Image) -> Result<Vec<f32>> {
    match image.format {
        PixelFormat::PixelFormat128bppRGBAFloat => {
            ensure_pixel_len(image, 16)?;

            Ok(image
                .pixels
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect())
        }
        PixelFormat::PixelFormat64bppRGBAHalfFloat => {
            ensure_pixel_len(image, 8)?;

            Ok(image
                .pixels
                .chunks_exact(2)
                .map(|chunk| f16::from_le_bytes([chunk[0], chunk[1]]).to_f32())
                .collect())
        }
        PixelFormat::Unknown => bail!("Unsupported pixel format for Hable tonemapping: Unknown"),
    }
}

fn ensure_pixel_len(image: &Image, bytes_per_pixel: usize) -> Result<()> {
    let expected_len = image
        .width
        .checked_mul(image.height)
        .and_then(|pixels| pixels.checked_mul(bytes_per_pixel as u32))
        .map(|len| len as usize)
        .ok_or_else(|| anyhow::anyhow!("Image dimensions overflow pixel buffer size"))?;

    ensure!(
        image.pixels.len() == expected_len,
        "Expected {expected_len} bytes for a {}x{} {:?} image, got {} bytes",
        image.width,
        image.height,
        image.format,
        image.pixels.len()
    );

    Ok(())
}

/// John Hable's filmic tonemapping curve, normalized so scene white maps to 1.0.
fn hable_filmic(value: f32) -> f32 {
    fn partial(x: f32) -> f32 {
        const A: f32 = 0.15;
        const B: f32 = 0.50;
        const C: f32 = 0.10;
        const D: f32 = 0.20;
        const E: f32 = 0.02;
        const F: f32 = 0.30;

        ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F
    }

    const EXPOSURE_BIAS: f32 = 2.0;
    const WHITE_POINT: f32 = 11.2;
    let white_scale = 1.0 / partial(WHITE_POINT);

    (partial(value * EXPOSURE_BIAS) * white_scale).min(1.0)
}

/// Apply Hable filmic tonemapping to interleaved RGB(A) f32 pixels.
///
/// The RGB channels in each pixel are scaled by the same factor derived from the
/// pixel's maximum RGB value. This preserves the input channel ratios while
/// compressing HDR luminance into the SDR range. Alpha, when present, is left
/// untouched.
fn hable_max_rgb_tonemap_row(row: &mut [f32], channels: usize) {
    debug_assert!(channels == 3 || channels == 4);

    for pixel in row.chunks_exact_mut(channels) {
        let max_hdr = pixel[0].max(pixel[1]).max(pixel[2]);
        if max_hdr <= 0.0 {
            pixel[0] = 0.0;
            pixel[1] = 0.0;
            pixel[2] = 0.0;
            continue;
        }

        let scale = hable_filmic(max_hdr) / max_hdr;
        pixel[0] = (pixel[0] * scale).max(0.0);
        pixel[1] = (pixel[1] * scale).max(0.0);
        pixel[2] = (pixel[2] * scale).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba_f32_image(width: u32, height: u32, values: &[f32]) -> Image {
        Image {
            pixels: values
                .iter()
                .flat_map(|&value| value.to_le_bytes())
                .collect(),
            width,
            height,
            format: PixelFormat::PixelFormat128bppRGBAFloat,
        }
    }

    fn rgba_f16_image(width: u32, height: u32, values: &[f32]) -> Image {
        Image {
            pixels: values
                .iter()
                .flat_map(|&value| f16::from_f32(value).to_le_bytes())
                .collect(),
            width,
            height,
            format: PixelFormat::PixelFormat64bppRGBAHalfFloat,
        }
    }

    fn rgba_unknown_image(width: u32, height: u32, values: &[f32]) -> Image {
        Image {
            pixels: values
                .iter()
                .flat_map(|&value| value.to_le_bytes())
                .collect(),
            width,
            height,
            format: PixelFormat::Unknown,
        }
    }

    fn assert_approx_eq(actual: f32, expected: f32) {
        let error = (actual - expected).abs();
        assert!(
            error <= 1e-6,
            "expected {actual} to be within 1e-6 of {expected}, error={error}"
        );
    }

    #[test]
    fn hable_row_preserves_alpha_and_scales_rgb_by_max_channel() {
        let mut row = [1.0, 2.0, 4.0, 0.25];
        let scale = hable_filmic(4.0) / 4.0;

        hable_max_rgb_tonemap_row(&mut row, 4);

        assert_approx_eq(row[0], 1.0 * scale);
        assert_approx_eq(row[1], 2.0 * scale);
        assert_approx_eq(row[2], hable_filmic(4.0));
        assert_eq!(row[3], 0.25);
    }

    #[test]
    fn hable_row_clamps_non_positive_rgb_to_black() {
        let mut row = [-1.0, -0.5, 0.0, 0.75];

        hable_max_rgb_tonemap_row(&mut row, 4);

        assert_eq!(row, [0.0, 0.0, 0.0, 0.75]);
    }

    #[test]
    fn tonemap_empty_input_returns_empty_output() {
        let input = rgba_f32_image(0, 0, &[]);

        let output =
            tonemap_hable_rgba_to_srgb_u8(&input).expect("empty image should tonemap successfully");

        assert!(output.is_empty());
    }

    #[test]
    fn tonemap_black_pixel_keeps_black_and_converts_alpha_linearly() {
        let input = rgba_f32_image(1, 1, &[0.0, 0.0, 0.0, 0.5]);

        let output = tonemap_hable_rgba_to_srgb_u8(&input)
            .expect("valid f32 image should tonemap successfully");

        assert_eq!(output, [0, 0, 0, 128]);
    }

    #[test]
    fn tonemap_primary_red_remains_red_dominant_without_gamut_mapping() {
        let input = rgba_f32_image(1, 1, &[1.0, 0.0, 0.0, 1.0]);

        let output = tonemap_hable_rgba_to_srgb_u8(&input)
            .expect("valid f32 image should tonemap successfully");

        assert_eq!(output.len(), 4);
        assert!(
            output[0] > output[1] && output[1] == output[2],
            "expected red to stay on the red channel without gamut mapping, got {output:?}"
        );
        assert_eq!(output[3], 255);
    }

    #[test]
    fn tonemap_accepts_half_float_images_for_compatibility() {
        let input = rgba_f16_image(1, 1, &[0.0, 0.0, 0.0, 1.0]);

        let output = tonemap_hable_rgba_to_srgb_u8(&input)
            .expect("valid f16 image should tonemap successfully");

        assert_eq!(output, [0, 0, 0, 255]);
    }

    #[test]
    fn tonemap_rejects_unknown_pixel_format() {
        let input = rgba_unknown_image(1, 1, &[0.0, 0.0, 0.0, 1.0]);

        let error = tonemap_hable_rgba_to_srgb_u8(&input)
            .expect_err("unknown image format should be rejected");

        assert!(
            error
                .to_string()
                .contains("Unsupported pixel format for Hable tonemapping"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn tonemap_rejects_mismatched_pixel_buffer_len() {
        let mut input = rgba_f32_image(1, 1, &[0.0, 0.0, 0.0, 1.0]);
        input.pixels.push(0xff);

        let error = tonemap_hable_rgba_to_srgb_u8(&input)
            .expect_err("mismatched pixel buffer length should be rejected");

        assert!(
            error.to_string().contains("Expected 16 bytes"),
            "unexpected error: {error:#}"
        );
    }
}
