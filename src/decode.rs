use std::fs::File;

use anyhow::{Context, Result};
use jpegxr::ImageDecode;
use log::debug;

use crate::convert::convert_128bpp_f32_to_64bpp_f16;
use crate::types::{Image, PixelFormat};

/// Decode JXR image to [`Image`] and convert to 64bppRGBAHalfFloat format
pub fn decode_jxr(path: &str) -> Result<Image> {
    let input_file = File::open(path).context("Failed to open JXR file")?;
    let mut decoder =
        ImageDecode::with_reader(input_file).context("Failed to initialize JXR decoder")?;

    let (width, height) = decoder
        .get_size()
        .context("Failed to get image dimensions")?;
    debug!("Image dimensions: {} x {}", width, height);

    let pixel_format = decoder
        .get_pixel_format()
        .context("Failed to get pixel format")?;
    debug!("Pixel format: {:?}", pixel_format);

    let (stride, input_format) = match pixel_format {
        jpegxr::PixelFormat::PixelFormat128bppRGBAFloat => {
            (width as usize * 16, PixelFormat::PixelFormat128bppRGBAFloat)
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported pixel format: {:?}",
                pixel_format
            ));
        }
    };

    let size = stride * height as usize;

    let mut buffer = vec![0u8; size];
    decoder
        .copy_all(&mut buffer, stride)
        .context("Failed to decode JXR pixels")?;

    let (pixels, output_format) = if input_format == PixelFormat::PixelFormat128bppRGBAFloat {
        (
            convert_128bpp_f32_to_64bpp_f16(&buffer),
            PixelFormat::PixelFormat64bppRGBAHalfFloat,
        )
    } else {
        (buffer, input_format)
    };

    Ok(Image {
        pixels,
        width: width as u32,
        height: height as u32,
        format: output_format,
    })
}
