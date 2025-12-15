use embedded_interfaces::codegen::interface_objects;
use embedded_interfaces::registers::{
    i2c::codecs::OneByteRegAddrCodec, spi::codecs::unsupported_codec::UnsupportedCodec,
};

pub type BME690SpiCodec = UnsupportedCodec<()>; // TODO: implement custom paged SPI codec
pub type BME690I2cCodec = OneByteRegAddrCodec;

// SPI register addresses with page logic:
// 0x73 stays the same for both pages
// page 0: addresses are shifted by -0x80 (0xF0 -> 0x70, 0xE0 -> 0x60, 0xD0 -> 0x50)
// page 1: addresses stay identical to I2C

interface_objects! {
    register_defaults {
        codec_error = (),
        i2c_codec = BME690I2cCodec,
        spi_codec = BME690SpiCodec,
    }

    register_devices [ super::BME690 ]

    enum Chip: u8 {
        0x61 BME690,
        _ Invalid(u8),
    }

    enum ChipVariant: u8 {
        0x02 BME690,
        _ Invalid(u8),
    }

    /// Reset magic values
    enum ResetMagic: u8{8} {
        /// Magic value to reset the device
        0xB6 Reset,
        /// Invalid reset magic
        _ Invalid(u8),
    }

    /// Sensor operation mode.
    enum SensorMode: u8{2} {
        /// Sleep mode is entered by default after power on reset. In sleep mode, no measurements are
        /// performed and power consumption is at a minimum. All registers are accessible.
        /// There are no special restrictions on interface timings.
        0b00 Sleep,
        /// In forced mode a single TPHG measurement cycle is performed in accordance
        /// with the configured oversampling, filter and gas sensor settings.
        /// After the measurement is complete, the device returns to [SensorMode::Sleep].
        /// To start a new measurement cycle, [SensorMode::Forced] must be set again.
        /// The gas sensor heater will only operate during the gas measurement phase.
        0b01 Forced,
        /// Continuous TPHG measurement cycles.
        /// Will stay in this mode indefinitely.
        /// Gas sensor heater operates continuously.
        0b10 Parallel,
        /// Invalid sensor mode.
        _ Invalid(u8),
    }

    /// Oversampling settings for pressure, temperature and humidity measurements.
    #[allow(non_camel_case_types)]
    enum Oversampling: u8{3} {
        0b000 Disabled,
        0b001 X_1,
        0b010 X_2,
        0b011 X_4,
        0b100 X_8,
        0b101..=7 X_16,
    }

    /// Lowpass filter settings for pressure and temperature values.
    /// Enabling any filter option increases the resolution of the
    /// respective measured quantity to 20 bits.
    #[allow(non_camel_case_types)]
    enum IIRFilter: u8{3} {
        0b000 Disabled,
        0b001 Coefficient1,
        0b010 Coefficient3,
        0b011 Coefficient7,
        0b100 Coefficient15,
        0b101 Coefficient31,
        0b110 Coefficient63,
        0b111 Coefficient127,
    }

    enum HeaterStep: u8{4} {
        0b0000 Step0,
        0b0001 Step1,
        0b0010 Step2,
        0b0011 Step3,
        0b0100 Step4,
        0b0101 Step5,
        0b0110 Step6,
        0b0111 Step7,
        0b1000 Step8,
        0b1001 Step9,

        _ Invalid(u8),
    }

    enum HeaterProfile: u8{4} {
        0b0000 NoConversion,
        0b0001 Step0,
        0b0010 Step0To1,
        0b0011 Step0To2,
        0b0100 Step0To3,
        0b0101 Step0To4,
        0b0110 Step0To5,
        0b0111 Step0To6,
        0b1000 Step0To7,
        0b1001 Step0To8,
        _ Step0To9,
    }

    /// The chip identification number. This number can
    /// be read as soon as the device finished the power-on-reset.
    register Id(addr = 0xD0, mode = r, size = 1) {
        chip: Chip = Chip::Invalid(0),
    }

    /// The sensor variant identification number. This number
    /// can be read as soon as the device finished the power-on-reset.
    register Variant(addr = 0xF0, mode = r, size = 1) {
        variant: ChipVariant = ChipVariant::Invalid(0),
    }

    /// The reset register. If the value 0xB6 is written to the register,
    /// the device is reset using the complete power-on-reset procedure.
    /// Writing other values than 0xB6 has no effect.
    register Reset(addr = 0xE0, mode = w, size = 1) {
        magic: ResetMagic = ResetMagic::Reset,
    }

    /// Only used in SPI mode to switch between register pages.
    /// Page 0 allows access from 0x00 to 0x7F
    /// Page 1 allows access from 0x7F to 0xFF
    register SpiPage(addr = 0x73, mode = rw, size = 1) {
        _: u8{3} = 0,
        page_one: bool = false,

        // reserved bits, register defaults to 0x01
        _: u8{4} = 1,
    }

    register Config(addr = 0x75, mode = rw, size = 1) {
        _: u8{3},
        filter: IIRFilter = IIRFilter::Disabled,
        _: u8{1},
        /// Enable SPI 3-wire mode.
        spi_3w_en: bool = false,
    }

    register CtrlMeasure(addr = 0x74, mode = rw, size = 1) {
        /// Oversampling control for temperature sensor.
        osrs_t: Oversampling = Oversampling::Disabled,
        /// Oversampling control for pressure sensor.
        osrs_p: Oversampling = Oversampling::Disabled,
        mode: SensorMode = SensorMode::Sleep,
    }

    register CtrlHumid(addr = 0x72, mode = rw, size = 1) {
        _: u8{1},
        /// Enable new data interrupt in SPI 3-wire mode.
        /// New data interrupt is then indicated on the SDO pad.
        spi_3w_int_en: bool = false,
        _: u8{3},
        /// Oversampling control for humidity sensor.
        osrs_h: Oversampling = Oversampling::Disabled,
    }

    register CtrlGas0(addr = 0x70, mode = rw, size = 1) {
        _: u8{4},
        heat_off: bool = false,
        _: u8{3},
    }

    register CtrlGas1Forced(addr = 0x71, mode = rw, size = 1) {
        _: u8{2},
        run_gas: bool = false,
        _: u8{1},
        nb_conv: HeaterStep = HeaterStep::Step0,
    }

    register CtrlGas1Parallel(addr = 0x71, mode = rw, size = 1) {
        _: u8{2},
        run_gas: bool = false,
        _: u8{1},
        nb_conv: HeaterProfile = HeaterProfile::NoConversion,
    }

    register CombinedConfigForced(addr = 0x70, mode = r, size = 6) {
        ctrl_gas_0: CtrlGas0Unpacked,
        ctrl_gas_1: CtrlGas1ForcedUnpacked,
        ctrl_hum: CtrlHumidUnpacked,
        spi_page: SpiPageUnpacked = SpiPage::default().with_page_one(true).unpack(),
        ctrl_measure: CtrlMeasureUnpacked,
        config: ConfigUnpacked,
    }

    register CombinedConfigParallel(addr = 0x70, mode = r, size = 6) {
        ctrl_gas_0: CtrlGas0Unpacked,
        ctrl_gas_1: CtrlGas1ParallelUnpacked,
        ctrl_hum: CtrlHumidUnpacked,
        spi_page: SpiPageUnpacked = SpiPage::default().with_page_one(true).unpack(),
        ctrl_measure: CtrlMeasureUnpacked,
        config: ConfigUnpacked,
    }

    register GasHeaterCurrent(addr = 0x59, mode = rw, size = 10) {
        idac_heat_0: u8,
        idac_heat_1: u8,
        idac_heat_2: u8,
        idac_heat_3: u8,
        idac_heat_4: u8,
        idac_heat_5: u8,
        idac_heat_6: u8,
        idac_heat_7: u8,
        idac_heat_8: u8,
        idac_heat_9: u8,
    }

    register GasHeaterResistance(addr = 0x5A, mode = rw, size = 10) {
        res_heat_0: u8,
        res_heat_1: u8,
        res_heat_2: u8,
        res_heat_3: u8,
        res_heat_4: u8,
        res_heat_5: u8,
        res_heat_6: u8,
        res_heat_7: u8,
        res_heat_8: u8,
        res_heat_9: u8,
    }

    register GasHeaterResistance0(addr = 0x5A, mode = rw, size = 1) {
        res_heat: u8,
    }

    register GasHeaterResistance1(addr = 0x5B, mode = rw, size = 1) {
        res_heat: u8,
    }

    register GasHeaterResistance2(addr = 0x5C, mode = rw, size = 1) {
        res_heat: u8,
    }

    register GasHeaterResistance3(addr = 0x5D, mode = rw, size = 1) {
        res_heat: u8,
    }

    register GasHeaterResistance4(addr = 0x5E, mode = rw, size = 1) {
        res_heat: u8,
    }

    register GasHeaterResistance5(addr = 0x5F, mode = rw, size = 1) {
        res_heat: u8,
    }

    register GasHeaterResistance6(addr = 0x60, mode = rw, size = 1) {
        res_heat: u8,
    }

    register GasHeaterResistance7(addr = 0x61, mode = rw, size = 1) {
        res_heat: u8,
    }

    register GasHeaterResistance8(addr = 0x62, mode = rw, size = 1) {
        res_heat: u8,
    }

    register GasHeaterResistance9(addr = 0x63, mode = rw, size = 1) {
        res_heat: u8,
    }

    #[allow(non_camel_case_types)]
    enum WaitTimeMul: u8{2} {
        0b00 X_1,
        0b01 X_4,
        0b10 X_16,
        0b11 X_64,
    }

    struct GasWaitTime(size = 1) {
        /// 64 steps of 1 ms
        mul: WaitTimeMul = WaitTimeMul::X_1,
        time: u8{6} = 0,
    }

    register GasWait(addr = 0x64, mode = rw, size = 10) {
        gas_wait_0: GasWaitTimeUnpacked,
        gas_wait_1: GasWaitTimeUnpacked,
        gas_wait_2: GasWaitTimeUnpacked,
        gas_wait_3: GasWaitTimeUnpacked,
        gas_wait_4: GasWaitTimeUnpacked,
        gas_wait_5: GasWaitTimeUnpacked,
        gas_wait_6: GasWaitTimeUnpacked,
        gas_wait_7: GasWaitTimeUnpacked,
        gas_wait_8: GasWaitTimeUnpacked,
        gas_wait_9: GasWaitTimeUnpacked,
    }

    register GasWait0(addr = 0x64, mode = rw, size = 1) {
        val: GasWaitTimeUnpacked,
    }

    register GasWait1(addr = 0x65, mode = rw, size = 1) {
        val: GasWaitTimeUnpacked,
    }

    register GasWait2(addr = 0x66, mode = rw, size = 1) {
        val: GasWaitTimeUnpacked,
    }

    register GasWait3(addr = 0x67, mode = rw, size = 1) {
        val: GasWaitTimeUnpacked,
    }

    register GasWait4(addr = 0x68, mode = rw, size = 1) {
        val: GasWaitTimeUnpacked,
    }

    register GasWait5(addr = 0x69, mode = rw, size = 1) {
        val: GasWaitTimeUnpacked,
    }

    register GasWait6(addr = 0x6A, mode = rw, size = 1) {
        val: GasWaitTimeUnpacked,
    }

    register GasWait7(addr = 0x6B, mode = rw, size = 1) {
        val: GasWaitTimeUnpacked,
    }

    register GasWait8(addr = 0x6C, mode = rw, size = 1) {
        val: GasWaitTimeUnpacked,
    }

    register GasWait9(addr = 0x6D, mode = rw, size = 1) {
        val: GasWaitTimeUnpacked,
    }

    register GasWaitShared(addr = 0x6E, mode = rw, size = 1) {
        /// 64 steps of 0.477 ms
        time: u8{6} = 0,
        mul: WaitTimeMul = WaitTimeMul::X_1,
    }

    // Data registers

    struct Value20Bit(size = 3) {
        val: u32{24, be},
    }

    struct GasData(size = 2) {
        gas_r: u16{10, be},
        gas_valid_r: bool,
        heat_stab_r: bool,
        gas_range_r: u8{4},
    }

    struct MeasurementStatus(size = 2) {
        new_data: bool,
        gas_measuring: bool,
        measuring: bool,
        _: u8{1},
        gas_meas_index: u8{4},
        sub_meas_index: u8,
    }

    register Pressure0(addr = 0x1F, mode = r, size = 3) {
        pressure: Value20BitUnpacked,
    }

    register Pressure1(addr = 0x30, mode = r, size = 3) {
        pressure: Value20BitUnpacked,
    }

    register Pressure2(addr = 0x41, mode = r, size = 3) {
        pressure: Value20BitUnpacked,
    }

    register Temperature0(addr = 0x22, mode = r, size = 3) {
        temperature: Value20BitUnpacked,
    }

    register Temperature1(addr = 0x33, mode = r, size = 3) {
        temperature: Value20BitUnpacked,
    }

    register Temperature2(addr = 0x44, mode = r, size = 3) {
        temperature: Value20BitUnpacked,
    }

    register Humidity0(addr = 0x25, mode = r, size = 2) {
        humidity: u16{be},
    }

    register Humidity1(addr = 0x36, mode = r, size = 2) {
        humidity: u16{be},
    }

    register Humidity2(addr = 0x47, mode = r, size = 2) {
        humidity: u16{be},
    }

    register Gas0(addr = 0x2C, mode = r, size = 2) {
        gas: GasDataUnpacked,
    }

    register Gas1(addr = 0x3D, mode = r, size = 2) {
        gas: GasDataUnpacked,
    }

    register Gas2(addr = 0x4E, mode = r, size = 2) {
        gas: GasDataUnpacked,
    }

    register Status0(addr = 0x1D, mode = r, size = 2) {
        status: MeasurementStatusUnpacked,
    }

    register Status1(addr = 0x2E, mode = r, size = 2) {
        status: MeasurementStatusUnpacked,
    }

    register Status2(addr = 0x3F, mode = r, size = 2) {
        status: MeasurementStatusUnpacked,
    }

    register BurstMeasurementsPT0(addr = 0x1F, mode = r, size = 6) {
        pressure: Value20BitUnpacked,
        temperature: Value20BitUnpacked,
    }

    register BurstMeasurementsPT1(addr = 0x30, mode = r, size = 6) {
        pressure: Value20BitUnpacked,
        temperature: Value20BitUnpacked,
    }

    register BurstMeasurementsPT2(addr = 0x41, mode = r, size = 6) {
        pressure: Value20BitUnpacked,
        temperature: Value20BitUnpacked,
    }

    register BurstMeasurementsPTH0(addr = 0x1F, mode = r, size = 8) {
        pressure: Value20BitUnpacked,
        temperature: Value20BitUnpacked,
        humidity: u16{be},
    }

    register BurstMeasurementsPTH1(addr = 0x30, mode = r, size = 8) {
        pressure: Value20BitUnpacked,
        temperature: Value20BitUnpacked,
        humidity: u16{be},
    }

    register BurstMeasurementsPTH2(addr = 0x41, mode = r, size = 8) {
        pressure: Value20BitUnpacked,
        temperature: Value20BitUnpacked,
        humidity: u16{be},
    }

    // Calibration registers

    register ParT1(addr = 0xE9, mode = r, size = 2) {
        par_t1: u16{le},
    }

    register ParT2_3(addr = 0x8A, mode = r, size = 3) {
        par_t2: u16{le},
        par_t3: i8,
    }

    register ParP(addr = 0x8E, mode = r, size = 18) {
        par_p5: i16{le},
        par_p6: i16{le},
        par_p7: i8,
        par_p8: i8,

        par_p1: u16{le},
        par_p2: u16{le},
        par_p3: i8,
        par_p4: i8,

        /// Reserved bytes
        _: u16,

        par_p9: i16{le},
        par_p10: i8,
        par_p11: i8,
    }

    register ParH(addr = 0xE1, mode = r, size = 7) {
                                    // LSB       | MSB
        par_h5: i16[0..8, 8..12],   // 0xE2[7:4] | 0xE1[7:0]
        par_h1: i16[16..24,12..16], // 0xE2[3:0] | 0xE3[7:0]
        par_h2: i8,                 // 0xE4
        par_h4: i8,                 // 0xE5
        par_h3: u8,                 // 0xE6
        par_h6: u8,                 // 0xE7
    }

    register ParG(addr = 0xEB, mode = r, size = 4) {
        par_g2: i16{le},
        par_g1: i8,
        par_g3: i8,
    }

    register ResHeat(addr = 0x00, mode = r, size = 3) {
        res_heat_val: i8,
        _: u16{10},
        res_heat_range: u8{2},
        _: u8{4},
    }

    register CalibData0(addr = 0x8A, mode = r, size = 22) {
        temp2_3: ParT2_3Unpacked,

        /// Reserved byte
        _: u8,

        pressure: ParPUnpacked,
    }

    register CalibData1(addr = 0xE1, mode = r, size = 14) {
        humidity: ParHUnpacked,

        /// Reserved byte
        _: u8, // 0xE8

        temp1: ParT1Unpacked,
        gas: ParGUnpacked,
    }
}

