use voprf::Group;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum Hash2CurveError {
    #[error("Hash to curve failed: {0}")]
    InternalError(String),
}

/// Hashes input data to a curve point using the hash_to_curve algorithm from RFC 9380.
///
/// This function implements the hash-to-curve operation as specified in RFC 9380,
/// using the Ristretto255 group and SHA-512 hash function.
///
/// # Arguments
/// * `input` - The input data to hash
/// * `dst` - Domain separation tag
///
/// # Returns
/// The serialized compressed curve point as a 32-byte array
#[uniffi::export]
pub fn hash_to_curve_ristretto255_sha512(
    input: Vec<u8>,
    dst: Vec<u8>,
) -> Result<Vec<u8>, Hash2CurveError> {
    use voprf::Ristretto255;

    let input_slice = input.as_slice();
    let dst_slice = dst.as_slice();

    let point = Ristretto255::hash_to_curve::<sha2::Sha512>(
        &[input_slice],
        &[dst_slice],
    )
    .map_err(|e| Hash2CurveError::InternalError(format!("{:?}", e)))?;

    Ok(point.compress().to_bytes().to_vec())
}

/// Hashes input data to a curve point using the hash_to_curve algorithm from RFC 9380.
///
/// This function implements the hash-to-curve operation as specified in RFC 9380,
/// using the NIST P-256 curve and SHA-256 hash function.
///
/// # Arguments
/// * `input` - The input data to hash
/// * `dst` - Domain separation tag
///
/// # Returns
/// The serialized compressed curve point as a 33-byte array (compressed SEC1 format)
#[uniffi::export]
pub fn hash_to_curve_p256_sha256(
    input: Vec<u8>,
    dst: Vec<u8>,
) -> Result<Vec<u8>, Hash2CurveError> {
    use p256::elliptic_curve::sec1::ToEncodedPoint;

    let input_slice = input.as_slice();
    let dst_slice = dst.as_slice();

    let point = p256::NistP256::hash_to_curve::<sha2::Sha256>(
        &[input_slice],
        &[dst_slice],
    )
    .map_err(|e| Hash2CurveError::InternalError(format!("{:?}", e)))?;

    Ok(point.to_encoded_point(true).as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p256_hash_to_curve_rfc9380() {
        // Test vector from RFC 9380 Section J.9.1
        // P256_XMD:SHA-256_SSWU_RO_
        // https://www.rfc-editor.org/rfc/rfc9380.html#name-p256_xmdsha-256_sswu_ro_

        let msg = b"abcdef0123456789";
        let dst = b"QUUX-V01-CS02-with-P256_XMD:SHA-256_SSWU_RO_";

        // Expected point P (compressed format, 0x02 prefix + x coordinate)
        // P.x = 65038ac8f2b1def042a5df0b33b1f4eca6bff7cb0f9c6c1526811864e544ed80
        // P.y = cad44d40a656e7aff4002a8de287abc8ae0482b5ae825822bb870d6df9b56ca3
        // Since P.y is even (last byte 0xa3 is odd), but we need to check the actual parity
        let expected_x = hex::decode("65038ac8f2b1def042a5df0b33b1f4eca6bff7cb0f9c6c1526811864e544ed80").unwrap();

        let result = hash_to_curve_p256_sha256(msg.to_vec(), dst.to_vec()).unwrap();

        // Compressed point format: 1 byte prefix (0x02 or 0x03) + 32 bytes x-coordinate
        assert_eq!(result.len(), 33);

        // Check y parity prefix
        assert!(result[0] == 0x03);

        // Check the x-coordinate matches
        assert_eq!(&result[1..], expected_x.as_slice());
    }

    #[test]
    fn test_ristretto255_hash_to_curve() {
        // Test that our Ristretto255 hash-to-curve function produces correct output
        // Note: RFC 9380 doesn't have test vectors for hash_to_curve using Ristretto255,
        // which is why we just use our own known outputs here.

        let dst = b"test_ristretto255_hash_to_curve";

        let test_cases = [
            (
                b"2007",
                "c81055e3b04a95399c3a18f89c05e3c78d2949b0a1d82f939e54b49d1fdc761b",
            ),
            (
                b"2009",
                "7cfcc8d02a65f174e2d922bdb36d575f7ed4b96609f68cb0fe0d9cf2a8705754",
            ),
        ];

        for (msg, expected_hex) in test_cases {
            let result = hash_to_curve_ristretto255_sha512(msg.to_vec(), dst.to_vec()).unwrap();

            // Ristretto255 compressed point is always 32 bytes
            assert_eq!(result.len(), 32);

            let result_hex = hex::encode(&result);
            assert_eq!(result_hex, expected_hex, "Failed for message: {:?}", String::from_utf8_lossy(msg));
        }
    }
}