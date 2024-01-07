use crate::{bga_pin, BGAPin, BGARow};

pub const LED_ARRAY_LOCATIONS: [BGAPin; 8] = [
    bga_pin(BGARow::J, 11),
    bga_pin(BGARow::K, 11),
    bga_pin(BGARow::K, 12),
    bga_pin(BGARow::K, 14),
    bga_pin(BGARow::L, 12),
    bga_pin(BGARow::L, 14),
    bga_pin(BGARow::M, 12),
    bga_pin(BGARow::N, 14),
];

pub const BASE_CLOCK_100MHZ_LOCATION: BGAPin = bga_pin(BGARow::P, 7);
