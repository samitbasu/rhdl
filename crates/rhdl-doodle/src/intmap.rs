/// A map from non-overlapping half-open intervals `[start, end)` to values of type `V`.
///
/// # Representation
/// Internally backed by a `BTreeMap<T, (T, V)>` where:
/// - key = interval start (inclusive)
/// - value = (interval end (exclusive), payload V)
///
/// This gives O(log n) time for every operation and lets callers hold
/// `&mut V` references without any special ceremony.
///
/// # Invariants (always maintained)
/// * Every stored interval is non-empty (`start < end`).
/// * No two stored intervals overlap (half-open: `[a,b)` and `[b,c)` are *not* overlapping).
use std::{collections::BTreeMap, fmt, ops::Range};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum IntervalMapError {
    /// `range.start >= range.end`
    #[error("interval is empty (start >= end)")]
    EmptyInterval,
    /// The interval overlaps one or more existing intervals.
    #[error("interval overlaps an existing entry")]
    Overlap,
    /// No interval contains the queried point.
    #[error("no interval contains the given point")]
    NotFound,
    /// Split point is not strictly inside the interval (`start < point < end` required).
    #[error("split point is not strictly inside the interval")]
    InvalidSplitPoint,
    /// The two intervals at the boundary don't adjoin, or their values differ.
    #[error("intervals do not adjoin or values differ")]
    CannotMerge,
}

// ---------------------------------------------------------------------------
// IntervalMap
// ---------------------------------------------------------------------------

pub struct IntervalMap<T, V> {
    /// Key: interval start (inclusive). Value: (interval end exclusive, payload).
    inner: BTreeMap<T, (T, V)>,
}

