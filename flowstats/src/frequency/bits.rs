pub const VERSION: u8 = 1;
pub const FIXED_HEADER_SIZE: usize = 25;

/// Structure to hold a count-min-sketch as a byte array with a header.
///
/// Fixed header layout (25 bytes):
/// - Byte 0: version (1 byte)
/// - Bytes 1-8: length_bytes (8 bytes, u64 LE) - size of table data after seeds
/// - Bytes 9-12: depth (4 bytes, u32 LE)
/// - Bytes 13-16: width (4 bytes, u32 LE)
/// - Bytes 17-24: total_count (8 bytes, u64 LE)
///
/// Seeds section (immediately after fixed header):
/// - seeds: depth * 8 bytes (u64 LE each)
///
/// Data layout (after seeds):
/// - table: depth * width * 8 bytes (u64 LE each, row-major order)
#[derive(Clone, Debug)]
pub(crate) struct Bits {
    header_and_bits: Vec<u8>,
}

impl Bits {
    /// Create a new Bits structure with the given depth and table size.
    /// Total size = FIXED_HEADER_SIZE + seeds (depth * 8) + table_bytes
    pub fn new(depth: usize, table_bytes: usize) -> Self {
        let seeds_size = depth * 8;
        let total_size = FIXED_HEADER_SIZE + seeds_size + table_bytes;
        let mut header_and_bits = vec![0; total_size];

        let header = &mut header_and_bits[0..FIXED_HEADER_SIZE];
        Self::set_version(header, VERSION);
        Self::set_length_bytes(header, table_bytes as u64);
        Self::set_depth(header, depth as u32);
        Self::set_width(header, 0);
        Self::set_total_count(header, 0);

        Bits { header_and_bits }
    }

    /// Returns the offset where seeds start
    pub fn seeds_offset() -> usize {
        FIXED_HEADER_SIZE
    }

    /// Returns the offset where table data starts, given a depth
    pub fn table_offset(depth: usize) -> usize {
        FIXED_HEADER_SIZE + depth * 8
    }

    /// Returns the seeds section as a slice
    pub fn seeds(&self) -> &[u8] {
        let depth = Self::get_depth(self.header()) as usize;
        let start = FIXED_HEADER_SIZE;
        let end = start + depth * 8;
        &self.header_and_bits[start..end]
    }

    /// Returns the seeds section as a mutable slice
    pub fn seeds_mut(&mut self) -> &mut [u8] {
        let depth = Self::get_depth(self.header()) as usize;
        let start = FIXED_HEADER_SIZE;
        let end = start + depth * 8;
        &mut self.header_and_bits[start..end]
    }

    /// Returns the table data section (after fixed header and seeds)
    pub fn table(&self) -> &[u8] {
        let depth = Self::get_depth(self.header()) as usize;
        let offset = Self::table_offset(depth);
        &self.header_and_bits[offset..]
    }

    /// Returns the table data section as mutable
    pub fn table_mut(&mut self) -> &mut [u8] {
        let depth = Self::get_depth(self.header()) as usize;
        let offset = Self::table_offset(depth);
        &mut self.header_and_bits[offset..]
    }

    pub fn header(&self) -> &[u8] {
        &self.header_and_bits[0..FIXED_HEADER_SIZE]
    }

    pub fn header_mut(&mut self) -> &mut [u8] {
        &mut self.header_and_bits[0..FIXED_HEADER_SIZE]
    }

    pub fn get_version(header: &[u8]) -> u8 {
        header[0]
    }

    pub fn set_version(header: &mut [u8], version: u8) {
        header[0] = version;
    }

    pub fn get_depth(header: &[u8]) -> u32 {
        u32::from_le_bytes(header[9..13].try_into().unwrap())
    }

    pub fn set_depth(header: &mut [u8], depth: u32) {
        header[9..13].copy_from_slice(&depth.to_le_bytes());
    }

    pub fn get_width(header: &[u8]) -> u32 {
        u32::from_le_bytes(header[13..17].try_into().unwrap())
    }

    pub fn set_width(header: &mut [u8], width: u32) {
        header[13..17].copy_from_slice(&width.to_le_bytes());
    }

    pub fn get_total_count(header: &[u8]) -> u64 {
        u64::from_le_bytes(header[17..25].try_into().unwrap())
    }

    pub fn set_total_count(header: &mut [u8], total_count: u64) {
        header[17..25].copy_from_slice(&total_count.to_le_bytes());
    }