impl Oversampling {
    pub const fn fac(&self) -> u8 {
        match self {
            Self::Disabled => 0,
            Self::X_1 => 1,
            Self::X_2 => 2,
            Self::X_4 => 4,
            Self::X_8 => 8,
            Self::X_16 => 16,
        }
    }
}

impl WaitTimeMul {
    const fn fac(&self) -> u8 {
        match self {
            Self::X_1 => 1,
            Self::X_4 => 4,
            Self::X_16 => 16,
            Self::X_64 => 64,
        }
    }
}

const fn wait_to_ms(time: u8, mul: WaitTimeMul) -> u16 {
    let fac = mul.fac() as u16;
    let time = time as u16;

    fac * time
}

const fn calc_scaled_ticks(mut ticks: u16) -> (u8, WaitTimeMul) {
    let mut fac = 1;
    while ticks > 64 {
        ticks /= 4;
        fac *= 4;
    }

    let mul = match fac {
        1 => WaitTimeMul::X_1,
        4 => WaitTimeMul::X_4,
        16 => WaitTimeMul::X_16,
        64 => WaitTimeMul::X_64,
        _ => unreachable!(),
    };

    (ticks as u8, mul)
}

const fn ms_to_wait(ms: u16) -> (u8, WaitTimeMul) {
    const MAX_MS: u16 = 64 * 64; // 64ticks * 64x * 1ms/tick
    if ms >= MAX_MS {
        return (64, WaitTimeMul::X_64);
    }

    calc_scaled_ticks(ms)
}

