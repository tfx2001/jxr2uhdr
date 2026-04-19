//! Emscripten export layer for JavaScript consumers.
//!
//! The exported functions use a C ABI so they can be called from the generated
//! `jxr2uhdr.js` glue with `Module.cwrap(...)` or direct `Module._...` calls.

use std::cell::RefCell;
use std::fmt::Display;
use std::ptr;
use std::slice;

use crate::decode::decode_jxr_bytes;
use crate::encode::encode_ultra_hdr_to_vec;
use crate::types::{Image, PixelFormat};

const RGBA_HALF_FLOAT_BYTES_PER_PIXEL: usize = 8;

thread_local! {
    static LAST_ERROR: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

pub struct ImageHandle {
    inner: Image,
}

pub struct BufferHandle {
    inner: Vec<u8>,
}

#[unsafe(no_mangle)]
pub extern "C" fn jxr_decode_jxr(input_ptr: *const u8, input_len: usize) -> *mut ImageHandle {
    export_result(|| {
        let input = unsafe { bytes_from_raw(input_ptr, input_len)? };
        let image = decode_jxr_bytes(input).map_err(stringify_error)?;
        Ok(Box::into_raw(Box::new(ImageHandle { inner: image })))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn jxr_encode_ultra_hdr(
    width: u32,
    height: u32,
    pixels_ptr: *const u8,
    pixels_len: usize,
    quality: i32,
) -> *mut BufferHandle {
    export_result(|| {
        let pixels = unsafe { bytes_from_raw(pixels_ptr, pixels_len)? };
        let mut image = image_from_rgba_half_float(width, height, pixels)?;
        let output = encode_ultra_hdr_to_vec(&mut image, quality).map_err(stringify_error)?;
        Ok(Box::into_raw(Box::new(BufferHandle { inner: output })))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn jxr_image_encode_ultra_hdr(
    handle: *mut ImageHandle,
    quality: i32,
) -> *mut BufferHandle {
    export_result(|| {
        let image = unsafe { image_handle_mut(handle)? };
        let output = encode_ultra_hdr_to_vec(&mut image.inner, quality).map_err(stringify_error)?;
        Ok(Box::into_raw(Box::new(BufferHandle { inner: output })))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn jxr_image_width(handle: *const ImageHandle) -> u32 {
    export_result(|| {
        let image = unsafe { image_handle_ref(handle)? };
        Ok(image.inner.width)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn jxr_image_height(handle: *const ImageHandle) -> u32 {
    export_result(|| {
        let image = unsafe { image_handle_ref(handle)? };
        Ok(image.inner.height)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn jxr_image_pixels_ptr(handle: *const ImageHandle) -> *const u8 {
    export_result(|| {
        let image = unsafe { image_handle_ref(handle)? };
        Ok(image.inner.pixels.as_ptr())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn jxr_image_pixels_len(handle: *const ImageHandle) -> usize {
    export_result(|| {
        let image = unsafe { image_handle_ref(handle)? };
        Ok(image.inner.pixels.len())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn jxr_image_free(handle: *mut ImageHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn jxr_buffer_ptr(handle: *const BufferHandle) -> *const u8 {
    export_result(|| {
        let buffer = unsafe { buffer_handle_ref(handle)? };
        Ok(buffer.inner.as_ptr())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn jxr_buffer_len(handle: *const BufferHandle) -> usize {
    export_result(|| {
        let buffer = unsafe { buffer_handle_ref(handle)? };
        Ok(buffer.inner.len())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn jxr_buffer_free(handle: *mut BufferHandle) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn jxr_last_error_message_ptr() -> *const u8 {
    LAST_ERROR.with(|slot| {
        let error = slot.borrow();
        if error.is_empty() {
            ptr::null()
        } else {
            error.as_ptr()
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn jxr_last_error_message_len() -> usize {
    LAST_ERROR.with(|slot| slot.borrow().len())
}

fn export_result<T: Default>(f: impl FnOnce() -> Result<T, String>) -> T {
    match f() {
        Ok(value) => {
            clear_last_error();
            value
        }
        Err(message) => {
            set_last_error(message);
            T::default()
        }
    }
}

fn image_from_rgba_half_float(width: u32, height: u32, pixels: &[u8]) -> Result<Image, String> {
    let expected_len = expected_rgba_half_float_len(width, height)?;
    if pixels.len() != expected_len {
        return Err(format!(
            "Expected {expected_len} bytes for a {width}x{height} 64bpp RGBA half-float image, got {} bytes",
            pixels.len()
        ));
    }

    Ok(Image {
        pixels: pixels.to_vec(),
        width,
        height,
        format: PixelFormat::PixelFormat64bppRGBAHalfFloat,
    })
}

fn expected_rgba_half_float_len(width: u32, height: u32) -> Result<usize, String> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(RGBA_HALF_FLOAT_BYTES_PER_PIXEL))
        .ok_or_else(|| "Image dimensions overflow the RGBA half-float buffer size".to_string())
}

unsafe fn bytes_from_raw<'a>(ptr: *const u8, len: usize) -> Result<&'a [u8], String> {
    if len == 0 {
        return Ok(&[]);
    }
    if ptr.is_null() {
        return Err("Received a null byte pointer with a non-zero length".to_string());
    }
    Ok(unsafe { slice::from_raw_parts(ptr, len) })
}

unsafe fn image_handle_ref(handle: *const ImageHandle) -> Result<&'static ImageHandle, String> {
    if handle.is_null() {
        return Err("Image handle is null".to_string());
    }
    Ok(unsafe { &*handle })
}

unsafe fn image_handle_mut(handle: *mut ImageHandle) -> Result<&'static mut ImageHandle, String> {
    if handle.is_null() {
        return Err("Image handle is null".to_string());
    }
    Ok(unsafe { &mut *handle })
}

unsafe fn buffer_handle_ref(handle: *const BufferHandle) -> Result<&'static BufferHandle, String> {
    if handle.is_null() {
        return Err("Buffer handle is null".to_string());
    }
    Ok(unsafe { &*handle })
}

fn set_last_error(message: String) {
    LAST_ERROR.with(|slot| {
        let mut error = slot.borrow_mut();
        error.clear();
        error.extend_from_slice(message.as_bytes());
    });
}

fn clear_last_error() {
    LAST_ERROR.with(|slot| slot.borrow_mut().clear());
}

fn stringify_error(error: impl Display) -> String {
    error.to_string()
}
