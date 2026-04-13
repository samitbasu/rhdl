use egui::{Pos2, Vec2};

use crate::{
    geometry::{Contact, ContactKind},
    grid::snap_to_grip,
    rectbox::LineAnchor,
};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
pub struct LineId(usize);

impl LineId {
    pub fn next(self) -> Self {
        LineId(self.0 + 1)
    }
}

pub struct PolyLine {
    pub start: LineAnchor,
    pub points: Vec<Pos2>,
    pub end: LineAnchor,
    pub id: LineId,
    pub editing: Option<usize>,
}

impl PolyLine {
    pub fn id(&self) -> LineId {
        self.id
    }
    pub fn start_edit(&mut self, contact: Contact) {
        match contact.kind {
            ContactKind::Start(ndx) => {
                if ndx != 0 {
                    self.editing = Some(ndx - 1);
                }
            }
            ContactKind::Middle(ndx, pos) => {
                self.points.insert(ndx, pos);
                self.editing = Some(ndx);
            }
            ContactKind::End(ndx) => {
                if ndx != self.points.len() + 1 {
                    self.editing = Some(ndx - 1);
                }
            }
        }
    }
    pub fn shift_edit_point(&mut self, delta: Vec2) {
        if let Some(ndx) = self.editing {
            self.points[ndx] += delta;
        }
    }
    pub fn edit_point(&self) -> Option<Pos2> {
        self.editing.map(|ndx| self.points[ndx])
    }
    pub fn finish_edit(&mut self) {
        self.points.iter_mut().for_each(|p| *p = snap_to_grip(*p));
        // Remove duplicate points
        self.points.dedup();
        // Remove points that are on the same horizontal or vertical
        // line as their neighbors, since they don't affect the shape of the line.
        let drop_flag = self
            .points
            .windows(3)
            .map(|w| {
                (w[0].x == w[1].x && w[1].x == w[2].x) || (w[0].y == w[1].y && w[1].y == w[2].y)
            })
            .collect::<Vec<_>>();
        // Drop the points based on drop_flag.  Drop_flag[0] corresponds to points[1], drop_flag[1] corresponds to points[2], etc.
        let points = std::mem::take(&mut self.points);
        let point_len = points.len();
        self.points = points
            .into_iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if i == 0 || i == point_len - 1 {
                    Some(p)
                } else if drop_flag[i - 1] {
                    None
                } else {
                    Some(p)
                }
            })
            .collect();
        self.editing = None;
    }
}