impl<T, V> IntervalMap<T, V>
where
    T: Ord + Copy,
{
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Check if a point is contained in any interval.
    pub fn contains_point(&self, point: T) -> bool {
        self.get(point).is_some()
    }

    /// Remove any interval containing the given point.
    /// Returns the removed interval and value if found.
    pub fn remove_containing(&mut self, point: T) -> Option<(Range<T>, V)> {
        let (start, _) = self.get(point)?;
        self.remove_exact(start.start)
    }

    /// Remove all intervals, making the map empty.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    // -----------------------------------------------------------------------
    // Core overlap check (O(log n))
    // -----------------------------------------------------------------------

    /// Returns `true` if `range` overlaps any stored interval.
    ///
    /// Two half-open intervals overlap iff `a.start < b.end && b.start < a.end`.
    pub fn has_overlap(&self, range: &Range<T>) -> bool {
        if range.start >= range.end {
            return false; // Empty ranges don't overlap anything
        }

        // 1. The last interval that *starts before* range.start might extend into it.
        if let Some((_, (end, _))) = self.inner.range(..range.start).next_back() {
            if *end > range.start {
                return true;
            }
        }
        // 2. Any interval starting inside [range.start, range.end) must overlap.
        self.inner.range(range.start..range.end).next().is_some()
    }

    // -----------------------------------------------------------------------
    // Insertion / removal (O(log n))
    // -----------------------------------------------------------------------

    /// Insert `range → value`. Fails if the range is empty or overlaps an existing interval.
    pub fn insert(&mut self, range: Range<T>, value: V) -> Result<(), IntervalMapError> {
        if range.start >= range.end {
            return Err(IntervalMapError::EmptyInterval);
        }
        if self.has_overlap(&range) {
            return Err(IntervalMapError::Overlap);
        }
        self.inner.insert(range.start, (range.end, value));
        Ok(())
    }

    /// Remove the interval whose start equals `start` exactly.
    /// Returns `(Range<T>, V)` if found.
    pub fn remove_exact(&mut self, start: T) -> Option<(Range<T>, V)> {
        self.inner.remove(&start).map(|(end, v)| (start..end, v))
    }

    // -----------------------------------------------------------------------
    // Point lookup (O(log n))
    // -----------------------------------------------------------------------

    /// Return the interval + immutable value reference containing `point`, or `None`.
    pub fn get(&self, point: T) -> Option<(Range<T>, &V)> {
        let (start, (end, value)) = self.inner.range(..=point).next_back()?;
        (*end > point).then_some((*start..*end, value))
    }

    /// Return the interval + mutable value reference containing `point`, or `None`.
    pub fn get_mut(&mut self, point: T) -> Option<(Range<T>, &mut V)> {
        // Find the interval that might contain the point
        let mut cursor = self.inner.range_mut(..=point);
        let (start, (end, value)) = cursor.next_back()?;

        if *end > point {
            Some((*start..*end, value))
        } else {
            None
        }
    }

    // -----------------------------------------------------------------------
    // Split (O(log n))
    // -----------------------------------------------------------------------

    /// Split the interval containing `point` into `[start, point)` and `[point, end)`.
    ///
    /// `point` must satisfy `start < point < end` (strictly inside).
    /// The `split_fn` function takes the original value and returns `(left_value, right_value)`.
    ///
    /// Returns `(Range<T>, Range<T>)` for the left and right intervals created.
    pub fn split<F>(
        &mut self,
        point: T,
        split_fn: F,
    ) -> Result<(Range<T>, Range<T>), IntervalMapError>
    where
        F: FnOnce(V) -> (V, V),
    {
        // --- find the enclosing interval (start < point required) ---
        let (start, end) = {
            let (s, (e, _)) = self
                .inner
                .range(..point) // strictly before point → guarantees start < point
                .next_back()
                .ok_or(IntervalMapError::NotFound)?;
            let (s, e) = (*s, *e);
            if e <= point {
                // The interval ends at or before point: point not inside.
                return Err(IntervalMapError::InvalidSplitPoint);
            }
            (s, e)
        };
        // By construction: start < point < end.

        // --- atomically replace with two halves ---
        let (_, orig) = self.inner.remove(&start).expect("just found it");
        let (left_val, right_val) = split_fn(orig);
        self.inner.insert(start, (point, left_val));
        self.inner.insert(point, (end, right_val));

        Ok((start..point, point..end))
    }

    // -----------------------------------------------------------------------
    // Merge (O(log n))
    // -----------------------------------------------------------------------

    /// Attempt to merge the interval *ending* at `boundary` with the one *starting* at `boundary`.
    ///
    /// Succeeds only when:
    /// * An interval `[a, boundary)` exists.
    /// * An interval `[boundary, b)` exists.
    /// * Their values compare equal (`V: PartialEq`).
    ///
    /// On success the two entries are replaced by `[a, b)` carrying the shared value.
    pub fn try_merge_at(&mut self, boundary: T) -> Result<(), IntervalMapError>
    where
        V: PartialEq,
    {
        // Right interval must start exactly at boundary.
        let right_end = match self.inner.get(&boundary) {
            Some((e, _)) => *e,
            None => return Err(IntervalMapError::CannotMerge),
        };

        // Left interval must end exactly at boundary.
        let left_start = {
            let (s, (e, _)) = self
                .inner
                .range(..boundary)
                .next_back()
                .ok_or(IntervalMapError::CannotMerge)?;
            if *e != boundary {
                return Err(IntervalMapError::CannotMerge);
            }
            *s
        };

        // Values must be equal.
        if self.inner[&left_start].1 != self.inner[&boundary].1 {
            return Err(IntervalMapError::CannotMerge);
        }

        // Merge: remove right, extend left's end.
        self.inner.remove(&boundary);
        self.inner.get_mut(&left_start).unwrap().0 = right_end;
        Ok(())
    }

    /// Check that a range is covered
    pub fn covered(&self, range: &Range<T>) -> bool {
        if range.start >= range.end {
            return true; // Empty range is trivially covered
        }
        let intervals = self.iter_range(range).map(|r| r.0).collect::<Vec<_>>();
        if intervals.is_empty() {
            return false; // No intervals at all
        }
        if intervals.first().unwrap().start != range.start {
            return false;
        }
        if intervals.last().unwrap().end != range.end {
            return false;
        }
        for w in intervals.windows(2) {
            if w[0].end != w[1].start {
                return false; // Gap between intervals
            }
        }
        true
    }

    /// Iterate immutably over every stored interval that overlaps `range`, in order.
    pub fn iter_range(&self, range: &Range<T>) -> impl Iterator<Item = (Range<T>, &V)> {
        // An interval starting *before* range.start might still overlap if its end > range.start.
        let first = self
            .inner
            .range(..range.start)
            .next_back()
            .filter(|(_, (e, _))| *e > range.start)
            .map(|(s, _)| *s)
            .unwrap_or(range.start);

        self.inner
            .range(first..range.end)
            .map(|(s, (e, v))| (*s..*e, v))
    }

    /// Iterate mutably over every stored interval that overlaps `range`, in order.
    /// The interval bounds themselves are not mutable.
    pub fn iter_range_mut(&mut self, range: &Range<T>) -> impl Iterator<Item = (Range<T>, &mut V)> {
        // Determine the scan-start with an immutable borrow, then drop it.
        let first = self
            .inner
            .range(..range.start)
            .next_back()
            .filter(|(_, (e, _))| *e > range.start)
            .map(|(s, _)| *s)
            .unwrap_or(range.start);

        self.inner
            .range_mut(first..range.end)
            .map(|(s, (e, v))| (*s..*e, v))
    }

    // -----------------------------------------------------------------------
    // Invariant validation (for testing)
    // -----------------------------------------------------------------------

    /// Check that all invariants are satisfied. Returns `true` if valid, `false` otherwise.
    ///
    /// The invariants checked are:
    /// 1. Every stored interval is non-empty (`start < end`).
    /// 2. No two stored intervals overlap (half-open intervals).
    /// 3. Intervals are stored in sorted order (guaranteed by BTreeMap but verified).
    pub fn is_valid(&self) -> bool {
        let mut prev_end: Option<T> = None;

        for (start, (end, _)) in &self.inner {
            // Invariant 1: Every interval must be non-empty
            if start >= end {
                return false;
            }

            // Invariant 2 & 3: No overlaps and sorted order
            if let Some(prev_end_val) = prev_end {
                if prev_end_val > *start {
                    return false; // Overlap detected
                }
            }

            prev_end = Some(*end);
        }

        true
    }
}

