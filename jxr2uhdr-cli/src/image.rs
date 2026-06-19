use anyhow::{Result, ensure};
use half::f16;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
/// Pixel format enum
pub enum PixelFormat {
    PixelFormat64bppRGBAHalfFloat,
    PixelFormat128bppRGBAFloat,
}

impl PixelFormat {
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            PixelFormat::PixelFormat128bppRGBAFloat => 16,
            PixelFormat::PixelFormat64bppRGBAHalfFloat => 8,
        }
    }
}

/// HDR image data structure, used to pass data between decoding and encoding
#[derive(Clone)]
pub struct Image {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    format: PixelFormat,
}

mod private {
    pub trait Sealed {}
}

pub trait ImagePixel: private::Sealed + Sized {
    fn from_image(image: &Image) -> Result<Vec<Self>>;
}

impl Image {
    pub fn from_bytes(
        width: u32,
        height: u32,
        format: PixelFormat,
        pixels: impl Into<Vec<u8>>,
    ) -> Result<Self> {
        let pixels = pixels.into();
        let bpp = format.bytes_per_pixel();
        ensure_pixel_len(width, height, format, pixels.len(), bpp)?;

        Ok(Self {
            pixels,
            width,
            height,
            format,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn format(&self) -> PixelFormat {
        self.format
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.pixels
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.pixels
    }

    pub fn to_pixels<T: ImagePixel>(&self) -> Result<Vec<T>> {
        T::from_image(self)
    }

    pub fn into_pixel_format(self, format: PixelFormat) -> Result<Self> {
        if self.format == format {
            return Ok(self);
        }

        let pixels: Vec<u8> = match format {
            PixelFormat::PixelFormat128bppRGBAFloat => {
                let f32_pixels = self.to_pixels::<f32>()?;
                let mut bytes = vec![0u8; f32_pixels.len() * 4];
                for (i, pixel) in f32_pixels.into_iter().enumerate() {
                    bytes[i * 4..i * 4 + 4].copy_from_slice(&pixel.to_le_bytes());
                }
                bytes
            }
            PixelFormat::PixelFormat64bppRGBAHalfFloat => {
                let f16_pixels = self.to_pixels::<f16>()?;
                let mut bytes = vec![0u8; f16_pixels.len() * 2];
                for (i, pixel) in f16_pixels.into_iter().enumerate() {
                    bytes[i * 2..i * 2 + 2].copy_from_slice(&pixel.to_le_bytes());
                }
                bytes
            }
        };

        Self::from_bytes(self.width, self.height, format, pixels)
    }
}

#[cfg(test)]
impl Image {
    pub(crate) fn f32_image(width: u32, height: u32, values: &[f32]) -> Self {
        Self::from_bytes(
            width,
            height,
            PixelFormat::PixelFormat128bppRGBAFloat,
            values
                .iter()
                .flat_map(|&value| value.to_le_bytes())
                .collect::<Vec<_>>(),
        )
        .expect("test f32 image should be valid")
    }

    pub(crate) fn f16_image(width: u32, height: u32, values: &[f32]) -> Self {
        Self::from_bytes(
            width,
            height,
            PixelFormat::PixelFormat64bppRGBAHalfFloat,
            values
                .iter()
                .flat_map(|&value| f16::from_f32(value).to_le_bytes())
                .collect::<Vec<_>>(),
        )
        .expect("test f16 image should be valid")
    }
}

impl private::Sealed for f32 {}

impl ImagePixel for f32 {
    fn from_image(image: &Image) -> Result<Vec<Self>> {
        match image.format {
            PixelFormat::PixelFormat128bppRGBAFloat => Ok(image
                .pixels
                .chunks_exact(4)
                .map(f32_from_le_bytes)
                .collect()),
            PixelFormat::PixelFormat64bppRGBAHalfFloat => Ok(image
                .pixels
                .chunks_exact(2)
                .map(|chunk| f16::from_le_bytes([chunk[0], chunk[1]]).to_f32())
                .collect()),
        }
    }
}

impl private::Sealed for f16 {}

impl ImagePixel for f16 {
    fn from_image(image: &Image) -> Result<Vec<Self>> {
        match image.format {
            PixelFormat::PixelFormat128bppRGBAFloat => Ok(image
                .pixels
                .chunks_exact(4)
                .map(|chunk| f16::from_f32(f32_from_le_bytes(chunk)))
                .collect()),
            PixelFormat::PixelFormat64bppRGBAHalfFloat => Ok(image
                .pixels
                .chunks_exact(2)
                .map(|chunk| f16::from_le_bytes([chunk[0], chunk[1]]))
                .collect()),
        }
    }
}

fn ensure_pixel_len(
    width: u32,
    height: u32,
    format: PixelFormat,
    actual_len: usize,
    bytes_per_pixel: usize,
) -> Result<()> {
    let expected_len = usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(bytes_per_pixel))
        .ok_or_else(|| anyhow::anyhow!("Image dimensions overflow pixel buffer size"))?;

    ensure!(
        actual_len == expected_len,
        "Expected {expected_len} bytes for a {width}x{height} {format:?} image, got {actual_len} bytes"
    );

    Ok(())
}

fn f32_from_le_bytes(bytes: &[u8]) -> f32 {
    f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

#[cfg(test)]
mod tests {
    use half::f16;

    use super::*;

    fn f16_bytes_to_f32(bytes: &[u8]) -> Vec<f32> {
        bytes
            .chunks_exact(2)
            .map(|chunk| f16::from_le_bytes([chunk[0], chunk[1]]).to_f32())
            .collect()
    }

    fn f16_pixels_to_f32(pixels: &[f16]) -> Vec<f32> {
        pixels.iter().map(|value| value.to_f32()).collect()
    }

    fn expect_from_bytes_err(result: Result<Image>, message: &str) -> anyhow::Error {
        match result {
            Ok(_) => panic!("{message}"),
            Err(error) => error,
        }
    }

    #[test]
    fn to_pixels_converts_empty_f32_image_to_empty_f16_channels() {
        let image = Image::f32_image(0, 0, &[]);

        let output = image
            .to_pixels::<f16>()
            .expect("empty f32 image should convert");

        assert!(output.is_empty());
    }

    #[test]
    fn to_pixels_converts_multiple_f32_pixels_to_f16_channels() {
        let image = Image::f32_image(
            2,
            1,
            &[
                1.0, 0.0, 0.0, 1.0, // red
                0.0, 1.0, 0.0, 1.0, // green
            ],
        );

        let output = image
            .to_pixels::<f16>()
            .expect("valid f32 image should convert");

        assert_eq!(
            f16_pixels_to_f32(&output),
            [1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0]
        );
    }

    #[test]
    fn to_pixels_converts_special_f32_values_to_f16_channels() {
        let image = Image::f32_image(1, 1, &[f32::INFINITY, f32::NEG_INFINITY, -0.0, f32::MAX]);

        let f16_values = image
            .to_pixels::<f16>()
            .expect("valid f32 image should convert");

        assert!(f16_values[0].is_infinite() && f16_values[0].is_sign_positive());
        assert!(f16_values[1].is_infinite() && f16_values[1].is_sign_negative());
        assert_eq!(f16_values[2].to_f32(), -0.0);
        assert!(f16_values[3].is_infinite());
    }

    #[test]
    fn into_pixel_format_converts_f32_image_to_f16_image() {
        let image = Image::f32_image(1, 1, &[0.25, 0.5, 0.75, 1.0]);

        let output = image
            .into_pixel_format(PixelFormat::PixelFormat64bppRGBAHalfFloat)
            .expect("valid f32 image should convert to f16");

        assert_eq!(output.width(), 1);
        assert_eq!(output.height(), 1);
        assert_eq!(output.format(), PixelFormat::PixelFormat64bppRGBAHalfFloat);
        assert_eq!(f16_bytes_to_f32(output.as_slice()), [0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn into_pixel_format_converts_f16_image_to_f32_image() {
        let image = Image::f16_image(1, 1, &[0.25, 0.5, 0.75, 1.0]);

        let output = image
            .into_pixel_format(PixelFormat::PixelFormat128bppRGBAFloat)
            .expect("valid f16 image should convert to f32");

        assert_eq!(output.width(), 1);
        assert_eq!(output.height(), 1);
        assert_eq!(output.format(), PixelFormat::PixelFormat128bppRGBAFloat);
        assert_eq!(output.to_pixels::<f32>().unwrap(), [0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn into_pixel_format_preserves_same_format_image() {
        let image = Image::f32_image(2, 1, &[1.0, 2.0, 4.0, 0.5, 8.0, 16.0, 32.0, 1.0]);
        let original_pixels = image.as_slice().to_vec();

        let output = image
            .into_pixel_format(PixelFormat::PixelFormat128bppRGBAFloat)
            .expect("same-format conversion should preserve image");

        assert_eq!(output.width(), 2);
        assert_eq!(output.height(), 1);
        assert_eq!(output.format(), PixelFormat::PixelFormat128bppRGBAFloat);
        assert_eq!(output.as_slice(), original_pixels.as_slice());
    }

    #[test]
    fn from_bytes_rejects_mismatched_pixel_buffer_len() {
        let error = expect_from_bytes_err(
            Image::from_bytes(1, 1, PixelFormat::PixelFormat128bppRGBAFloat, vec![0; 17]),
            "mismatched pixel buffer length should be rejected",
        );

        assert!(
            error.to_string().contains("Expected 16 bytes"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn from_bytes_rejects_dimension_overflow() {
        let error = expect_from_bytes_err(
            Image::from_bytes(
                u32::MAX,
                u32::MAX,
                PixelFormat::PixelFormat128bppRGBAFloat,
                vec![],
            ),
            "overflowing dimensions should be rejected",
        );

        assert!(
            error
                .to_string()
                .contains("Image dimensions overflow pixel buffer size"),
            "unexpected error: {error:#}"
        );
    }
}
