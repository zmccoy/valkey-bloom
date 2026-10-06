// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: Copyright (c) 2025 flowstats Contributors
// SPDX-FileContributor: https://github.com/vnvo/flowstats/blob/v0.1.2/src/frequency/count_min.rs

//! Count-Min Sketch frequency estimator
//!
//! The Count-Min Sketch is a probabilistic data structure for estimating
//! the frequency of elements in a data stream.

use crate::frequency::bits::Bits;
use crate::math;
use crate::traits::{FrequencySketch, MergeError, Sketch};
use xxhash_rust::xxh3::xxh3_64_with_seed;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Count-Min Sketch for frequency estimation
///
/// The Count-Min Sketch provides frequency estimates with the following guarantees:
/// - Point query: `actual_count <= estimate <= actual_count + ε * N`
/// - Where ε = e/width and N is the total count
/// - Probability of exceeding the error bound: δ = 1/2^depth
///
/// # Example
///
/// ```
/// use flowstats::frequency::CountMinSketch;
/// use flowstats::traits::FrequencySketch;
///
/// // Create with 1% error rate and 0.1% failure probability
/// let mut cms = CountMinSketch::new(0.01, 0.001);
///
/// // Add items
/// cms.add(b"apple", 5);
/// cms.add(b"banana", 3);
/// cms.add(b"apple", 2);
///
/// // Query frequency
/// let apple_count = cms.estimate(b"apple"); // ~7
/// let banana_count = cms.estimate(b"banana"); // ~3
/// ```
#[derive(Clone, Debug)]
pub struct CountMinSketch {
    /// Width of each row
    width: usize,
    /// Number of rows (hash functions)
    depth: usize,
    /// Counter table
    table: Vec<Vec<u64>>,
    /// Total count of all items
    total_count: u64,
    /// Seeds for hash functions
    seeds: Vec<u64>,
}

impl CountMinSketch {
    /// Create a new Count-Min Sketch with the given error parameters
    ///
    /// # Arguments
    ///
    /// * `epsilon` - Maximum overcount as a fraction of total (e.g., 0.01 for 1%)
    /// * `delta` - Probability of exceeding the error bound (e.g., 0.001 for 0.1%)
    ///
    /// # Panics
    ///
    /// Panics if epsilon or delta are not in (0, 1)
    pub fn new(epsilon: f64, delta: f64) -> Self {
        assert!(epsilon > 0.0 && epsilon < 1.0, "epsilon must be in (0, 1)");
        assert!(delta > 0.0 && delta < 1.0, "delta must be in (0, 1)");

        // width = ceil(e / epsilon)
        // depth = ceil(ln(1/delta))
        let width = math::ceil(core::f64::consts::E / epsilon) as usize;
        let depth = math::ceil(math::ln(1.0 / delta)) as usize;

        Self::with_dimensions(width, depth)
    }

    /// Create a Count-Min Sketch with specific dimensions
    ///
    /// # Arguments
    ///
    /// * `width` - Width of each row (larger = lower error)
    /// * `depth` - Number of rows (larger = lower failure probability)
    pub fn with_dimensions(width: usize, depth: usize) -> Self {
        assert!(width > 0, "width must be positive");
        assert!(depth > 0, "depth must be positive");

        // Generate seeds for hash functions.  This is deterministic per position even for recreation. 
        let seeds: Vec<u64> = (0..depth)
            .map(|i| (i as u64).wrapping_mul(0x9e3779b97f4a7c15))
            .collect();

        Self {
            width,
            depth,
            table: vec![vec![0u64; width]; depth],
            total_count: 0,
            seeds,
        }
    }

    /// Get the width of the sketch
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the depth of the sketch
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Get the total count of all items
    pub fn total_count(&self) -> u64 {
        self.total_count
    }

    /// Add count to an item
    pub fn add(&mut self, item: &[u8], count: u64) {
        self.total_count += count;

        for (row, &seed) in self.seeds.iter().enumerate() {
            let hash = xxh3_64_with_seed(item, seed);
            let col = (hash as usize) % self.width;
            self.table[row][col] = self.table[row][col].saturating_add(count);
        }
    }