impl<T: Ord + Copy + fmt::Debug, V: Clone + fmt::Debug> fmt::Debug for IntervalMap<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut m = f.debug_map();
        for (s, (e, v)) in &self.inner {
            m.entry(&format_args!("{s:?}..{e:?}"), v);
        }
        m.finish()
    }
}

impl<T: Ord + Copy, V: Clone> Clone for IntervalMap<T, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Ord + Copy, V> Default for IntervalMap<T, V> {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // A newtyped signed integer, just like the caller will use.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct Pos(i64);

    fn pos(n: i64) -> Pos {
        Pos(n)
    }

    #[test]
    fn basic_insert_and_get() {
        let mut m: IntervalMap<Pos, &str> = IntervalMap::new();
        m.insert(pos(0)..pos(10), "hello").unwrap();
        let (range, &val) = m.get(pos(5)).unwrap();
        assert_eq!(range, pos(0)..pos(10));
        assert_eq!(val, "hello");
        assert!(m.get(pos(10)).is_none()); // half-open: 10 is not included
        assert!(m.is_valid());
    }

    #[test]
    fn overlap_rejected() {
        let mut m: IntervalMap<Pos, i32> = IntervalMap::new();
        m.insert(pos(0)..pos(10), 1).unwrap();
        assert_eq!(m.insert(pos(5)..pos(15), 2), Err(IntervalMapError::Overlap));
        assert_eq!(m.insert(pos(0)..pos(10), 3), Err(IntervalMapError::Overlap));
        // Adjacent is allowed
        m.insert(pos(10)..pos(20), 2).unwrap();
        assert!(m.is_valid());
    }

