
#[allow(unused)]
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
/// Pixel format enum
pub enum PixelFormat {
    PixelFormat64bppRGBAHalfFloat,
    PixelFormat128bppRGBAFloat,
    Unknown,
}

/// HDR image data structure, used to pass data between decoding and encoding
pub struct Image {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
}

impl From<jpegxr::PixelFormat> for PixelFormat {
    fn from(format: jpegxr::PixelFormat) -> Self {
        match format {
            jpegxr::PixelFormat::PixelFormat64bppRGBHalf => {
                PixelFormat::PixelFormat64bppRGBAHalfFloat
            }
            jpegxr::PixelFormat::PixelFormat128bppRGBAFloat => {
                PixelFormat::PixelFormat128bppRGBAFloat
            }
            _ => PixelFormat::Unknown,
        }
    }
}

impl From<PixelFormat> for ultrahdr::ImgFormat {
    fn from(val: PixelFormat) -> Self {
        match val {
            PixelFormat::PixelFormat64bppRGBAHalfFloat => {
                ultrahdr::ImgFormat::UHDR_IMG_FMT_64bppRGBAHalfFloat
            }
            _ => ultrahdr::ImgFormat::UHDR_IMG_FMT_UNSPECIFIED,
        }
    }
}