    //Multiplies the table using saturating_mul by the weight passed in.
    pub fn weight(&mut self, weight: u64) {
        for row in 0..self.depth {
            for column in 0..self.width {
                let v = self.table[row][column];
                self.table[row][column] = v.saturating_mul(weight);
            }
        }
    }

    /// Add count using conservative update
    ///
    /// Conservative update improves accuracy by only incrementing counters
    /// up to the new estimated value. This reduces over-counting.
    pub fn add_conservative(&mut self, item: &[u8], count: u64) {
        self.total_count += count;

        // First pass: find current estimate (minimum)
        let min_val = self.estimate(item);
        let new_val = min_val.saturating_add(count);

        // Second pass: set all counters to at least new_val
        for (row, &seed) in self.seeds.iter().enumerate() {
            let hash = xxh3_64_with_seed(item, seed);
            let col = (hash as usize) % self.width;

            if self.table[row][col] < new_val {
                self.table[row][col] = new_val;
            }
        }
    }

    /// Estimate the frequency of an item
    pub fn estimate(&self, item: &[u8]) -> u64 {
        let mut min_count = u64::MAX;

        for (row, &seed) in self.seeds.iter().enumerate() {
            let hash = xxh3_64_with_seed(item, seed);
            let col = (hash as usize) % self.width;
            min_count = min_count.min(self.table[row][col]);
        }

        min_count
    }

    /// Inner product of two sketches
    ///
    /// This can be used to estimate the dot product of two frequency distributions.
    pub fn inner_product(&self, other: &Self) -> Option<u64> {
        if self.width != other.width || self.depth != other.depth {
            return None;
        }

        let mut min_product = u64::MAX;

        for row in 0..self.depth {
            let product: u64 = self.table[row]
                .iter()
                .zip(other.table[row].iter())
                .fold(0u64, |acc, (&a, &b)| {
                    acc.saturating_add(a.saturating_mul(b))
                });
            min_product = min_product.min(product);
        }

        Some(min_product)
    }

    /// Theoretical error bound (epsilon * total_count)
    pub fn error_bound(&self) -> u64 {
        let epsilon = core::f64::consts::E / self.width as f64;
        (epsilon * self.total_count as f64) as u64
    }


    /// Serialize the sketch to little-endian bytes using the Bits format.
    ///
    /// Layout:
    /// - Fixed header (25 bytes): version, length, depth, width, total_count
    /// - Seeds: depth * 8 bytes (u64 LE each)
    /// - Table: depth * width * 8 bytes (u64 LE each, row-major)
    pub fn into_bytes(self) -> Vec<u8> {
        let table_size = self.depth * self.width * 8;

        let mut bits = Bits::new(self.depth, table_size);

        // Set header fields
        let header = bits.header_mut();
        Bits::set_width(header, self.width as u32);
        Bits::set_total_count(header, self.total_count);

        // Write seeds
        let seeds_data = bits.seeds_mut();
        for (i, &seed) in self.seeds.iter().enumerate() {
            let offset = i * 8;
            seeds_data[offset..offset + 8].copy_from_slice(&seed.to_le_bytes());
        }

        // Write table (row-major)
        let table_data = bits.table_mut();
        let mut offset = 0;
        for row in &self.table {
            for &counter in row {
                table_data[offset..offset + 8].copy_from_slice(&counter.to_le_bytes());
                offset += 8;
            }
        }

        bits.into_bytes()
    }

