use bondrewd::{BitfieldEnum, Bitfields};
use embedded_devices_derive::device_register;
use embedded_registers::register;

/// Gain selection for the ADC.
#[derive(BitfieldEnum, Copy, Clone, PartialEq, Eq, Debug, defmt::Format)]
#[bondrewd_enum(u8)]
pub enum AlsGain {
    /// x1 gain
    X_1 = 0b00,
    /// x2 gain
    X_2 = 0b01,
    /// x(1/8) gain
    X_1_8 = 0b10,
    /// x(1/4) gain
    X_1_4 = 0b11,
}

/// Integration time for the ADC.
#[derive(BitfieldEnum, Copy, Clone, PartialEq, Eq, Debug, defmt::Format)]
#[bondrewd_enum(u8)]
pub enum AlsIntegrationTime {
    /// 25 ms
    T_25 = 0b1100,
    /// 50 ms
    T_50 = 0b1000,
    /// 100 ms
    T_100 = 0b0000,
    /// 200 ms
    T_200 = 0b0001,
    /// 400 ms
    T_400 = 0b0010,
    /// 800 ms
    T_800 = 0b0011,
}

/// Persistence protection number.
#[derive(BitfieldEnum, Copy, Clone, PartialEq, Eq, Debug, defmt::Format)]
#[bondrewd_enum(u8)]
pub enum AlsPersistence {
    One = 0b00,
    Two = 0b01,
    Four = 0b10,
    Eight = 0b11,
}
