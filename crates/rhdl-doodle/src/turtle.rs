use egui::Pos2;

/// A TURTLE graphics model for stateful drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Default)]
struct DrawState {
    pos: Pos2,
}

#[derive(Debug, Clone, Copy)]
pub enum Mark {
    Line {
        from: Pos2,
        to: Pos2,
        stroke: egui::Stroke,
    },
    Circle {
        center: Pos2,
        radius: f32,
        fill: egui::Color32,
    },
}

#[derive(Debug, Clone, Default)]
pub struct Turtle {
    marks: Vec<Mark>,
    state: DrawState,
    stack: Vec<DrawState>,
}

pub fn draw(marks: &[Mark], painter: &egui::Painter) {
    for mark in marks {
        match mark {
            Mark::Line { from, to, stroke } => {
                painter.line_segment([*from, *to], *stroke);
            }
            Mark::Circle {
                center,
                radius,
                fill,
            } => {
                painter.circle_filled(*center, *radius, *fill);
            }
        }
    }
}

impl Turtle {
    pub fn line_to(&mut self, pos: Pos2, stroke: egui::Stroke) {
        self.marks.push(Mark::Line {
            from: self.state.pos,
            to: pos,
            stroke,
        });
        self.state.pos = pos;
    }
    pub fn move_to(&mut self, pos: Pos2) {
        self.state.pos = pos;
    }
    pub fn circle(&mut self, radius: f32, fill: egui::Color32) {
        self.marks.push(Mark::Circle {
            center: self.state.pos,
            radius,
            fill,
        });
    }
    pub fn push(&mut self) {
        self.stack.push(self.state.clone());
    }
    pub fn pop(&mut self) {
        if let Some(state) = self.stack.pop() {
            self.state = state;
        }
    }
    pub fn compile(self) -> Vec<Mark> {
        self.marks
    }
}
