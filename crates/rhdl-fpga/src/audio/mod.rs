#![warn(missing_docs)]
//! Audio output and audio-protocol widgets.
//!
//! Currently houses just the PWM / sigma-delta stereo audio
//! generator ([`audio_pwm::AudioPwm`]).  Future inhabitants of
//! this category include I²S transmit / receive, S/PDIF encoder /
//! decoder, AC'97, audio resamplers, simple linear-PCM tone
//! generators, and the audio islands of HDMI / DisplayPort.
//!
//! MIDI lives in `serial_bus::midi` rather than here — its wire
//! layer is essentially UART at 31250 baud and the structural
//! shape is closer to the protocol-PHY family than to the audio
//! family.  When MIDI grows a synth / sequencer companion, that
//! companion belongs here.
pub mod audio_pwm;
