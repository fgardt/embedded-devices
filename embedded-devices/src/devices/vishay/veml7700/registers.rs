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

// todo: better name, this controls how many consecutive faults must occur to trigger an interrupt
/// Persistence protection number.
#[derive(BitfieldEnum, Copy, Clone, PartialEq, Eq, Debug, defmt::Format)]
#[bondrewd_enum(u8)]
pub enum AlsPersistence {
    One = 0b00,
    Two = 0b01,
    Four = 0b10,
    Eight = 0b11,
}

#[device_register(super::VEML7700)]
#[register(address = 0x00, mode = "rw")]
#[bondrewd(read_from = "msb0", default_endianness = "be", enforce_bytes = 2)]
pub struct Command {
    #[bondrewd(bit_length = 3, reserve)]
    #[allow(dead_code)]
    pub reserved_1: u8,
    /// ALS gain selection.
    #[bondrewd(bit_length = 2)]
    #[register]
    pub als_gain: AlsGain,
    #[bondrewd(bit_length = 1, reserve)]
    #[allow(dead_code)]
    pub reserved_2: u8,
    /// ALS integration time setting.
    #[bondrewd(bit_length = 4)]
    #[register(default = AlsIntegrationTime::T_100)]
    pub als_integration_time: AlsIntegrationTime,
    /// ALS persistence protect number setting.
    #[bondrewd(bit_length = 2, reserve)]
    #[register(default = AlsPersistence::One)]
    pub als_persistence: AlsPersistence,
    #[bondrewd(bit_length = 2, reserve)]
    #[allow(dead_code)]
    pub reserved_3: u8,
    /// ALS interrupt enable setting.
    #[register]
    pub als_interrupt: bool,
    /// ALS shut down setting.
    #[register]
    pub als_shutdown: bool,
}

#[device_register(super::VEML7700)]
#[register(address = 0x01, mode = "rw")]
#[bondrewd(read_from = "msb0", default_endianness = "be", enforce_bytes = 2)]
pub struct ThresholdWindowHigh {
    /// ALS high threshold window setting.
    #[register]
    pub raw_threshold: u16,
}

#[device_register(super::VEML7700)]
#[register(address = 0x02, mode = "rw")]
#[bondrewd(read_from = "msb0", default_endianness = "be", enforce_bytes = 2)]
pub struct ThresholdWindowLow {
    /// ALS low threshold window setting.
    #[register]
    pub raw_threshold: u16,
}

/// Power saving mode.
#[derive(BitfieldEnum, Copy, Clone, PartialEq, Eq, Debug, defmt::Format)]
#[bondrewd_enum(u8)]
pub enum PowerSavingMode {
    Mode1 = 0b00,
    Mode2 = 0b01,
    Mode3 = 0b10,
    Mode4 = 0b11,
}

#[device_register(super::VEML7700)]
#[register(address = 0x03, mode = "rw")]
#[bondrewd(read_from = "msb0", default_endianness = "be", enforce_bytes = 2)]
pub struct PowerSaving {
    #[bondrewd(bit_length = 13, reserve)]
    reserved: u16,
    /// Power saving mode setting.
    #[bondrewd(bit_length = 2)]
    #[register(default = PowerSavingMode::Mode1)]
    pub mode: PowerSavingMode,
    /// Power saving mode enable setting.
    #[register]
    pub enable: bool,
}

#[device_register(super::VEML7700)]
#[register(address = 0x04, mode = "r")]
#[bondrewd(read_from = "msb0", default_endianness = "be", enforce_bytes = 2)]
pub struct AlsData {
    /// ALS high resolution output data.
    #[register]
    pub raw_data: u16,
}

#[device_register(super::VEML7700)]
#[register(address = 0x05, mode = "r")]
#[bondrewd(read_from = "msb0", default_endianness = "be", enforce_bytes = 2)]
pub struct WhiteData {
    /// White channel output data.
    #[register]
    pub raw_data: u16,
}

#[device_register(super::VEML7700)]
#[register(address = 0x06, mode = "r")]
#[bondrewd(read_from = "msb0", default_endianness = "be", enforce_bytes = 2)]
pub struct InterruptStatus {
    /// Indicates if the ALS low threshold has been exceeded.
    #[register]
    pub exceeded_low_threshold: bool,
    /// Indicates if the ALS high threshold has been exceeded.
    #[register]
    pub exceeded_high_threshold: bool,
    #[bondrewd(bit_length = 14, reserve)]
    #[allow(dead_code)]
    pub reserved: u16,
}