impl GasWaitTime {
    pub fn new(ms: u16) -> Self {
        let (time, mul) = ms_to_wait(ms);
        Self::default().with_time(time).with_mul(mul)
    }

    pub fn ms(&self) -> u16 {
        wait_to_ms(self.read_time(), self.read_mul())
    }
}

impl GasWaitTimeUnpacked {
    pub fn new(ms: u16) -> Self {
        let (time, mul) = ms_to_wait(ms);
        Self { time, mul }
    }

    pub fn ms(&self) -> u16 {
        wait_to_ms(self.time, self.mul)
    }
}

const fn shared_to_us(time: u8, mul: WaitTimeMul) -> u32 {
    let fac = mul.fac() as u32;
    let time = time as u32;

    fac * time * 477
}

const fn us_to_shared(us: u32) -> (u8, WaitTimeMul) {
    const MAX_US: u32 = 64 * 64 * 477; // 64ticks * 64x * 477us/tick
    if us >= MAX_US {
        return (64, WaitTimeMul::X_64);
    }

    calc_scaled_ticks((us / 477) as u16)
}

impl GasWaitShared {
    pub fn new(us: u32) -> Self {
        let (time, mul) = us_to_shared(us);
        Self::default().with_time(time).with_mul(mul)
    }

    pub fn us(&self) -> u32 {
        shared_to_us(self.read_time(), self.read_mul())
    }
}

impl GasWaitSharedUnpacked {
    pub fn new(us: u32) -> Self {
        let (time, mul) = us_to_shared(us);
        Self { time, mul }
    }

    pub fn us(&self) -> u32 {
        shared_to_us(self.time, self.mul)
    }
}
