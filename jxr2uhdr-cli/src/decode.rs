use std::fs::File;
use std::io::{Cursor, Read, Seek};

use anyhow::{Context, Result, anyhow};
use jpegxr::ImageDecode;
use log::debug;

use crate::{Image, PixelFormat};

/// Decode JXR image to [`Image`].
pub fn decode_jxr(path: &str) -> Result<Image> {
    let input_file = File::open(path).context("Failed to open JXR file")?;
    decode_jxr_reader(input_file)
}

/// Decode JXR bytes to [`Image`].
pub fn decode_jxr_bytes(bytes: &[u8]) -> Result<Image> {
    decode_jxr_reader(Cursor::new(bytes))
}

fn decode_jxr_reader<R>(reader: R) -> Result<Image>
where
    R: Read + Seek,
{
    let mut decoder =
        ImageDecode::with_reader(reader).context("Failed to initialize JXR decoder")?;

    let (width, height) = decoder
        .get_size()
        .context("Failed to get image dimensions")?;
    debug!("Image dimensions: {} x {}", width, height);

    let pixel_format = decoder
        .get_pixel_format()
        .context("Failed to get pixel format")?;
    debug!("Pixel format: {:?}", pixel_format);

    let input_format: PixelFormat = pixel_format.try_into()?;
    let bytes_per_pixel = input_format.bytes_per_pixel();
    let stride = (width as usize)
        .checked_mul(bytes_per_pixel)
        .context("Width multiplication overflow")?;
    let size = stride
        .checked_mul(height as usize)
        .context("Image size overflow")?;

    let mut buffer = vec![0u8; size];
    decoder
        .copy_all(&mut buffer, stride)
        .context("Failed to decode JXR pixels")?;

    Image::from_bytes(width as u32, height as u32, input_format, buffer)
}

impl TryFrom<jpegxr::PixelFormat> for PixelFormat {
    type Error = anyhow::Error;

    fn try_from(format: jpegxr::PixelFormat) -> Result<Self, Self::Error> {
        match format {
            jpegxr::PixelFormat::PixelFormat128bppRGBAFloat => {
                Ok(PixelFormat::PixelFormat128bppRGBAFloat)
            }
            jpegxr::PixelFormat::PixelFormat64bppRGBAHalf => {
                Ok(PixelFormat::PixelFormat64bppRGBAHalfFloat)
            }
            unsupported => Err(anyhow!("Unsupported pixel format: {unsupported:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/sunrise-hdr.jxr")
    }

    #[test]
    fn decode_jxr_file_returns_expected_image_metadata() {
        let fixture = fixture_path();

        let image = decode_jxr(
            fixture
                .to_str()
                .expect("fixture path should be valid UTF-8"),
        )
        .expect("sample JXR fixture should decode successfully");

        assert_eq!(image.width(), 3440);
        assert_eq!(image.height(), 1440);
        assert_eq!(image.format(), PixelFormat::PixelFormat128bppRGBAFloat);
        assert_eq!(
            image.as_slice().len(),
            image.width() as usize * image.height() as usize * 16
        );
        assert!(image.as_slice().iter().any(|&byte| byte != 0));
    }

    #[test]
    fn decode_jxr_bytes_matches_file_decode_result() {
        let fixture = fixture_path();
        let bytes = fs::read(&fixture).expect("sample JXR fixture should be readable");

        let decoded_from_bytes =
            decode_jxr_bytes(&bytes).expect("sample JXR bytes should decode successfully");

        assert_eq!(decoded_from_bytes.width(), 3440);
        assert_eq!(decoded_from_bytes.height(), 1440);
        assert_eq!(
            decoded_from_bytes.format(),
            PixelFormat::PixelFormat128bppRGBAFloat
        );
        assert_eq!(
            decoded_from_bytes.as_slice().len(),
            decoded_from_bytes.width() as usize * decoded_from_bytes.height() as usize * 16
        );
        assert!(decoded_from_bytes.as_slice().iter().any(|&byte| byte != 0));
    }

    #[test]
    fn decode_jxr_bytes_rejects_invalid_input() {
        let error = match decode_jxr_bytes(b"not a valid jxr stream") {
            Ok(_) => panic!("invalid bytes should fail to decode"),
            Err(error) => error,
        };

        assert!(
            format!("{error:#}").contains("Failed to initialize JXR decoder"),
            "unexpected error: {error:#}"
        );
    }
}