    pub fn get_length_bytes(header: &[u8]) -> u64 {
        u64::from_le_bytes(header[1..9].try_into().unwrap())
    }

    pub fn set_length_bytes(header: &mut [u8], length_bytes: u64) {
        header[1..9].copy_from_slice(&length_bytes.to_le_bytes());
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, &'static str> {
        if bytes.len() < FIXED_HEADER_SIZE {
            return Err("buffer too small for header");
        }
        let header = &bytes[0..FIXED_HEADER_SIZE];
        let version = Self::get_version(header);
        if version != VERSION {
            return Err("unsupported version");
        }
        let depth = Self::get_depth(header) as usize;
        let length_bytes = Self::get_length_bytes(header) as usize;
        let expected_size = FIXED_HEADER_SIZE + depth * 8 + length_bytes;
        if bytes.len() < expected_size {
            return Err("buffer too small for data");
        }
        Ok(Self { header_and_bits: bytes })
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.header_and_bits
    }

    pub fn realloc_large_heap_allocated_objects(mut self, f: fn(Vec<u8>) -> Vec<u8>) -> Self {
        let previous_len = self.header_and_bits.len();
        self.header_and_bits = f(self.header_and_bits);
        assert_eq!(previous_len, self.header_and_bits.len());
        assert_eq!(Self::get_version(self.header()), VERSION);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_correct_size() {
        // depth=4 means 32 bytes for seeds, table_bytes=100
        let bits = Bits::new(4, 100);
        let expected = FIXED_HEADER_SIZE + 4 * 8 + 100;
        assert_eq!(bits.header_and_bits.len(), expected);
        assert_eq!(bits.seeds().len(), 32);
        assert_eq!(bits.table().len(), 100);
        assert_eq!(bits.header().len(), FIXED_HEADER_SIZE);
    }

    #[test]
    fn test_new_sets_version() {
        let bits = Bits::new(1, 0);
        assert_eq!(Bits::get_version(bits.header()), VERSION);
    }

    #[test]
    fn test_new_sets_length_bytes() {
        let bits = Bits::new(2, 256);
        assert_eq!(Bits::get_length_bytes(bits.header()), 256);
    }

    #[test]
    fn test_new_sets_depth() {
        let bits = Bits::new(5, 0);
        assert_eq!(Bits::get_depth(bits.header()), 5);
    }

    #[test]
    fn test_depth_round_trip() {
        let mut bits = Bits::new(1, 0);
        Bits::set_depth(bits.header_mut(), 42);
        assert_eq!(Bits::get_depth(bits.header()), 42);
    }

    #[test]
    fn test_width_round_trip() {
        let mut bits = Bits::new(1, 0);
        Bits::set_width(bits.header_mut(), 1000);
        assert_eq!(Bits::get_width(bits.header()), 1000);
    }

    #[test]
    fn test_total_count_round_trip() {
        let mut bits = Bits::new(1, 0);
        Bits::set_total_count(bits.header_mut(), 9876543210);
        assert_eq!(Bits::get_total_count(bits.header()), 9876543210);
    }

    #[test]
    fn test_length_bytes_round_trip() {
        let mut bits = Bits::new(1, 0);
        Bits::set_length_bytes(bits.header_mut(), 123456789);
        assert_eq!(Bits::get_length_bytes(bits.header()), 123456789);
    }

    #[test]
    fn test_version_round_trip() {
        let mut bits = Bits::new(1, 0);
        Bits::set_version(bits.header_mut(), 99);
        assert_eq!(Bits::get_version(bits.header()), 99);
    }

    #[test]
    fn test_seeds_write_and_read() {
        let mut bits = Bits::new(2, 0);
        let seeds = bits.seeds_mut();
        seeds[0..8].copy_from_slice(&111u64.to_le_bytes());
        seeds[8..16].copy_from_slice(&222u64.to_le_bytes());

        let read_seeds = bits.seeds();
        let seed1 = u64::from_le_bytes(read_seeds[0..8].try_into().unwrap());
        let seed2 = u64::from_le_bytes(read_seeds[8..16].try_into().unwrap());
        assert_eq!(seed1, 111);
        assert_eq!(seed2, 222);
    }

    #[test]
    fn test_table_write_and_read() {
        let mut bits = Bits::new(1, 8);
        bits.table_mut().copy_from_slice(&42u64.to_le_bytes());

        let value = u64::from_le_bytes(bits.table().try_into().unwrap());
        assert_eq!(value, 42);
    }

    #[test]
    fn test_into_bytes_and_from_bytes_round_trip() {
        let mut bits = Bits::new(2, 8);
        Bits::set_width(bits.header_mut(), 100);
        Bits::set_total_count(bits.header_mut(), 5000);
        bits.seeds_mut()[0..8].copy_from_slice(&111u64.to_le_bytes());
        bits.seeds_mut()[8..16].copy_from_slice(&222u64.to_le_bytes());
        bits.table_mut().copy_from_slice(&12345u64.to_le_bytes());

        let bytes = bits.into_bytes();
        let restored = Bits::from_bytes(bytes).unwrap();

        assert_eq!(Bits::get_version(restored.header()), VERSION);
        assert_eq!(Bits::get_depth(restored.header()), 2);
        assert_eq!(Bits::get_width(restored.header()), 100);
        assert_eq!(Bits::get_total_count(restored.header()), 5000);
        assert_eq!(Bits::get_length_bytes(restored.header()), 8);

        let seed1 = u64::from_le_bytes(restored.seeds()[0..8].try_into().unwrap());
        let seed2 = u64::from_le_bytes(restored.seeds()[8..16].try_into().unwrap());
        assert_eq!(seed1, 111);
        assert_eq!(seed2, 222);

        let value = u64::from_le_bytes(restored.table().try_into().unwrap());
        assert_eq!(value, 12345);
    }

    #[test]
    fn test_from_bytes_error_buffer_too_small_for_header() {
        let small_buffer = vec![0u8; 16];
        let result = Bits::from_bytes(small_buffer);
        assert_eq!(result.unwrap_err(), "buffer too small for header");
    }

    #[test]
    fn test_from_bytes_error_unsupported_version() {
        let mut buffer = vec![0u8; FIXED_HEADER_SIZE];
        buffer[0] = 99; // invalid version
        let result = Bits::from_bytes(buffer);
        assert_eq!(result.unwrap_err(), "unsupported version");
    }

    #[test]
    fn test_from_bytes_error_buffer_too_small_for_data() {
        let mut buffer = vec![0u8; FIXED_HEADER_SIZE];
        Bits::set_version(&mut buffer, VERSION);
        Bits::set_depth(&mut buffer, 2);
        Bits::set_length_bytes(&mut buffer, 100);
        // buffer has header but no seeds or table data

        let result = Bits::from_bytes(buffer);
        assert_eq!(result.unwrap_err(), "buffer too small for data");
    }

    #[test]
    fn test_header_fields_do_not_overlap() {
        let mut bits = Bits::new(1, 0);
        Bits::set_version(bits.header_mut(), 1);
        Bits::set_length_bytes(bits.header_mut(), 0xFFFFFFFFFFFFFFFF);
        Bits::set_depth(bits.header_mut(), 0xAAAAAAAA);
        Bits::set_width(bits.header_mut(), 0xBBBBBBBB);
        Bits::set_total_count(bits.header_mut(), 0xCCCCCCCCCCCCCCCC);

        assert_eq!(Bits::get_version(bits.header()), 1);
        assert_eq!(Bits::get_length_bytes(bits.header()), 0xFFFFFFFFFFFFFFFF);
        assert_eq!(Bits::get_depth(bits.header()), 0xAAAAAAAA);
        assert_eq!(Bits::get_width(bits.header()), 0xBBBBBBBB);
        assert_eq!(Bits::get_total_count(bits.header()), 0xCCCCCCCCCCCCCCCC);
    }

    #[test]
    fn test_little_endian_encoding() {
        let mut bits = Bits::new(1, 0);
        Bits::set_width(bits.header_mut(), 0x04030201);

        let header = bits.header();
        // Width is at bytes 13-16, little-endian means LSB first
        assert_eq!(header[13], 0x01);
        assert_eq!(header[14], 0x02);
        assert_eq!(header[15], 0x03);
        assert_eq!(header[16], 0x04);
    }

    #[test]
    fn test_table_offset() {
        assert_eq!(Bits::table_offset(0), FIXED_HEADER_SIZE);
        assert_eq!(Bits::table_offset(1), FIXED_HEADER_SIZE + 8);
        assert_eq!(Bits::table_offset(5), FIXED_HEADER_SIZE + 40);
    }
}
