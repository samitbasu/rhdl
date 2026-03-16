use egui::Pos2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContactKind {
    Start(usize),
    Middle(usize, Pos2),
    End(usize),
}

#[derive(Clone, Copy, Debug)]
pub struct Contact {
    pub kind: ContactKind,
    pub distance: f32,
    pub location: Pos2,
}

// Find the closest point to the line segment along with the distance to it.
pub fn minimum_distance_and_location_on_segment(
    ndx: usize,
    start: Pos2,
    end: Pos2,
    target: Pos2,
    hit_dist: f32,
) -> Contact {
    let line_vec = end - start;
    let line_len = line_vec.length();
    if line_len == 0.0 {
        return Contact {
            kind: ContactKind::Start(ndx),
            distance: start.distance(target),
            location: start,
        };
    }
    let line_unit_vec = line_vec / line_len;
    let projection = (target - start).dot(line_unit_vec);
    if projection < hit_dist * 2.0 {
        Contact {
            kind: ContactKind::Start(ndx),
            distance: start.distance(target),
            location: start,
        }
    } else if projection > line_len - hit_dist * 2.0 {
        Contact {
            kind: ContactKind::End(ndx + 1),
            distance: end.distance(target),
            location: end,
        }
    } else {
        let closest_point = start + line_unit_vec * projection;
        Contact {
            kind: ContactKind::Middle(ndx, closest_point),
            distance: closest_point.distance(target),
            location: closest_point,
        }
    }
}

// Find the closest point to the polyline along with the distance to it.
pub fn minimum_distance_and_location(points: &[Pos2], target: Pos2) -> Contact {
    points
        .windows(2)
        .enumerate()
        .map(|(i, w)| minimum_distance_and_location_on_segment(i, w[0], w[1], target, 5.0))
        .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
        .unwrap()
}
