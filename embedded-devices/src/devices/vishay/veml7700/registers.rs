use bondrewd::BitfieldEnum;
use embedded_devices_derive::device_register;
use embedded_registers::register;

/// Gain selection for the ADC.
#[derive(BitfieldEnum, Copy, Clone, PartialEq, Eq, Debug, defmt::Format)]
#[bondrewd_enum(u8)]
#[allow(non_camel_case_types)]
pub enum Gain {
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
#[allow(non_camel_case_types)]
pub enum IntegrationTime {
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

impl IntegrationTime {
    pub fn ms(&self) -> u32 {
        match self {
            Self::T_25 => 25,
            Self::T_50 => 50,
            Self::T_100 => 100,
            Self::T_200 => 200,
            Self::T_400 => 400,
            Self::T_800 => 800,
        }
    }
}

/// Persistence protection number.
#[derive(BitfieldEnum, Copy, Clone, PartialEq, Eq, Debug, defmt::Format)]
#[bondrewd_enum(u8)]
pub enum InterruptThresholdCount {
    One = 0b00,
    Two = 0b01,
    Four = 0b10,
    Eight = 0b11,
}

#[device_register(super::VEML7700)]
#[register(address = 0x00, mode = "rw")]
#[bondrewd(read_from = "msb0", default_endianness = "be", enforce_bytes = 2)]
pub struct Config {
    #[bondrewd(bit_length = 3, reserve)]
    #[allow(dead_code)]
    pub reserved_1: u8,
    /// ALS gain configuration.
    #[bondrewd(enum_primitive = "u8", bit_length = 2)]
    #[register(default = Gain::X_1_8)]
    pub gain: Gain,
    #[bondrewd(bit_length = 1, reserve)]
    #[allow(dead_code)]
    pub reserved_2: u8,
    /// ALS integration time configuration.
    #[bondrewd(enum_primitive = "u8", bit_length = 4)]
    #[register(default = IntegrationTime::T_100)]
    pub integration_time: IntegrationTime,
    /// Interrupt .
    #[bondrewd(enum_primitive = "u8", bit_length = 2)]
    #[register(default = InterruptThresholdCount::One)]
    pub interrupt_threshold_count: InterruptThresholdCount,
    #[bondrewd(bit_length = 2, reserve)]
    #[allow(dead_code)]
    pub reserved_3: u8,
    /// Interrupt enable setting.
    #[register]
    pub interrupt_enable: bool,
    /// Shutdown setting.
    #[register]
    pub shutdown: bool,
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
    #[allow(dead_code)]
    pub reserved: u16,
    /// Power saving mode setting.
    #[bondrewd(enum_primitive = "u8", bit_length = 2)]
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

#[derive(BitfieldEnum, Copy, Clone, PartialEq, Eq, Debug, defmt::Format)]
#[bondrewd_enum(u8)]
pub enum AddressOption {
    // datasheet mentions 0x20 / 0x90 but those are 8 bit addresses (rw bit 0)
    // so converted to 7 bit: 0x20 -> 0x10, 0x90 -> 0x48
    X10 = 0xC4,
    X48 = 0xC8,
}

#[device_register(super::VEML7700)]
#[register(address = 0x07, mode = "r")]
#[bondrewd(read_from = "msb0", default_endianness = "be", enforce_bytes = 2)]
pub struct DeviceId {
    #[bondrewd(enum_primitive = "u8", bit_length = 8)]
    #[register(default = AddressOption::X10)]
    pub slave_address: AddressOption,
    /// Device ID.
    #[register(default = 0x81)]
    pub id: u8,
}
