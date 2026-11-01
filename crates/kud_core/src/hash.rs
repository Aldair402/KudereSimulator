//! Deterministic state hashing.
//!
//! Used by replay verification and regression tests to compare simulation
//! states without serializing them. The hash is FNV-1a over an explicit byte
//! sequence: no allocator, no platform hash seed, no iteration-order dependence.
//!
//! Callers must feed values in a canonical order. Hashing an unordered map
//! directly would make the result depend on the hash seed and defeat the point.
//!
//! The hash is stable across runs and platforms for the same input sequence.
//! It is not a cryptographic hash and must not be used as one.

/// FNV-1a offset basis for 64-bit hashes.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a prime for 64-bit hashes.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Incremental deterministic hasher for simulation state.
#[derive(Debug, Clone)]
pub struct StateHasher {
    state: u64,
}

impl Default for StateHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl StateHasher {
    /// Creates a hasher at the FNV-1a offset basis.
    #[inline]
    pub fn new() -> Self {
        Self {
            state: FNV_OFFSET_BASIS,
        }
    }

    /// Feeds raw bytes into the hash.
    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.state ^= u64::from(*byte);
            self.state = self.state.wrapping_mul(FNV_PRIME);
        }
    }

    /// Feeds a length-prefixed byte slice, so `["ab","c"]` differs from `["a","bc"]`.
    #[inline]
    pub fn write_slice(&mut self, bytes: &[u8]) {
        self.write_u64(bytes.len() as u64);
        self.write_bytes(bytes);
    }

    /// Feeds a string as length-prefixed UTF-8.
    #[inline]
    pub fn write_str(&mut self, value: &str) {
        self.write_slice(value.as_bytes());
    }

    /// Feeds a boolean.
    #[inline]
    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    /// Feeds one byte.
    #[inline]
    pub fn write_u8(&mut self, value: u8) {
        self.write_bytes(&[value]);
    }

    /// Feeds a 16-bit integer in little-endian order.
    #[inline]
    pub fn write_u16(&mut self, value: u16) {
        self.write_bytes(&value.to_le_bytes());
    }

    /// Feeds a 32-bit integer in little-endian order.
    #[inline]
    pub fn write_u32(&mut self, value: u32) {
        self.write_bytes(&value.to_le_bytes());
    }

    /// Feeds a 64-bit integer in little-endian order.
    #[inline]
    pub fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    /// Feeds a float by its bit pattern.
    ///
    /// Negative zero is normalized to positive zero so a sign flip on a zero
    /// component does not register as a state change. NaN is hashed as-is;
    /// simulation state is expected to be finite.
    #[inline]
    pub fn write_f32(&mut self, value: f32) {
        let normalized = if value == 0.0 { 0.0 } else { value };
        self.write_u32(normalized.to_bits());
    }

    /// Feeds a 3D vector component by component.
    #[inline]
    pub fn write_vec3(&mut self, value: glam::Vec3) {
        self.write_f32(value.x);
        self.write_f32(value.y);
        self.write_f32(value.z);
    }

    /// Returns the hash of everything written so far.
    #[inline]
    pub fn finish(&self) -> u64 {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_hash_is_the_offset_basis() {
        assert_eq!(StateHasher::new().finish(), FNV_OFFSET_BASIS);
    }

    #[test]
    fn matches_known_fnv1a_vector() {
        // Published FNV-1a 64 test vector for the ASCII string "hello".
        let mut hasher = StateHasher::new();
        hasher.write_bytes(b"hello");
        assert_eq!(hasher.finish(), 0xa430_d846_80aa_bd0b);
    }

    #[test]
    fn same_input_produces_same_hash() {
        let mut a = StateHasher::new();
        let mut b = StateHasher::new();
        for hasher in [&mut a, &mut b] {
            hasher.write_u32(7);
            hasher.write_f32(1.5);
            hasher.write_str("student");
            hasher.write_bool(true);
        }
        assert_eq!(a.finish(), b.finish());
    }

    #[test]
    fn field_order_matters() {
        let mut a = StateHasher::new();
        a.write_u32(1);
        a.write_u32(2);
        let mut b = StateHasher::new();
        b.write_u32(2);
        b.write_u32(1);
        assert_ne!(a.finish(), b.finish());
    }

    #[test]
    fn length_prefix_prevents_concatenation_collisions() {
        let mut a = StateHasher::new();
        a.write_str("ab");
        a.write_str("c");
        let mut b = StateHasher::new();
        b.write_str("a");
        b.write_str("bc");
        assert_ne!(a.finish(), b.finish());
    }

    #[test]
    fn negative_zero_hashes_as_zero() {
        let mut a = StateHasher::new();
        a.write_f32(-0.0);
        let mut b = StateHasher::new();
        b.write_f32(0.0);
        assert_eq!(a.finish(), b.finish());
    }

    #[test]
    fn tiny_float_change_changes_hash() {
        let mut a = StateHasher::new();
        a.write_f32(1.0);
        let mut b = StateHasher::new();
        b.write_f32(1.0 + f32::EPSILON);
        assert_ne!(a.finish(), b.finish());
    }
}