    /// Deserialize a sketch from little-endian bytes (takes ownership)
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, &'static str> {
        let bits = Bits::from_bytes(bytes)?;
        Self::from_bits(&bits)
    }

    /// Deserialize a sketch from little-endian bytes (borrows)
    pub fn from_existing(bytes: &[u8]) -> Result<Self, &'static str> {
        let bits = Bits::from_bytes(bytes.to_vec())?;
        Self::from_bits(&bits)
    }

    fn from_bits(bits: &Bits) -> Result<Self, &'static str> {
        let header = bits.header();
        let depth = Bits::get_depth(header) as usize;
        let width = Bits::get_width(header) as usize;
        let total_count = Bits::get_total_count(header);

        if width == 0 || depth == 0 {
            return Err("width and depth must be positive");
        }

        // Read seeds
        let seeds_data = bits.seeds();
        let mut seeds = Vec::with_capacity(depth);
        for i in 0..depth {
            let offset = i * 8;
            let seed = u64::from_le_bytes(seeds_data[offset..offset + 8].try_into().unwrap());
            seeds.push(seed);
        }

        // Read table
        let table_data = bits.table();
        let mut table = Vec::with_capacity(depth);
        let mut offset = 0;
        for _ in 0..depth {
            let mut row = Vec::with_capacity(width);
            for _ in 0..width {
                let counter = u64::from_le_bytes(table_data[offset..offset + 8].try_into().unwrap());
                row.push(counter);
                offset += 8;
            }
            table.push(row);
        }

        Ok(Self {
            width,
            depth,
            table,
            total_count,
            seeds,
        })
    }

    //Is this still needed with the deterministic creation above?
    pub fn new_by_dimension_with_seed(width: usize, depth: usize, seed: [u8; 32]) -> Self {
        todo!()
    }

 }

impl Sketch for CountMinSketch {
    type Item = [u8];

    fn update(&mut self, item: &[u8]) {
        self.add(item, 1);
    }

    fn merge(&mut self, other: &Self, weight: u64) -> Result<(), MergeError> {
        if self.width != other.width || self.depth != other.depth {
            return Err(MergeError::IncompatibleConfig {
                expected: format!("{}x{}", self.width, self.depth),
                found: format!("{}x{}", other.width, other.depth),
            });
        }

        for row in 0..self.depth {
            for col in 0..self.width {
                self.table[row][col] = self.table[row][col].saturating_add(other.table[row][col].saturating_mul(weight));
            }
        }

        self.total_count += other.total_count;

        Ok(())
    }

    fn clear(&mut self) {
        for row in &mut self.table {
            row.fill(0);
        }
        self.total_count = 0;
    }

    fn size_bytes(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.depth * self.width * core::mem::size_of::<u64>()
            + self.seeds.len() * core::mem::size_of::<u64>()
    }

}


