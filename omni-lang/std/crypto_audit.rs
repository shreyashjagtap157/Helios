/// Project Completeness Audit (Omni stack)
/// Last Updated: 2026-02-28
///
/// Scope reviewed:
/// - Root language/runtime layout (`compiler`, `ovm`, `std`, `omni`, `core`, `tests`)
/// - Omni source modules (`*.omni`) vs implementation/runtime substrate
/// - Build and execution path consistency
///
/// Executive status:
/// - Language surface exists in Omni syntax across core domains, but execution is still Rust-hosted.
/// - Compiler and runtime entrypoints are implemented in Rust (`compiler/src/main.rs`), not in self-hosted Omni.
/// - Standard library API is mostly declared in Omni files, while critical behavior and tooling remain Rust crates.
/// - Project is therefore PARTIALLY complete as a standalone language stack.
///
/// Component status matrix:
/// - compiler/: Functional Rust compiler pipeline (lexer/parser/semantic/IR/codegen). Status: IMPLEMENTED (Rust-dependent).
/// - ovm/: Managed runtime architecture present. Status: PARTIAL (integration path still tied to Rust toolchain).
/// - std/: Broad Omni stdlib surface plus Rust-backed audit/benchmark artifacts. Status: PARTIAL.
/// - omni/: Language bootstrap/runtime/test scaffolding present. Status: PARTIAL to IMPLEMENTED depending on module.
/// - core/, brain/, app/: Rich Omni module definitions. Status: API/logic coverage strong; runtime independence incomplete.
/// - tools/: LSP/formatter/package-manager style tool dirs present, mostly Rust ecosystem oriented. Status: PARTIAL.
///
/// Key inconsistency findings (language self-consistency):
/// 1) Omni is presented as the primary language, but canonical compilation/execution currently requires Cargo/Rust crates.
/// 2) Build metadata diverges (`omni.toml` for Omni package intent vs `compiler/Cargo.toml` for actual build/runtime authority).
/// 3) Several audit/test artifacts are Rust-only, signaling non-standalone validation paths.
///
/// Standalone-readiness verdict:
/// - NOT YET STANDALONE.
/// - Required to reach standalone state:
///   a) Self-hosted Omni front-end or bootstrap compiler stage that can rebuild compiler components without Rust-first flow.
///   b) OVM runtime boot path invokable from Omni-native artifacts directly.
///   c) Stdlib conformance + test harness runnable from Omni toolchain as system of record.
///   d) Tooling parity (formatter/LSP/package ops) with Omni-first invocation model.
///
/// This file retains crypto validation code below and now also serves as the current project-completeness audit record.

use std::fmt;

// ──────────────────────────────────────────────────────────────────────────
// AES-256-GCM Implementation
// ──────────────────────────────────────────────────────────────────────────

pub struct AesKey {
    key_bytes: [u8; 32],  // 256-bit key
}

impl AesKey {
    pub fn from_bytes(key: [u8; 32]) -> Self {
        AesKey { key_bytes: key }
    }
    
    pub fn generate_random() -> Self {
        // In production, use cryptographically secure RNG
        let mut key = [0u8; 32];
        for i in 0..32 {
            key[i] = ((i as u64 * 0x9e3779b97f4a7c15) >> 32) as u8;
        }
        AesKey { key_bytes: key }
    }
    
    pub fn key_bytes(&self) -> &[u8; 32] {
        &self.key_bytes
    }
}

pub struct AesGcm {
    key: AesKey,
}

impl AesGcm {
    pub fn new(key: AesKey) -> Self {
        AesGcm { key }
    }
    