    #[test]
    fn split_and_mutate() {
        let mut m: IntervalMap<Pos, i32> = IntervalMap::new();
        m.insert(pos(0)..pos(100), 42).unwrap();

        let (left_range, right_range) = m.split(pos(50), |_orig| (1, 2)).unwrap();

        // Verify the values were set correctly by the split function
        let (_, &v_left) = m.get(pos(25)).unwrap();
        let (_, &v_right) = m.get(pos(75)).unwrap();
        assert_eq!(v_left, 1);
        assert_eq!(v_right, 2);

        // Also test that we can still mutate after split using the ranges
        *m.get_mut(left_range.start).unwrap().1 = 10;
        *m.get_mut(right_range.start).unwrap().1 = 20;

        let (_, &v_left) = m.get(pos(25)).unwrap();
        let (_, &v_right) = m.get(pos(75)).unwrap();
        assert_eq!(v_left, 10);
        assert_eq!(v_right, 20);
        assert!(m.is_valid());
    }

    #[test]
    fn split_using_original_value() {
        let mut m: IntervalMap<Pos, i32> = IntervalMap::new();
        m.insert(pos(0)..pos(100), 42).unwrap();

        // Split using the original value - left gets original, right gets original + 1
        let (_left_range, _right_range) = m.split(pos(50), |orig| (orig, orig + 1)).unwrap();

        let (_, &v_left) = m.get(pos(25)).unwrap();
        let (_, &v_right) = m.get(pos(75)).unwrap();
        assert_eq!(v_left, 42);
        assert_eq!(v_right, 43);
        assert!(m.is_valid());
        assert!(m.covered(&(pos(0)..pos(100))));
    }

    #[test]
    fn new_convenience_methods() {
        let mut m: IntervalMap<Pos, i32> = IntervalMap::new();
        m.insert(pos(0)..pos(10), 1).unwrap();
        m.insert(pos(20)..pos(30), 2).unwrap();

        // Test contains_point
        assert!(m.contains_point(pos(5)));
        assert!(!m.contains_point(pos(15)));

        // Test remove_containing
        let removed = m.remove_containing(pos(25)).unwrap();
        assert_eq!(removed, (pos(20)..pos(30), 2));
        assert!(!m.contains_point(pos(25)));

        // Test clear
        m.clear();
        assert!(m.is_empty());
    }

    #[test]
    fn merge() {
        let mut m: IntervalMap<Pos, i32> = IntervalMap::new();
        m.insert(pos(0)..pos(50), 7).unwrap();
        m.insert(pos(50)..pos(100), 7).unwrap();
        m.try_merge_at(pos(50)).unwrap();
        assert_eq!(m.len(), 1);
        let (range, _) = m.get(pos(75)).unwrap();
        assert_eq!(range, pos(0)..pos(100));
        assert!(m.is_valid());
    }

    #[test]
    fn merge_rejects_different_values() {
        let mut m: IntervalMap<Pos, i32> = IntervalMap::new();
        m.insert(pos(0)..pos(50), 1).unwrap();
        m.insert(pos(50)..pos(100), 2).unwrap();
        assert_eq!(m.try_merge_at(pos(50)), Err(IntervalMapError::CannotMerge));
        assert!(m.is_valid());
    }

    #[test]
    fn iter_range() {
        let mut m: IntervalMap<Pos, i32> = IntervalMap::new();
        m.insert(pos(0)..pos(10), 1).unwrap();
        m.insert(pos(10)..pos(20), 2).unwrap();
        m.insert(pos(20)..pos(30), 3).unwrap();
        m.insert(pos(30)..pos(40), 4).unwrap();

        // Query overlaps intervals 2 and 3 (and partially 1).
        let hits: Vec<_> = m
            .iter_range(&(pos(5)..pos(25)))
            .map(|(r, &v)| (r, v))
            .collect();
        assert_eq!(
            hits,
            vec![
                (pos(0)..pos(10), 1),
                (pos(10)..pos(20), 2),
                (pos(20)..pos(30), 3),
            ]
        );
        assert!(m.is_valid());
    }

