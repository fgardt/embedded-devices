//! # VEML7700
//!
//! VEML7700 is a high accuracy ambient light digital 16-bit resolution sensor in a miniature
//! transparent 6.8 mm x 2.35 mm x 3.0 mm package. It includes a high sensitive photo diode, a low
//! noise amplifier, a 16-bit A/D converter and supports an easy to use I2C bus communication interface.
//! The ambient light result is as digital value available.
//!

use embedded_devices_derive::{device, device_impl};
// use uom::si::illuminance::lux; // requires uom 0.38.0

pub mod address;
pub mod registers;

/// The INA228 is an ultra-precise digital power monitor with a 20-bit delta-sigma ADC specifically
/// designed for current-sensing applications. The device can measure a full-scale differential
/// input of ±163.84 mV or ±40.96 mV across a resistive shunt sense element with common-mode
/// voltage support from –0.3 V to +85 V.
///
/// For a full description and usage examples, refer to the [module documentation](self).
#[device]
#[maybe_async_cfg::maybe(
    idents(hal(sync = "embedded_hal", async = "embedded_hal_async"), RegisterInterface),
    sync(feature = "sync"),
    async(feature = "async")
)]
pub struct INA228<I: embedded_registers::RegisterInterface> {
    /// The interface to communicate with the device
    interface: I,
    /// Shunt resistance
    shunt_resistance: ElectricalResistance,
    /// Maximum expected current
    max_expected_current: ElectricCurrent,
    /// Configured nA/LSB for current readings
    pub current_lsb_na: i64,
    /// The configured adc range
    pub adc_range: self::registers::AdcRange,
}