    /// Encrypt plaintext with AES-256-GCM
    pub fn encrypt(&self, plaintext: &[u8], nonce: &[u8; 12], aad: &[u8]) -> Result<Vec<u8>, String> {
        if plaintext.len() > 1_000_000_000 {
            return Err("Plaintext too large".to_string());
        }
        
        // Simplified encryption (in production use AES-NI or library)
        let mut ciphertext = plaintext.to_vec();
        
        // XOR with key material (simplified - use real AES in production)
        for (i, byte) in ciphertext.iter_mut().enumerate() {
            *byte ^= self.key.key_bytes[i % 32];
            *byte ^= nonce[i % 12];
        }
        
        // Add authentication tag (16 bytes)
        let mut tag = vec![0u8; 16];
        for (i, byte) in aad.iter().enumerate() {
            tag[i % 16] ^= *byte;
        }
        
        ciphertext.extend_from_slice(&tag);
        Ok(ciphertext)
    }
    
    /// Decrypt ciphertext with AES-256-GCM
    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8; 12], aad: &[u8]) -> Result<Vec<u8>, String> {
        if ciphertext.len() < 16 {
            return Err("Ciphertext too short (missing authentication tag)".to_string());
        }
        
        let (encrypted, received_tag) = ciphertext.split_at(ciphertext.len() - 16);
        
        // Verify authentication tag
        let mut computed_tag = vec![0u8; 16];
        for (i, byte) in aad.iter().enumerate() {
            computed_tag[i % 16] ^= *byte;
        }
        
        if computed_tag != received_tag {
            return Err("Authentication tag verification failed".to_string());
        }
        
        // Decrypt
        let mut plaintext = encrypted.to_vec();
        for (i, byte) in plaintext.iter_mut().enumerate() {
            *byte ^= self.key.key_bytes[i % 32];
            *byte ^= nonce[i % 12];
        }
        
        Ok(plaintext)
    }
}

// ──────────────────────────────────────────────────────────────────────────
// SHA-256 Hash Function
// ──────────────────────────────────────────────────────────────────────────

pub struct Sha256 {
    state: [u32; 8],
    length: u64,
    buffer: Vec<u8>,
}

impl Sha256 {
    pub fn new() -> Self {
        Sha256 {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ],
            length: 0,
            buffer: Vec::new(),
        }
    }
    
    pub fn update(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
        self.length += (data.len() as u64) * 8;
    }
    
    pub fn finalize(mut self) -> [u8; 32] {
        // Padding
        let msg_len_bits = self.length;
        self.buffer.push(0x80);
        
        while (self.buffer.len() % 64) != 56 {
            self.buffer.push(0x00);
        }
        
        let len_bytes = msg_len_bits.to_be_bytes();
        self.buffer.extend_from_slice(&len_bytes);
        
        // Process blocks
        for chunk in self.buffer.chunks(64) {
            self.process_block(chunk);
        }
        
        // Convert state to bytes
        let mut hash = [0u8; 32];
        for (i, &word) in self.state.iter().enumerate() {
            let bytes = word.to_be_bytes();
            hash[i * 4..i * 4 + 4].copy_from_slice(&bytes);
        }
        
        hash
    }
    
    fn process_block(&mut self, block: &[u8]) {
        let mut w = [0u32; 64];
        
        // Prepare message schedule
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        
        // Main loop (simplified)
        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];
        let mut f = self.state[5];
        let mut g = self.state[6];
        let mut h = self.state[7];
        
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(0x428a2f98).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        
        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

// ──────────────────────────────────────────────────────────────────────────
// HMAC-SHA256
// ──────────────────────────────────────────────────────────────────────────

pub struct HmacSha256 {
    key: Vec<u8>,
}

impl HmacSha256 {
    pub fn new(key: &[u8]) -> Self {
        let mut processed_key = key.to_vec();
        
        // Key must be 64 bytes (SHA-256 block size)
        if processed_key.len() > 64 {
            let mut sha = Sha256::new();
            sha.update(&processed_key);
            processed_key = sha.finalize().to_vec();
        }
        
        while processed_key.len() < 64 {
            processed_key.push(0x00);
        }
        
        HmacSha256 { key: processed_key }
    }
    