    #[test]
    fn iter_range_mut() {
        let mut m: IntervalMap<Pos, i32> = IntervalMap::new();
        m.insert(pos(0)..pos(10), 10).unwrap();
        m.insert(pos(10)..pos(20), 20).unwrap();
        m.insert(pos(20)..pos(30), 30).unwrap();

        for (_, v) in m.iter_range_mut(&(pos(5)..pos(25))) {
            *v *= 2;
        }
        let vals: Vec<_> = m.iter_range(&(pos(0)..pos(80))).map(|(_, &v)| v).collect();
        assert_eq!(vals, vec![20, 40, 60]);
        assert!(m.is_valid());
    }

    #[test]
    fn get_mut() {
        let mut m: IntervalMap<Pos, String> = IntervalMap::new();
        m.insert(pos(0)..pos(10), "hello".into()).unwrap();
        let (_, v) = m.get_mut(pos(5)).unwrap();
        v.push_str(" world");
        assert_eq!(m.get(pos(5)).unwrap().1, "hello world");
        assert!(m.is_valid());
    }

    #[test]
    fn is_valid_detects_violations() {
        // Test the is_valid method can detect violations by manually constructing invalid states
        use std::collections::BTreeMap;

        // Test 1: Empty interval (start >= end) - this should be invalid
        let mut invalid_map = IntervalMap {
            inner: BTreeMap::new(),
        };
        invalid_map.inner.insert(pos(5), (pos(5), 42)); // Empty interval [5, 5)
        assert!(!invalid_map.is_valid());

        // Test 2: Overlapping intervals - this should be invalid
        let mut overlapping_map = IntervalMap {
            inner: BTreeMap::new(),
        };
        overlapping_map.inner.insert(pos(0), (pos(10), 1));
        overlapping_map.inner.insert(pos(5), (pos(15), 2)); // Overlaps with [0,10)
        assert!(!overlapping_map.is_valid());

        // Test 3: Valid state should pass
        let mut valid_map = IntervalMap {
            inner: BTreeMap::new(),
        };
        valid_map.inner.insert(pos(0), (pos(10), 1));
        valid_map.inner.insert(pos(10), (pos(20), 2)); // Adjacent, non-overlapping
        assert!(valid_map.is_valid());
    }

