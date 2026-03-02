//! HKDF-SHA256 (RFC 5869) — native implementation over valuecore::hmac_sha256.
//!
//! extract(salt, ikm) = HMAC-SHA256(salt, ikm)
//! expand(prk, info, len) = T(1) || T(2) || ... truncated to len bytes
//!   where T(0) = b""
//!         T(i) = HMAC-SHA256(prk, T(i-1) || info || [i])

use crate::hmac_sha256::hmac_sha256;
use anyhow::{anyhow, Result};

/// Maximum output length for HKDF-SHA256: 255 * 32 bytes.
pub const HKDF_SHA256_MAX_LEN: usize = 255 * 32;

/// HKDF-Extract: returns a 32-byte pseudorandom key.
/// If salt is empty, uses a zero-filled 32-byte salt per RFC 5869 §2.2.
pub fn hkdf_extract(salt: &[u8], ikm: &[u8]) -> [u8; 32] {
    let effective_salt: &[u8] = if salt.is_empty() {
        &[0u8; 32]
    } else {
        salt
    };
    hmac_sha256(effective_salt, ikm)
}

/// HKDF-Expand: derives `len` bytes from `prk` and `info`.
/// Returns an error if `len` exceeds 255 * 32 = 8160 bytes.
pub fn hkdf_expand(prk: &[u8], info: &[u8], len: usize) -> Result<Vec<u8>> {
    if len == 0 {
        return Ok(vec![]);
    }
    if len > HKDF_SHA256_MAX_LEN {
        return Err(anyhow!("ERROR_BADARG hkdf_expand: len {} exceeds max {}", len, HKDF_SHA256_MAX_LEN));
    }
    let n = (len + 31) / 32; // ceil(len / HashLen)
    let mut out: Vec<u8> = Vec::with_capacity(n * 32);
    let mut t_prev: Vec<u8> = vec![];

    for i in 1u8..=(n as u8) {
        let mut data = Vec::with_capacity(t_prev.len() + info.len() + 1);
        data.extend_from_slice(&t_prev);
        data.extend_from_slice(info);
        data.push(i);
        let t = hmac_sha256(prk, &data);
        out.extend_from_slice(&t);
        t_prev = t.to_vec();
    }

    out.truncate(len);
    Ok(out)
}

/// HKDF-SHA256: extract then expand.
/// salt: can be empty (uses zero salt per RFC 5869)
/// ikm:  input keying material
/// info: context/application-specific info
/// len:  desired output length in bytes
pub fn hkdf_sha256(salt: &[u8], ikm: &[u8], info: &[u8], len: usize) -> Result<Vec<u8>> {
    let prk = hkdf_extract(salt, ikm);
    hkdf_expand(&prk, info, len)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 5869 Test Case 1
    #[test]
    fn rfc5869_test_case_1() {
        let ikm  = hex("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b");
        let salt = hex("000102030405060708090a0b0c");
        let info = hex("f0f1f2f3f4f5f6f7f8f9");
        let len  = 42;

        let prk_expected = hex("077709362c2e32df0ddc3f0dc47bba6390b6c73bb50f9c3122ec844ad7c2b3e5");
        let okm_expected = hex("3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865");

        let prk = hkdf_extract(&salt, &ikm);
        assert_eq!(prk.to_vec(), prk_expected, "PRK mismatch");

        let okm = hkdf_expand(&prk, &info, len).unwrap();
        assert_eq!(okm, okm_expected, "OKM mismatch");

        let combined = hkdf_sha256(&salt, &ikm, &info, len).unwrap();
        assert_eq!(combined, okm_expected, "combined mismatch");
    }

    // RFC 5869 Test Case 2
    #[test]
    fn rfc5869_test_case_2() {
        let ikm  = hex("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f\
                         202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f\
                         404142434445464748494a4b4c4d4e4f");
        let salt = hex("606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f\
                         808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f\
                         a0a1a2a3a4a5a6a7a8a9aaabacadaeaf");
        let info = hex("b0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0c1c2c3c4c5c6c7c8c9cacbcccdcecf\
                         d0d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaebecedeeef\
                         f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff");
        let len  = 82;

        let okm_expected = hex("b11e398dc80327a1c8e7f78c596a49344f012eda2d4efad8a050cc4c19afa97c\
                                 59045a99cac7827271cb41c65e590e09da3275600c2f09b8367793a9aca3db71\
                                 cc30c58179ec3e87c14c01d5c1f3434f1d87");

        let combined = hkdf_sha256(&salt, &ikm, &info, len).unwrap();
        assert_eq!(combined, okm_expected, "TC2 OKM mismatch");
    }

    // RFC 5869 Test Case 3 — zero-length salt and info
    #[test]
    fn rfc5869_test_case_3() {
        let ikm  = hex("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b");
        let salt = vec![];
        let info = vec![];
        let len  = 42;

        let okm_expected = hex("8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d\
                                 9d201395faa4b61a96c8");

        let combined = hkdf_sha256(&salt, &ikm, &info, len).unwrap();
        assert_eq!(combined, okm_expected, "TC3 OKM mismatch");
    }

    #[test]
    fn len_zero_returns_empty() {
        let out = hkdf_sha256(b"salt", b"ikm", b"info", 0).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn len_exceeds_max_is_error() {
        let e = hkdf_sha256(b"salt", b"ikm", b"info", HKDF_SHA256_MAX_LEN + 1).unwrap_err();
        assert!(e.to_string().contains("ERROR_BADARG"), "{}", e);
    }

    fn hex(s: &str) -> Vec<u8> {
        let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i+2], 16).unwrap())
            .collect()
    }
}
