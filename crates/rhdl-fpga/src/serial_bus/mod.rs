#![warn(missing_docs)]
//! Serial-bus and protocol-PHY widgets.
//!
//! Each widget in this category drives or receives data over a
//! serial wire protocol — either a true bus (UART, SPI, I²C, CAN,
//! LIN) or a single-wire sensor / device protocol (1-Wire, DHT22,
//! IR-remote, SENT, MIDI, WS2812).  The shared shape is "produce
//! a digital pin signal that, when fed into the right physical
//! transceiver, communicates with off-chip silicon."
//!
//! Most widgets here pair with an external level-shifter,
//! transceiver, or driver IC — for example, a CAN widget needs an
//! external TJA1050; a 1-Wire widget needs an external pull-up
//! resistor and the host wraps the open-drain output with
//! `tristate::simple` at the pad.  The `core::tristate` family is
//! the canonical way to expose a true bidirectional bus to the
//! I/O pads.
pub mod can_master;
pub mod dht22;
pub mod epaper_ssd16xx;
pub mod half_spi_master;
pub mod hd44780;
pub mod i2c_master;
pub mod ir_nec_rx;
pub mod lin_master;
pub mod mfm_encoder;
pub mod midi;
pub mod mipi_dbi_type_b;
pub mod mipi_dbi_type_c;
pub mod one_wire_master;
pub mod rs485_master;
pub mod sent_rx;
pub mod smbus_host;
pub mod smpte_ltc_encoder;
pub mod spi_master;
pub mod spi_slave;
pub mod ti_hdq;
pub mod uart;
pub mod uart_16550;
pub mod uart_rx;
pub mod uart_tx;
pub mod ws2812;