    #[test]
    fn random_mutation_test() {
        use rand::rngs::SmallRng;
        use rand::{Rng, RngExt, SeedableRng};
        use std::time::{Duration, Instant};

        const N: usize = 2_000_000; // Number of mutations to perform
        const SEED: u64 = 12345; // Fixed seed for reproducible testing

        let mut rng = SmallRng::seed_from_u64(SEED);
        let mut map: IntervalMap<Pos, i32> = IntervalMap::new();

        // Operation counters
        let mut insert_count = 0u64;
        let mut split_count = 0u64;
        let mut merge_count = 0u64;
        let mut delete_count = 0u64;

        // Timing accumulators
        let mut insert_time = Duration::ZERO;
        let mut split_time = Duration::ZERO;
        let mut merge_time = Duration::ZERO;
        let mut delete_time = Duration::ZERO;

        // Start with some initial random intervals
        for i in 0..10 {
            let start = rng.random_range(0..100);
            let end = rng.random_range(start + 1..start + 20);
            let value = i as i32;
            let _ = map.insert(pos(start)..pos(end), value); // May fail due to overlap, that's ok
        }

        let test_start = Instant::now();

        // Perform N random mutations
        for iteration in 0..N {
            assert!(
                map.is_valid(),
                "Invariant violated at iteration {}",
                iteration
            );

            if map.is_empty() {
                // If empty, just insert a random interval
                let start = rng.random_range(0..100);
                let end = rng.random_range(start + 1..start + 20);

                let op_start = Instant::now();
                let _ = map.insert(pos(start)..pos(end), iteration as i32);
                insert_time += op_start.elapsed();
                insert_count += 1;
                continue;
            }

            // Choose a random operation: 0=insert, 1=split, 2=merge, 3=delete
            let operation = rng.random_range(0..4);
            let infinity = pos(-1000)..pos(1000);

            match operation {
                0 => {
                    // Insert
                    let start = rng.random_range(0..200);
                    let end = rng.random_range(start + 1..start + 20);

                    let op_start = Instant::now();
                    let _ = map.insert(pos(start)..pos(end), iteration as i32);
                    insert_time += op_start.elapsed();
                    insert_count += 1;
                }
                1 => {
                    // Split
                    let intervals: Vec<_> =
                        map.iter_range(&infinity).map(|(range, _)| range).collect();
                    if !intervals.is_empty() {
                        let idx = rng.random_range(0..intervals.len());
                        let range = intervals[idx].clone();
                        if range.start.0 + 1 < range.end.0 {
                            let split_point = rng.random_range(range.start.0 + 1..range.end.0);

                            let op_start = Instant::now();
                            let _ = map.split(pos(split_point), |orig| (orig, orig + 1000));
                            split_time += op_start.elapsed();
                            split_count += 1;
                        }
                    }
                }
                2 => {
                    // Merge
                    let intervals: Vec<_> =
                        map.iter_range(&infinity).map(|(range, _)| range).collect();
                    for i in 0..intervals.len().saturating_sub(1) {
                        if intervals[i].end == intervals[i + 1].start {
                            // Found adjacent intervals, try to merge
                            if let Some((_, v1)) = map.get_mut(pos(intervals[i].start.0 + 1)) {
                                *v1 = 999; // Set a common value
                            }
                            if let Some((_, v2)) = map.get_mut(pos(intervals[i + 1].start.0 + 1)) {
                                *v2 = 999; // Set the same common value
                            }

                            let op_start = Instant::now();
                            let result = map.try_merge_at(intervals[i].end);
                            merge_time += op_start.elapsed();
                            if result.is_ok() {
                                merge_count += 1;
                            }
                            break; // Only try one merge per iteration
                        }
                    }
                }
                3 => {
                    // Delete
                    let intervals: Vec<_> =
                        map.iter_range(&infinity).map(|(range, _)| range).collect();
                    if !intervals.is_empty() {
                        let idx = rng.random_range(0..intervals.len());
                        let range = intervals[idx].clone();

                        let op_start = Instant::now();
                        let _ = map.remove_exact(range.start);
                        delete_time += op_start.elapsed();
                        delete_count += 1;
                    }
                }
                _ => unreachable!(),
            }

            // Validate after each operation
            assert!(
                map.is_valid(),
                "Invariant violated after operation {} at iteration {}",
                operation,
                iteration
            );
        }

        let total_test_time = test_start.elapsed();

        // Calculate and display statistics
        let total_ops = insert_count + split_count + merge_count + delete_count;

        println!("=== Random Mutation Test Results ===");
        println!("Total iterations: {}", N);
        println!("Total operations executed: {}", total_ops);
        println!("Final map size: {}", map.len());
        println!("Total test time: {:.2?}", total_test_time);
        println!();

        println!("Operation Counts:");
        println!(
            " Insert: {} ({:.1}%)",
            insert_count,
            100.0 * insert_count as f64 / total_ops as f64
        );
        println!(
            " Split: {} ({:.1}%)",
            split_count,
            100.0 * split_count as f64 / total_ops as f64
        );
        println!(
            " Merge: {} ({:.1}%)",
            merge_count,
            100.0 * merge_count as f64 / total_ops as f64
        );
        println!(
            " Delete: {} ({:.1}%)",
            delete_count,
            100.0 * delete_count as f64 / total_ops as f64
        );
        println!();

        println!("Average Time Per Operation:");
        if insert_count > 0 {
            println!(
                " Insert: {:.1} ns",
                insert_time.as_nanos() as f64 / insert_count as f64
            );
        }
        if split_count > 0 {
            println!(
                " Split: {:.1} ns",
                split_time.as_nanos() as f64 / split_count as f64
            );
        }
        if merge_count > 0 {
            println!(
                " Merge: {:.1} ns",
                merge_time.as_nanos() as f64 / merge_count as f64
            );
        } else {
            println!(" Merge: N/A (no successful merges)");
        }
        if delete_count > 0 {
            println!(
                " Delete: {:.1} ns",
                delete_time.as_nanos() as f64 / delete_count as f64
            );
        }
    }
}
