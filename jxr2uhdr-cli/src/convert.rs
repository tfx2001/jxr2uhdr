use half::f16;

/// Convert 128bpp RGBA f32 bytes to 64bpp RGBA f16 bytes
pub fn convert_128bpp_f32_to_64bpp_f16(f32_rgba_bytes: &[u8]) -> Vec<u8> {
    if f32_rgba_bytes.is_empty() {
        return Vec::new();
    }

    // Use bytemuck to zero-cost safely map [u8] to [f32] slice
    let f32_pixels: &[f32] = bytemuck::cast_slice(f32_rgba_bytes);

    // Iterate over all f32 channels and convert them to f16 using IEEE-754 standard
    let f16_pixels: Vec<u8> = f32_pixels
        .iter()
        .flat_map(|&val| f16::from_f32(val).to_le_bytes())
        .collect();

    f16_pixels
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
        let output = convert_128bpp_f32_to_64bpp_f16(&input);

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
        let output = convert_128bpp_f32_to_64bpp_f16(&[]);
        assert!(output.is_empty());
    }

    #[test]
    fn test_multiple_pixels() {
        let input = f32_to_bytes(&[
            1.0, 0.0, 0.0, 1.0, // pixel 1: red
            0.0, 1.0, 0.0, 1.0, // pixel 2: green
        ]);
        let output = convert_128bpp_f32_to_64bpp_f16(&input);
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
        let output = convert_128bpp_f32_to_64bpp_f16(&input);
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
}
