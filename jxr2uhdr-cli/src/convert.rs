use anyhow::Result;
use half::f16;

/// Convert 128bpp RGBA f32 bytes to 64bpp RGBA f16 bytes
pub fn convert_128bpp_f32_to_64bpp_f16(f32_rgba_bytes: &[u8]) -> Result<Vec<u8>> {
    if f32_rgba_bytes.is_empty() {
        return Ok(Vec::new());
    }

    // Use bytemuck to zero-cost safely map [u8] to [f32] slice
    let f32_pixels: &[f32] = bytemuck::try_cast_slice(f32_rgba_bytes).map_err(|error| {
        anyhow::anyhow!("Failed to interpret 128bpp RGBA f32 bytes as f32 values: {error:?}")
    })?;

    // Iterate over all f32 channels and convert them to f16 using IEEE-754 standard
    let f16_pixels: Vec<u8> = f32_pixels
        .iter()
        .flat_map(|&val| f16::from_f32(val).to_le_bytes())
        .collect();

    Ok(f16_pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: convert f32 slice to raw bytes
    fn f32_to_bytes(values: &[f32]) -> Vec<u8> {
        bytemuck::cast_slice(values).to_vec()
    }

    #[test]
    fn test_single_pixel_rgba() {
        // One RGBA pixel: R=1.0, G=0.0, B=0.5, A=0.25
        let input = f32_to_bytes(&[1.0f32, 0.0, 0.5, 0.25]);
        let output = convert_128bpp_f32_to_64bpp_f16(&input)
            .expect("valid f32 RGBA bytes should convert to f16");

        // 16 bytes (4×f32) → 8 bytes (4×f16)
        assert_eq!(output.len(), input.len() / 2);

        // Verify each channel by reconstructing f16 values
        let f16_values: Vec<f16> = output
            .chunks_exact(2)
            .map(|c| f16::from_le_bytes([c[0], c[1]]))
            .collect();

        assert_eq!(f16_values[0].to_f32(), 1.0); // R
        assert_eq!(f16_values[1].to_f32(), 0.0); // G
        assert_eq!(f16_values[2].to_f32(), 0.5); // B
        assert_eq!(f16_values[3].to_f32(), 0.25); // A
    }

    #[test]
    fn test_empty_input() {
        let output = convert_128bpp_f32_to_64bpp_f16(&[])
            .expect("empty input should convert to empty output");
        assert!(output.is_empty());
    }

    #[test]
    fn test_multiple_pixels() {
        let input = f32_to_bytes(&[
            1.0, 0.0, 0.0, 1.0, // pixel 1: red
            0.0, 1.0, 0.0, 1.0, // pixel 2: green
        ]);
        let output = convert_128bpp_f32_to_64bpp_f16(&input)
            .expect("valid f32 RGBA bytes should convert to f16");
        assert_eq!(output.len(), 16); // 2 pixels × 4 channels × 2 bytes

        let f16_values: Vec<f32> = output
            .chunks_exact(2)
            .map(|c| f16::from_le_bytes([c[0], c[1]]).to_f32())
            .collect();

        // pixel 1
        assert_eq!(f16_values[0], 1.0);
        assert_eq!(f16_values[1], 0.0);
        assert_eq!(f16_values[2], 0.0);
        assert_eq!(f16_values[3], 1.0);
        // pixel 2
        assert_eq!(f16_values[4], 0.0);
        assert_eq!(f16_values[5], 1.0);
        assert_eq!(f16_values[6], 0.0);
        assert_eq!(f16_values[7], 1.0);
    }

    #[test]
    fn test_special_values() {
        let input = f32_to_bytes(&[f32::INFINITY, f32::NEG_INFINITY, -0.0, f32::MAX]);
        let output = convert_128bpp_f32_to_64bpp_f16(&input)
            .expect("valid f32 RGBA bytes should convert to f16");
        assert_eq!(output.len(), 8);

        let f16_values: Vec<f16> = output
            .chunks_exact(2)
            .map(|c| f16::from_le_bytes([c[0], c[1]]))
            .collect();

        assert!(f16_values[0].is_infinite() && f16_values[0].is_sign_positive());
        assert!(f16_values[1].is_infinite() && f16_values[1].is_sign_negative());
        assert_eq!(f16_values[2].to_f32(), -0.0);
        assert!(f16_values[3].is_infinite()); // f32::MAX overflows to inf in f16
    }

    #[test]
    fn rejects_input_that_cannot_be_cast_to_f32_values() {
        let error = convert_128bpp_f32_to_64bpp_f16(&[0, 1, 2])
            .expect_err("input length that is not a multiple of f32 size should be rejected");

        assert!(
            error
                .to_string()
                .contains("Failed to interpret 128bpp RGBA f32 bytes as f32 values"),
            "unexpected error: {error:#}"
        );
    }
}
