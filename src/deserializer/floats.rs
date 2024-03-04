use borsh::BorshDeserialize;

use crate::errors::{ChainparserError, ChainparserResult};

// Floats we saw in the logs for accounts of '4MangoMjqJ2firMokCjjGgoK8d4MXcrgL7XJaL3w6fVg'
// that borsh considered NaNs:
//
// [ 79, 103, 129, 255]
// [ 85, 255, 255, 255]
// [ 91,  62, 255, 255]
// [ 97, 176, 250, 255]
// [120, 254, 255, 255]
// [131, 253, 255, 255]
// [153, 165, 247, 255]
// [159, 203, 152, 255]
// [184,  89, 253, 255]
// [255, 255, 255, 255]

// However via tests I discovered that borsh doesn't handle `127` (0x7F) as last byte well either.
// Thus if the last 7 bits of the last byte are set then that denotes NaN

// For f64 the second to last byte has to have the first 4 bits set and the last byte has
// to be have the last 7 set to be considered NaN.

const LOWER7_BITS_MASK: u8 = 0b0111_1111;
const UPPER4_BITS_MASK: u8 = 0b1111_0000;

/// Deserializes f32 supporting NAN by identifying them instead of using borsh which
/// does not support NAN and errors instead.
pub fn deserialize_f32(buf: &mut &[u8]) -> ChainparserResult<f32> {
    if buf.len() >= 4 {
        let f32_slice = [buf[0], buf[1], buf[2], buf[3]];
        if (buf[3] & LOWER7_BITS_MASK) == LOWER7_BITS_MASK {
            *buf = &buf[4..];
            Ok(f32::NAN)
        } else {
            f32::deserialize(buf).map_err(|e| {
                ChainparserError::BorshDeserializeFloatError(
                    "f32".to_string(),
                    e,
                    f32_slice.to_vec(),
                )
            })
        }
    } else {
        f32::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeFloatError(
                "f32".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }
}

/// Deserializes f64 supporting NAN by identifying them instead of using borsh which
/// does not support NAN and errors instead.
pub fn deserialize_f64(buf: &mut &[u8]) -> ChainparserResult<f64> {
    if buf.len() >= 8 {
        let f64_slice = [
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7],
        ];
        if (buf[6] & UPPER4_BITS_MASK) == UPPER4_BITS_MASK
            && (buf[7] & LOWER7_BITS_MASK) == LOWER7_BITS_MASK
        {
            *buf = &buf[8..];
            Ok(f64::NAN)
        } else {
            f64::deserialize(buf).map_err(|e| {
                ChainparserError::BorshDeserializeFloatError(
                    "f64".to_string(),
                    e,
                    f64_slice.to_vec(),
                )
            })
        }
    } else {
        f64::deserialize(buf).map_err(|e| {
            ChainparserError::BorshDeserializeFloatError(
                "f64".to_string(),
                e,
                buf.to_vec(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_nan() {
        let cases = vec![
            [79, 103, 129, 0b1111_1111],
            [0, 0, 192, 0b0111_1111],
            [0, 0, 0, 0b0111_1111],
            [0, 0, 0, 0b1111_1111],
        ];
        for case in cases {
            let buf = case.to_vec();
            let res = deserialize_f32(&mut &buf[..]);
            assert!(res.unwrap().is_nan());
        }
    }

    #[test]
    fn f32_not_nan() {
        let cases = vec![
            [79, 103, 129, 0b1111_1110],
            [0, 0, 192, 0b0111_1101],
            [0, 0, 0, 0b0111_1101],
            [0, 0, 0, 0b1011_1111],
        ];
        for case in cases {
            let buf = case.to_vec();
            let res = deserialize_f32(&mut &buf[..]);
            assert!(!res.unwrap().is_nan());
        }
    }

    #[test]
    fn f64_nan() {
        let cases = vec![
            [100, 0, 0, 0, 79, 103, 0b1111_1111, 0b1111_1111],
            [100, 0, 0, 0, 79, 103, 0b1111_1111, 0b0111_1111],
            [100, 0, 0, 0, 79, 103, 0b1111_1000, 0b0111_1111],
            [100, 0, 0, 0, 79, 103, 0b1111_0000, 0b0111_1111],
            [100, 0, 0, 0, 79, 103, 0b1111_0001, 0b0111_1111],
            [100, 0, 0, 0, 79, 103, 0b1111_0101, 0b0111_1111],
            [100, 0, 0, 0, 79, 103, 0b1111_1101, 0b0111_1111],
        ];
        for case in cases {
            let buf = case.to_vec();
            let res = deserialize_f64(&mut &buf[..]);
            assert!(res.unwrap().is_nan());
        }
    }

    #[test]
    fn f64_not_nan() {
        let cases = vec![
            [100, 0, 0, 0, 79, 103, 129, 0b1111_1110],
            [100, 0, 0, 0, 0, 0, 192, 0b0111_1101],
            [100, 0, 0, 0, 0, 0, 0, 0b0111_1101],
            [100, 0, 0, 0, 0, 0, 0, 0b1011_1111],
            [100, 0, 0, 0, 79, 103, 0b0111_1111, 0b1111_1111],
        ];
        for case in cases {
            let buf = case.to_vec();
            let res = deserialize_f64(&mut &buf[..]);
            assert!(!res.unwrap().is_nan());
        }
    }
}
