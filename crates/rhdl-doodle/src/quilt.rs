use crate::intmap::IntervalMap;
use anyhow::Result;
use std::collections::BTreeMap;
use std::ops::Range;

// Nomenclature from Miriyala, Hornick and Tamassia
// "An Incremental Approach to Aesthetic Graph Layout"
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
enum SegmentKind {
    Virtual,
    Edge,
    Vertex,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CoordX(i32);

impl std::fmt::Debug for CoordX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_x", self.0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CoordY(i32);

impl std::fmt::Debug for CoordY {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_y", self.0)
    }
}

fn coord_x(x: i32) -> CoordX {
    CoordX(x)
}

fn coord_y(y: i32) -> CoordY {
    CoordY(y)
}

struct Point {
    x: CoordX,
    y: CoordY,
}

fn point(x: CoordX, y: CoordY) -> Point {
    Point { x, y }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
struct Rectangle {
    x_range: Range<CoordX>,
    y_range: Range<CoordY>,
}

impl Rectangle {
    fn intersects(&self, other: &Rectangle) -> bool {
        self.x_range.start < other.x_range.end
            && self.x_range.end > other.x_range.start
            && self.y_range.start < other.y_range.end
            && self.y_range.end > other.y_range.start
    }
    fn intersect(&self, other: &Rectangle) -> Rectangle {
        let x_start = self.x_range.start.max(other.x_range.start);
        let x_end = self.x_range.end.min(other.x_range.end);
        let y_start = self.y_range.start.max(other.y_range.start);
        let y_end = self.y_range.end.min(other.y_range.end);
        rectangle(x_start..x_end, y_start..y_end)
    }
    fn contains_y(&self, y: CoordY) -> bool {
        self.y_range.contains(&y)
    }
    fn contains_x(&self, x: CoordX) -> bool {
        self.x_range.contains(&x)
    }
    fn contains(&self, point: Point) -> bool {
        self.contains_x(point.x) && self.contains_y(point.y)
    }
    fn start_x(&self) -> CoordX {
        self.x_range.start
    }
    fn end_x(&self) -> CoordX {
        self.x_range.end
    }
    fn start_y(&self) -> CoordY {
        self.y_range.start
    }
    fn end_y(&self) -> CoordY {
        self.y_range.end
    }
    fn x_range(&self) -> Range<CoordX> {
        self.x_range.clone()
    }
    fn y_range(&self) -> Range<CoordY> {
        self.y_range.clone()
    }
}

const fn rectangle(x_range: Range<CoordX>, y_range: Range<CoordY>) -> Rectangle {
    Rectangle { x_range, y_range }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
struct RegionId(usize);

impl RegionId {
    fn next(self) -> Self {
        RegionId(self.0 + 1)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
struct Segment {
    kind: SegmentKind,
    prev: RegionId,
    next: RegionId,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
enum RegionKind {
    Node,
    Space,
}

#[derive(Clone, PartialEq, Debug, Hash)]
struct Region {
    rect: Rectangle,
    kind: RegionKind,
}

impl Region {
    fn rect(&self) -> &Rectangle {
        &self.rect
    }
    fn kind(&self) -> RegionKind {
        self.kind
    }
}

const fn region(rect: Rectangle, kind: RegionKind) -> Region {
    Region { rect, kind }
}

const CANVAS_ID: RegionId = RegionId(0);

#[derive(Clone)]
pub struct Quilt {
    vsegs: BTreeMap<CoordX, IntervalMap<CoordY, Segment>>,
    hsegs: BTreeMap<CoordY, IntervalMap<CoordX, Segment>>,
    regions: BTreeMap<RegionId, Region>,
    next_id: RegionId,
}

impl std::fmt::Display for Quilt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, region) in &self.regions {
            writeln!(f, "Region {:?}: {:?}", id, region)?;
        }
        for (x, vsegs) in &self.vsegs {
            for (y_range, seg) in vsegs.iter_range(&(coord_y(-100000)..coord_y(100000))) {
                writeln!(
                    f,
                    "Vertical Segment at x={:?} from y={:?}..{:?}: {:?}",
                    x, y_range.start, y_range.end, seg
                )?;
            }
        }
        for (y, hsegs) in &self.hsegs {
            for (x_range, seg) in hsegs.iter_range(&(coord_x(-100000)..coord_x(100000))) {
                writeln!(
                    f,
                    "Horizontal Segment at y={:?} from x={:?}..{:?}: {:?}",
                    y, x_range.start, x_range.end, seg
                )?;
            }
        }
        Ok(())
    }
}

impl Default for Quilt {
    fn default() -> Self {
        let mut regions = BTreeMap::new();
        let canvas = region(
            rectangle(coord_x(-1000)..coord_x(1000), coord_y(-1000)..coord_y(1000)),
            RegionKind::Space,
        );
        regions.insert(CANVAS_ID, canvas);
        let mut vsegs = BTreeMap::new();
        let mut vline = IntervalMap::new();
        vline
            .insert(
                coord_y(-1000)..coord_y(1000),
                Segment {
                    kind: SegmentKind::Virtual,
                    prev: CANVAS_ID,
                    next: CANVAS_ID,
                },
            )
            .unwrap();
        vsegs.insert(coord_x(-1000), vline.clone());
        vsegs.insert(coord_x(1000), vline.clone());
        let mut hsegs = BTreeMap::new();
        let mut hline = IntervalMap::new();
        hline
            .insert(
                coord_x(-1000)..coord_x(1000),
                Segment {
                    kind: SegmentKind::Virtual,
                    prev: CANVAS_ID,
                    next: CANVAS_ID,
                },
            )
            .unwrap();
        hsegs.insert(coord_y(-1000), hline.clone());
        hsegs.insert(coord_y(1000), hline.clone());
        Self {
            vsegs,
            hsegs,
            regions,
            next_id: RegionId(1),
        }
    }
}

impl Quilt {
    // Allocate a new region
    fn alloc_region(&mut self, x_range: &Range<CoordX>, y_range: &Range<CoordY>) -> RegionId {
        let new_id = self.next_id;
        self.next_id = self.next_id.next();
        self.regions.insert(
            new_id,
            region(
                rectangle(x_range.clone(), y_range.clone()),
                RegionKind::Space,
            ),
        );
        new_id
    }
    // Forget a region
    fn free_region(&mut self, region_id: RegionId) {
        self.regions.remove(&region_id);
    }
    // Split a region by the given rectangle, returning the ID of the new region.
    // If the region is contained within the given rectangle, then nothing happens
    // and the original region ID is returned.
    // If there is no overlap between the region and the rectangle, then no new region is returned.
    fn split_region_with_rectangle(
        &mut self,
        region: RegionId,
        rect: &Rectangle,
    ) -> Result<Option<RegionId>> {
        let mut base = self.regions[&region].rect().clone();
        if !base.intersects(rect) {
            return Ok(None);
        }
        // Compute the intersection of the two rectangles.
        // We may need up to 4 splits.  Each split needs to be conditioned
        // on if the coordinate lies on the boundary of the original rectangle.
        // The order matters, since we want the gaps to be vertically large.
        let mut active = region;
        let intersected = base.intersect(rect);
        if intersected.start_x() != base.start_x() {
            let (_, working_region) =
                self.split_at_x(active, intersected.start_x(), SegmentKind::Virtual)?;
            base = self.regions[&working_region].rect().clone();
            active = working_region;
        }
        let intersected = base.intersect(rect);
        if intersected.start_y() != base.start_y() {
            let (_, working_region) =
                self.split_at_y(active, intersected.start_y(), SegmentKind::Virtual)?;
            base = self.regions[&working_region].rect().clone();
            active = working_region;
        }
        let intersected = base.intersect(rect);
        if intersected.end_x() != base.end_x() {
            let (working_region, _) =
                self.split_at_x(active, intersected.end_x(), SegmentKind::Virtual)?;
            base = self.regions[&working_region].rect().clone();
            active = working_region;
        }
        let intersected = base.intersect(rect);
        if intersected.end_y() != base.end_y() {
            let (working_region, _) =
                self.split_at_y(active, intersected.end_y(), SegmentKind::Virtual)?;
            base = self.regions[&working_region].rect().clone();
            active = working_region;
        }
        debug_assert_eq!(&base, rect);
        Ok(Some(active))
    }
    // Check that a rectangle is empty, meaning that it does not contain any regions of kind Node.
    fn is_available(&self, rect: &Rectangle) -> bool {
        self.iter_overlapping(rect)
            .all(|(_id, region)| region.kind() != RegionKind::Node)
    }
    // Iterate over all regions that overlap with the given rectangle.
    fn iter_overlapping(&self, rect: &Rectangle) -> impl Iterator<Item = (&RegionId, &Region)> {
        self.regions
            .iter()
            .filter(move |(_id, region)| region.rect().intersects(rect))
    }
    // Split a given region with a horizontal line at the given y coordinate.
    fn split_at_y(
        &mut self,
        region_id: RegionId,
        y: CoordY,
        kind: SegmentKind,
    ) -> Result<(RegionId, RegionId)> {
        let rect = self.regions[&region_id].rect().clone();
        if !rect.contains_y(y) {
            anyhow::bail!("Y coordinate is outside the bounds of the region");
        }
        let (top_y_range, bottom_y_range) = (rect.y_range.start..y, y..rect.y_range.end);
        let top_id = self.alloc_region(&rect.x_range, &top_y_range);
        let bottom_id = self.alloc_region(&rect.x_range, &bottom_y_range);
        let hseg = Segment {
            kind,
            prev: top_id,
            next: bottom_id,
        };
        // Add this horizontal segment into the hsegs map
        // There should be no overlapping segments in the same row.
        self.hsegs
            .entry(y)
            .or_default()
            .insert(rect.x_range.clone(), hseg)?;
        // Any horizontal segments on the old region boundary need to be updated to point to the new regions
        // horizontal segments on the min-y boundary need their "next" pointer updated (which is the bottom hand
        // region).
        self.hsegs
            .entry(rect.y_range.start)
            .or_default()
            .iter_range_mut(&rect.x_range)
            .for_each(|(_range, segment)| {
                if segment.next == region_id {
                    segment.next = top_id;
                }
            });
        // horizontal segments on the max-y boundary need their "prev" pointer updated (which is the top hand region).
        self.hsegs
            .entry(rect.y_range.end)
            .or_default()
            .iter_range_mut(&rect.x_range)
            .for_each(|(_range, segment)| {
                if segment.prev == region_id {
                    segment.prev = bottom_id;
                }
            });
        // Any vertical segment on the min-x boundary of the old region should:
        // 1. If it is entirely above the new cut point, it should have it's "next" pointer set to top_id
        // 2. If it is entirely below the new cut point, it should have it's "next" pointer set to bottom_id
        // 3. If it intersects with the new cut point, it should be split into two segments, with the upper segment
        // having it's "next" pointer set to top_id and the lower segment having it's "next
        // pointer set to bottom_id.
        // Skip if the region is the CANVAS
        if let Some(vseg) = self.vsegs.get_mut(&rect.x_range.start) {
            vseg.split(y, |segment| (segment, segment))?;
        }
        if let Some(vseg) = self.vsegs.get_mut(&rect.x_range.end) {
            vseg.split(y, |segment| (segment, segment))?;
        }
        if let Some(vsegs) = self.vsegs.get_mut(&rect.x_range.start) {
            vsegs
                .iter_range_mut(&top_y_range)
                .for_each(|(_range, segment)| {
                    segment.next = top_id;
                });
            vsegs
                .iter_range_mut(&bottom_y_range)
                .for_each(|(_range, segment)| {
                    segment.next = bottom_id;
                });
        }
        if let Some(vsegs) = self.vsegs.get_mut(&rect.x_range.end) {
            vsegs
                .iter_range_mut(&top_y_range)
                .for_each(|(_range, segment)| {
                    segment.prev = top_id;
                });
            vsegs
                .iter_range_mut(&bottom_y_range)
                .for_each(|(_range, segment)| {
                    segment.prev = bottom_id;
                });
        }
        self.free_region(region_id);
        Ok((top_id, bottom_id))
    }
    // Split a given region with a vertical line at the given x coordinate,
    // returning the ID of the two new regions (left and right).
    // This can fail if the provided y coordinate is outside the bounds of the region.
    fn split_at_x(
        &mut self,
        region_id: RegionId,
        x: CoordX,
        kind: SegmentKind,
    ) -> Result<(RegionId, RegionId)> {
        let rect = self.regions[&region_id].rect().clone();
        if !rect.contains_x(x) {
            anyhow::bail!("X coordinate is outside the bounds of the region");
        }
        let (left_x_range, right_x_range) = (rect.x_range.start..x, x..rect.x_range.end);
        let left_id = self.alloc_region(&left_x_range, &rect.y_range);
        let right_id = self.alloc_region(&right_x_range, &rect.y_range);
        let vseg = Segment {
            kind,
            prev: left_id,
            next: right_id,
        };
        // Add this vertical segment into the vsegs map
        // There should be no overlapping segments in the same column.
        self.vsegs
            .entry(x)
            .or_default()
            .insert(rect.y_range.clone(), vseg)?;
        // Any vertical segments on the old region boundary need to be updated to point to the new regions
        // vertical segments on the min-x boundary need their "next" pointer updated (which is the right hand
        // region).
        self.vsegs
            .entry(rect.x_range.start)
            .or_default()
            .iter_range_mut(&rect.y_range)
            .for_each(|(_range, segment)| {
                if segment.next == region_id {
                    segment.next = left_id;
                }
            });
        // vertical segments on the max-x boundary need their "prev" pointer updated (which is the left hand region).
        self.vsegs
            .entry(rect.x_range.end)
            .or_default()
            .iter_range_mut(&rect.y_range)
            .for_each(|(_range, segment)| {
                if segment.prev == region_id {
                    segment.prev = right_id;
                }
            });
        // Any horizontal segment on the min-y boundary of the old region should:
        // 1. If it is entirely to the left of the new cut point, it should have it's "next" pointer set to left_id
        // 2. If it is entirely to the right of the new cut point, it should have it's "next" pointer set to right_id
        // 3. If it intersects with the new cut point, it should be split into two segments, with the left hand segment
        // having it's "next" pointer set to left_id and the right hand segment having it's "next" pointer
        // set to right_id.
        // We first take care of splitting the two horizontal segments that contain the intersection point.
        // Skip if the region is the CANVAS
        if let Some(hseg) = self.hsegs.get_mut(&rect.y_range.start) {
            hseg.split(x, |segment| (segment, segment))?;
        }
        if let Some(hseg) = self.hsegs.get_mut(&rect.y_range.end) {
            hseg.split(x, |segment| (segment, segment))?;
        }
        // Now, all horizontal segments to the left of the cut point need to be updated.
        // If the horizontal segment is on the top edge of hte rect (y min), then the next pointer
        // needs to be updated.  If the horizontal segment is on the bottom edge of the rect (y max), then the prev pointer
        // needs to be updated.
        if let Some(hsegs) = self.hsegs.get_mut(&rect.y_range.start) {
            hsegs
                .iter_range_mut(&left_x_range)
                .for_each(|(_range, segment)| {
                    segment.next = left_id;
                });
            hsegs
                .iter_range_mut(&right_x_range)
                .for_each(|(_range, segment)| {
                    segment.next = right_id;
                });
        }
        if let Some(hsegs) = self.hsegs.get_mut(&rect.y_range.end) {
            hsegs
                .iter_range_mut(&left_x_range)
                .for_each(|(_range, segment)| {
                    segment.prev = left_id;
                });
            hsegs
                .iter_range_mut(&right_x_range)
                .for_each(|(_range, segment)| {
                    segment.prev = right_id;
                });
        }
        self.free_region(region_id);
        Ok((left_id, right_id))
    }
    fn is_valid(&self) -> bool {
        // There are several invariants to check here.
        // First, we will check that every region (except the CANVAS_ID) has vertical segments
        // on it's left and right boundaries, and that it is covered on both.
        for (id, region) in &self.regions {
            if *id == CANVAS_ID {
                continue;
            }
            let rect = region.rect();
            assert!(self.vsegs[&rect.start_x()].covered(&rect.y_range()));
            assert!(self.vsegs[&rect.end_x()].covered(&rect.y_range()));
            for (_, seg) in self.vsegs[&rect.start_x()].iter_range(&rect.y_range) {
                if seg.next != *id {
                    return false;
                }
            }
            for (_, seg) in self.vsegs[&rect.end_x()].iter_range(&rect.y_range) {
                if seg.prev != *id {
                    return false;
                }
            }
            assert!(self.hsegs[&rect.start_y()].covered(&rect.x_range()));
            assert!(self.hsegs[&rect.end_y()].covered(&rect.x_range()));
            for (_, seg) in self.hsegs[&rect.start_y()].iter_range(&rect.x_range) {
                if seg.next != *id {
                    return false;
                }
            }
            for (_, seg) in self.hsegs[&rect.end_y()].iter_range(&rect.x_range) {
                if seg.prev != *id {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use rand::random_range;

    use super::*;
    #[test]
    fn test_create_quilt() {
        let mut elapsed = std::time::Duration::default();
        (0..100).for_each(|_| {
            let mut quilt = Quilt::default();
            assert!(quilt.regions.contains_key(&CANVAS_ID));
            let rect1 = rectangle(coord_x(0)..coord_x(100), coord_y(0)..coord_y(100));
            let tic = std::time::Instant::now();
            let r1 = quilt
                .split_region_with_rectangle(CANVAS_ID, &rect1)
                .unwrap()
                .unwrap();
            let rect2 = rectangle(coord_x(10)..coord_x(20), coord_y(10)..coord_y(20));
            let r2 = quilt
                .split_region_with_rectangle(r1, &rect2)
                .unwrap()
                .unwrap();
            assert_eq!(quilt.regions[&r2].rect(), &rect2);
            let toc = std::time::Instant::now();
            elapsed += toc - tic;
        });
        println!("Time taken: {:?}", elapsed);
    }
}
