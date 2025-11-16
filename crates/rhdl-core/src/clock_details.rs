#[derive(Debug, Clone)]
pub struct ClockDetails {
    pub name: String,
    pub period_in_fs: u64,
    pub offset_in_fs: u64,
    pub initial_state: bool,
}

impl ClockDetails {
    pub fn new(name: &str, period_in_fs: u64, offset_in_fs: u64, initial_state: bool) -> Self {
        Self {
            name: name.to_string(),
            period_in_fs,
            offset_in_fs,
            initial_state,
        }
    }
    pub fn pos_edge_at(&self, time: u64) -> bool {
        if time < self.offset_in_fs {
            return false;
        }
        let time = time - self.offset_in_fs;
        let period = self.period_in_fs;
        time.is_multiple_of(period)
    }
    pub fn neg_edge_at(&self, time: u64) -> bool {
        if time < self.offset_in_fs {
            return false;
        }
        let time = time - self.offset_in_fs;
        let period = self.period_in_fs;
        time % period == period / 2
    }
    pub fn next_edge_after(&self, time: u64) -> u64 {
        if time < self.offset_in_fs {
            return self.offset_in_fs;
        }
        let time = time - self.offset_in_fs;
        let period = self.period_in_fs / 2;
        (time / period + 1) * period + self.offset_in_fs
    }
}

#[test]
fn test_clock_details() {
    let clock = ClockDetails::new("clk", 10, 0, false);
    assert!(clock.pos_edge_at(0));
    assert!(!clock.pos_edge_at(5));
    assert!(clock.pos_edge_at(10));
    assert!(!clock.pos_edge_at(15));
    assert!(!clock.neg_edge_at(0));
    assert!(clock.neg_edge_at(5));
    assert!(!clock.neg_edge_at(10));
    assert_eq!(clock.next_edge_after(1), 5);
    assert_eq!(clock.next_edge_after(0), 5);
    assert_eq!(clock.next_edge_after(5), 10);
}
