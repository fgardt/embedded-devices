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

type VEML7700I2cCodec = embedded_registers::i2c::codecs::OneByteRegAddrCodec;

#[device]
#[maybe_async_cfg::maybe(
    idents(hal(sync = "embedded_hal", async = "embedded_hal_async"), RegisterInterface),
    sync(feature = "sync"),
    async(feature = "async")
)]
pub struct INA226<I: embedded_registers::RegisterInterface> {
    /// The interface to communicate with the device
    interface: I,
}

#[maybe_async_cfg::maybe(
    idents(hal(sync = "embedded_hal", async = "embedded_hal_async"), I2cDevice),
    sync(feature = "sync"),
    async(feature = "async")
)]
impl<I> INA226<embedded_registers::i2c::I2cDevice<I, hal::i2c::SevenBitAddress, VEML7700I2cCodec>>
where
    I: hal::i2c::I2c<hal::i2c::SevenBitAddress> + hal::i2c::ErrorType,
{
    /// Initializes a new device with the given address on the specified bus.
    /// This consumes the I2C bus `I`.
    ///
    /// Before using this device, you should call the [`Self::init`] method.
    #[inline]
    pub fn new_i2c(interface: I, address: Address) -> Self {
        Self {
            interface: embedded_registers::i2c::I2cDevice::new(interface, address.into()),
        }
    }
}