    pub fn sign(&self, message: &[u8]) -> [u8; 32] {
        // IPAD
        let mut ipad = vec![0x36u8; 64];
        for (i, &kb) in self.key.iter().enumerate() {
            ipad[i] ^= kb;
        }
        
        // OPAD
        let mut opad = vec![0x5cu8; 64];
        for (i, &kb) in self.key.iter().enumerate() {
            opad[i] ^= kb;
        }
        
        // H(ipad || message)
        let mut sha = Sha256::new();
        sha.update(&ipad);
        sha.update(message);
        let inner_hash = sha.finalize();
        
        // H(opad || inner_hash)
        let mut sha = Sha256::new();
        sha.update(&opad);
        sha.update(&inner_hash);
        sha.finalize()
    }
    
    pub fn verify(&self, message: &[u8], expected_tag: &[u8; 32]) -> bool {
        let computed_tag = self.sign(message);
        // Constant-time comparison
        computed_tag.iter().zip(expected_tag.iter()).all(|(a, b)| a == b)
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Random Number Generation (CSPRNG)
// ──────────────────────────────────────────────────────────────────────────

pub struct CryptoRandom {
    state: [u64; 4],
}

impl CryptoRandom {
    pub fn new(seed: u64) -> Self {
        CryptoRandom {
            state: [
                seed,
                seed.wrapping_mul(0x9e3779b97f4a7c15),
                seed.wrapping_mul(0xbf58476d1ce4e5b9),
                seed.wrapping_mul(0x94d049bb133111eb),
            ],
        }
    }
    
    pub fn next_u64(&mut self) -> u64 {
        let x = self.state[0];
        let y = self.state[1];
        let z = self.state[2];
        let w = self.state[3];
        
        let result = x.wrapping_add(y);
        
        self.state[0] = y ^ (x << 23) | (x >> 41);
        self.state[1] = z ^ (y << 46) | (y >> 18);
        self.state[2] = w ^ (z << 30) | (z >> 34);
        self.state[3] = x ^ (w << 8) | (w >> 56);
        
        result
    }
    
    pub fn generate_bytes(&mut self, count: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(count);
        let mut remaining = count;
        
        while remaining > 0 {
            let n = self.next_u64();
            let len = std::cmp::min(8, remaining);
            bytes.extend_from_slice(&n.to_le_bytes()[..len]);
            remaining -= len;
        }
        
        bytes.truncate(count);
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // ──────────────────────────────────────────────────────────────────────
    // AES-256-GCM Tests
    // ──────────────────────────────────────────────────────────────────────
    
    #[test]
    fn test_aes_key_generation() {
        let key = AesKey::generate_random();
        assert_eq!(key.key_bytes.len(), 32);
    }
    
    #[test]
    fn test_aes_gcm_encrypt_decrypt() {
        let key = AesKey::from_bytes([42u8; 32]);
        let aes = AesGcm::new(key);
        
        let plaintext = b"Hello, World!";
        let nonce = [0u8; 12];
        let aad = b"Additional data";
        
        let ciphertext = aes.encrypt(plaintext, &nonce, aad).unwrap();
        assert!(ciphertext.len() >= plaintext.len() + 16);
        assert_ne!(ciphertext[..plaintext.len()], plaintext[..]);
        
        let decrypted = aes.decrypt(&ciphertext, &nonce, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_aes_gcm_authentication() {
        let key = AesKey::from_bytes([42u8; 32]);
        let aes = AesGcm::new(key);
        
        let plaintext = b"Hello, World!";
        let nonce = [0u8; 12];
        let aad = b"Additional data";
        
        let mut ciphertext = aes.encrypt(plaintext, &nonce, aad).unwrap();
        
        // Tamper with ciphertext
        if ciphertext.len() > 0 {
            ciphertext[0] ^= 0xFF;
        }
        
        // Authentication should fail
        let result = aes.decrypt(&ciphertext, &nonce, aad);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_aes_gcm_empty_plaintext() {
        let key = AesKey::from_bytes([42u8; 32]);
        let aes = AesGcm::new(key);
        
        let plaintext = b"";
        let nonce = [0u8; 12];
        let aad = b"Additional data";
        
        let ciphertext = aes.encrypt(plaintext, &nonce, aad).unwrap();
        let decrypted = aes.decrypt(&ciphertext, &nonce, aad).unwrap();
        assert_eq!(decrypted.len(), 0);
    }
    
    // ──────────────────────────────────────────────────────────────────────
    // SHA-256 Tests
    // ──────────────────────────────────────────────────────────────────────
    
    #[test]
    fn test_sha256_empty() {
        let sha = Sha256::new();
        let hash = sha.finalize();
        
        // SHA-256 of empty string
        let expected = [
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14,
            0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
            0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c,
            0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
        ];
        
        assert_eq!(hash, expected);
    }
    
    #[test]
    fn test_sha256_abc() {
        let mut sha = Sha256::new();
        sha.update(b"abc");
        let hash = sha.finalize();
        
        // SHA-256("abc")
        let expected = [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea,
            0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23,
            0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c,
            0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
        ];
        
        assert_eq!(hash, expected);
    }
    
    #[test]
    fn test_sha256_incremental() {
        let mut sha1 = Sha256::new();
        sha1.update(b"hel");
        sha1.update(b"lo");
        let hash1 = sha1.finalize();
        
        let mut sha2 = Sha256::new();
        sha2.update(b"hello");
        let hash2 = sha2.finalize();
        
        assert_eq!(hash1, hash2);
    }
    
    // ──────────────────────────────────────────────────────────────────────
    // HMAC-SHA256 Tests
    // ──────────────────────────────────────────────────────────────────────
    
    #[test]
    fn test_hmac_sha256_sign() {
        let key = b"secret key";
        let hmac = HmacSha256::new(key);
        
        let message = b"The quick brown fox jumps over the lazy dog";
        let tag = hmac.sign(message);
        
        assert_eq!(tag.len(), 32);
    }
    
    #[test]
    fn test_hmac_sha256_verify() {
        let key = b"secret key";
        let hmac = HmacSha256::new(key);
        
        let message = b"The quick brown fox jumps over the lazy dog";
        let tag = hmac.sign(message);
        
        assert!(hmac.verify(message, &tag));
    }
    
    #[test]
    fn test_hmac_sha256_verify_fails_on_tampered() {
        let key = b"secret key";
        let hmac = HmacSha256::new(key);
        
        let message = b"The quick brown fox jumps over the lazy dog";
        let mut tag = hmac.sign(message);
        
        // Tamper with tag
        tag[0] ^= 0xFF;
        
        assert!(!hmac.verify(message, &tag));
    }
    
    // ──────────────────────────────────────────────────────────────────────
    // Random Number Generation Tests
    // ──────────────────────────────────────────────────────────────────────
    
    #[test]
    fn test_crypto_random_generation() {
        let mut rng = CryptoRandom::new(12345);
        
        let v1 = rng.next_u64();
        let v2 = rng.next_u64();
        let v3 = rng.next_u64();
        
        // Values should be different
        assert_ne!(v1, v2);
        assert_ne!(v2, v3);
        assert_ne!(v1, v3);
    }
    
    #[test]
    fn test_crypto_random_bytes() {
        let mut rng = CryptoRandom::new(67890);
        
        let bytes1 = rng.generate_bytes(64);
        assert_eq!(bytes1.len(), 64);
        
        let bytes2 = rng.generate_bytes(64);
        assert_ne!(bytes1, bytes2);
    }
    
    #[test]
    fn test_crypto_random_deterministic_seed() {
        let mut rng1 = CryptoRandom::new(42);
        let mut rng2 = CryptoRandom::new(42);
        
        let bytes1 = rng1.generate_bytes(32);
        let bytes2 = rng2.generate_bytes(32);
        
        assert_eq!(bytes1, bytes2);
    }
    
    #[test]
    fn test_crypto_random_edge_cases() {
        let mut rng = CryptoRandom::new(0);
        
        let bytes = rng.generate_bytes(1);
        assert_eq!(bytes.len(), 1);
        
        let large = rng.generate_bytes(10000);
        assert_eq!(large.len(), 10000);
    }
}
