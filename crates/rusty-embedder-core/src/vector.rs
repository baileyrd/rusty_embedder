use crate::error::{EmbedError, Result};

/// Packs a vector of `f32`s into little-endian bytes.
///
/// This matches exactly what `sqlite-vec`'s `vec0` virtual tables expect
/// on disk: a contiguous little-endian float32 array, with no length
/// prefix or header - the same layout `knowledge-mcp`'s Python
/// `serialize_f32` (`struct.pack(f"<{len(vector)}f", *vector)`) produces.
/// Callers store the result directly into a `vec0` column.
pub fn serialize_f32(vector: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vector.len() * 4);
    for value in vector {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

/// Unpacks a little-endian float32 byte blob (as produced by
/// [`serialize_f32`], or read back from a `sqlite-vec` `vec0` column) into
/// a `Vec<f32>`.
///
/// Returns [`EmbedError::InvalidInput`] if `bytes` isn't a whole number of
/// 4-byte float32s, rather than panicking on a malformed blob.
pub fn deserialize_f32(bytes: &[u8]) -> Result<Vec<f32>> {
    if bytes.len() % 4 != 0 {
        return Err(EmbedError::InvalidInput(format!(
            "byte blob length {} is not a multiple of 4 (not a valid f32 array)",
            bytes.len()
        )));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_serialize_and_deserialize() {
        let original = vec![1.0f32, -2.5, 0.0, 3.4028235e38, -1.1754944e-38];
        let bytes = serialize_f32(&original);
        assert_eq!(bytes.len(), original.len() * 4);
        assert_eq!(deserialize_f32(&bytes).unwrap(), original);
    }

    #[test]
    fn matches_known_little_endian_layout() {
        // 1.0f32 in little-endian IEEE-754 bytes is 00 00 80 3F - the same
        // layout sqlite-vec's vec0 tables and Python's struct.pack("<f",
        // 1.0) both produce.
        assert_eq!(serialize_f32(&[1.0f32]), vec![0x00, 0x00, 0x80, 0x3F]);
    }

    #[test]
    fn empty_vector_roundtrips_to_empty_bytes() {
        assert!(serialize_f32(&[]).is_empty());
        assert_eq!(deserialize_f32(&[]).unwrap(), Vec::<f32>::new());
    }

    #[test]
    fn rejects_byte_length_not_a_multiple_of_four_instead_of_panicking() {
        let err = deserialize_f32(&[0, 1, 2]).unwrap_err();
        assert!(matches!(err, EmbedError::InvalidInput(_)));
    }
}