impl FrequencySketch for CountMinSketch {
    fn estimate_frequency(&self, item: &[u8]) -> u64 {
        self.estimate(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut cms = CountMinSketch::new(0.01, 0.01);

        cms.add(b"apple", 5);
        cms.add(b"banana", 3);
        cms.add(b"cherry", 1);
        cms.add(b"apple", 2);

        // Estimates should be at least the true count
        assert!(cms.estimate(b"apple") >= 7);
        assert!(cms.estimate(b"banana") >= 3);
        assert!(cms.estimate(b"cherry") >= 1);
    }

    #[test]
    fn test_empty() {
        let cms = CountMinSketch::new(0.01, 0.01);
        assert_eq!(cms.estimate(b"anything"), 0);
        assert_eq!(cms.total_count(), 0);
    }

    #[test]
    fn test_conservative_update() {
        let mut cms1 = CountMinSketch::new(0.001, 0.001);
        let mut cms2 = CountMinSketch::new(0.001, 0.001);

        // Add many items
        for i in 0..10000 {
            let item = format!("item_{}", i);
            cms1.add(item.as_bytes(), 1);
            cms2.add_conservative(item.as_bytes(), 1);
        }

        // Conservative update should generally have lower estimates for items we query
        // (fewer collisions impacting the count)
        let test_item = b"test_item";
        cms1.add(test_item, 100);
        cms2.add_conservative(test_item, 100);

        // Both should report at least 100
        assert!(cms1.estimate(test_item) >= 100);
        assert!(cms2.estimate(test_item) >= 100);
    }

    #[test]
    fn test_merge() {
        let mut cms1 = CountMinSketch::with_dimensions(1000, 5);
        let mut cms2 = CountMinSketch::with_dimensions(1000, 5);

        cms1.add(b"apple", 5);
        cms2.add(b"banana", 3);

        cms1.merge(&cms2, 1).unwrap();

        assert!(cms1.estimate(b"apple") >= 5);
        assert!(cms1.estimate(b"banana") >= 3);
        assert_eq!(cms1.total_count(), 8);
    }

    #[test]
    fn test_merge_incompatible() {
        let mut cms1 = CountMinSketch::with_dimensions(1000, 5);
        let cms2 = CountMinSketch::with_dimensions(2000, 5);

        assert!(cms1.merge(&cms2, 1).is_err());
    }

    #[test]
    fn test_dimensions() {
        let cms = CountMinSketch::with_dimensions(1000, 5);
        assert_eq!(cms.width(), 1000);
        assert_eq!(cms.depth(), 5);
    }

    #[test]
    fn test_clear() {
        let mut cms = CountMinSketch::new(0.01, 0.01);

        cms.add(b"item", 100);
        assert!(cms.estimate(b"item") >= 100);

        cms.clear();
        assert_eq!(cms.estimate(b"item"), 0);
        assert_eq!(cms.total_count(), 0);
    }

    #[test]
    fn test_heavy_usage() {
        let mut cms = CountMinSketch::new(0.01, 0.001);

        // Add 100,000 items
        for i in 0..100_000 {
            let item = format!("user_{}", i % 1000); // 1000 unique items
            cms.add(item.as_bytes(), 1);
        }

        // Each item should have been added ~100 times
        // With 1% error, we allow some slack
        for i in 0..10 {
            let item = format!("user_{}", i);
            let estimate = cms.estimate(item.as_bytes());
            // Should be at least 100, and not more than 100 + error_bound
            assert!(estimate >= 100, "item {} estimate {} < 100", i, estimate);
        }
    }

    #[test]
    fn test_error_bound() {
        let cms = CountMinSketch::new(0.01, 0.001);
        // Error bound formula: epsilon * total_count
        // Initially 0
        assert_eq!(cms.error_bound(), 0);
    }

    #[test]
    fn test_inner_product() {
        let mut cms1 = CountMinSketch::with_dimensions(1000, 5);
        let mut cms2 = CountMinSketch::with_dimensions(1000, 5);

        cms1.add(b"a", 10);
        cms1.add(b"b", 5);

        cms2.add(b"a", 3);
        cms2.add(b"b", 2);

        // Inner product should give approximation of sum(f1[i] * f2[i])
        // = 10*3 + 5*2 = 40
        let ip = cms1.inner_product(&cms2).unwrap();
        assert!(ip >= 40, "inner product {} < 40", ip);
    }

    #[test]
    fn test_bytes_round_trip() {
        let mut cms = CountMinSketch::with_dimensions(100, 4);
        cms.add(b"apple", 5);
        cms.add(b"banana", 3);
        cms.add(b"cherry", 10);

        let original_width = cms.width();
        let original_depth = cms.depth();
        let original_total_count = cms.total_count();
        let apple_estimate = cms.estimate(b"apple");
        let banana_estimate = cms.estimate(b"banana");
        let cherry_estimate = cms.estimate(b"cherry");

        let bytes = cms.into_bytes();
        let restored = CountMinSketch::from_bytes(bytes).unwrap();

        assert_eq!(restored.width(), original_width);
        assert_eq!(restored.depth(), original_depth);
        assert_eq!(restored.total_count(), original_total_count);
        assert_eq!(restored.estimate(b"apple"), apple_estimate);
        assert_eq!(restored.estimate(b"banana"), banana_estimate);
        assert_eq!(restored.estimate(b"cherry"), cherry_estimate);
    }

    #[test]
    fn test_from_existing() {
        let mut cms = CountMinSketch::with_dimensions(50, 3);
        cms.add(b"test", 42);

        let bytes = cms.into_bytes();
        let restored = CountMinSketch::from_existing(&bytes).unwrap();

        assert_eq!(restored.width(), 50);
        assert_eq!(restored.depth(), 3);
        assert_eq!(restored.estimate(b"test"), 42);
    }
}
