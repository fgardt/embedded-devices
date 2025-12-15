use embedded_devices_derive::{forward_register_fns, sensor};
use embedded_interfaces::TransportError;
use uom::si::electrical_resistance::ohm;
use uom::si::f64::{ElectricalResistance, Pressure, Ratio, ThermodynamicTemperature};
use uom::si::pressure::pascal;
use uom::si::ratio::percent;
use uom::si::thermodynamic_temperature::degree_celsius;

pub mod address;
pub mod registers;

use self::address::Address;
use registers::{CalibData0, CalibData1, IIRFilter, Id, Oversampling, ResHeat, SensorMode, Variant};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, thiserror::Error)]
pub enum InitError<BusError> {
    /// Transport error
    #[error("transport error")]
    Transport(#[from] TransportError<(), BusError>),
    /// Invalid chip id was encountered in `init`
    #[error("invalid chip id {0:#02x}")]
    InvalidChip(u8),
    /// Invalid chip variant was encountered in `init`
    #[error("invalid chip id {0:#02x}")]
    InvalidChipVariant(u8),
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, thiserror::Error)]
pub enum ConfigurationError<BusError> {
    /// Transport error
    #[error("transport error")]
    Transport(#[from] TransportError<(), BusError>),
    /// The calibration data was not read from the device. Call `init` or `calibrate` first.
    #[error("not calibrated")]
    NotCalibrated,
    /// The device is currently performing a measurement. Configuration can only be changed while the device is in [`SensorMode::Sleep`].
    #[error("cannot change configuration while measurement is active")]
    ActiveMeasurement,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, thiserror::Error)]
pub enum MeasurementError<BusError> {
    /// Transport error
    #[error("transport error")]
    Transport(#[from] TransportError<(), BusError>),
    /// The calibration data was not read from the device, but a measurement was requested. Call `init` or `calibrate` first.
    #[error("not calibrated")]
    NotCalibrated,
    /// The one-shot measurement configuration was not set, but a measurement was requested. Call `configure_oneshot` first.
    #[error("device not configured for one-shot measurement")]
    NotConfigured,
    /// Either a temperature measurement or a configured ambient temperature is required for the
    /// compensation calculations of pressure and humidity measurements, but neither was available.
    #[error("temperature measurement or configured ambient is required for compensation calculations")]
    TemperatureUnknown,
}

/// Measurement data
#[derive(Debug, embedded_devices_derive::Measurement)]
pub struct Measurement {
    /// Current temperature
    #[measurement(Temperature)]
    pub temperature: Option<ThermodynamicTemperature>,
    /// Current pressure or None if the sensor reported and invalid value
    #[measurement(Pressure)]
    pub pressure: Option<Pressure>,
    /// Current relative humidity
    #[measurement(RelativeHumidity)]
    pub humidity: Option<Ratio>,
    /// Current gas resistance or None if the sensor reported and invalid value
    #[measurement(GasResistance)]
    pub gas_resistance: Option<ElectricalResistance>,

    pub gas_valid: bool,
    pub heater_stable: bool,
}

/// Temperature values used for compensation calculations
/// for pressure and humidity measurements.
#[derive(Debug, Clone, Copy)]
struct CompTemp {
    /// linerized temperature
    t_lin: i64,
    /// temperature in °C * 100
    t_fine: i16,
}

impl CompTemp {
    const fn from_ambient(ambient_temp: i8) -> Self {
        let t_fine = ambient_temp as i16 * 100;
        // undo last step of temp compensation for t_fine -> t_lin
        let t_lin = (t_fine as i64 * CalibrationData::COMP_TEMP_TLIN_DIV) / CalibrationData::COMP_TEMP_TLIN_FAC;

        Self { t_lin, t_fine }
    }
}

#[maybe_async_cfg::maybe(
    idents(hal(sync = "embedded_hal", async = "embedded_hal_async"), RegisterInterface),
    sync(feature = "sync"),
    async(feature = "async")
)]
pub struct BME690<D: hal::delay::DelayNs, I: embedded_interfaces::registers::RegisterInterface> {
    /// The delay provider
    delay: D,
    /// The interface to communicate with the device
    interface: I,
    /// The calibration data read from the device
    calibration_data: Option<CalibrationData>,
    /// The configuration used to setup the device for [`SensorMode::Forced`] measurements.
    /// This is required to calculate the measurement duration for the delay after starting a measurement and to know which sensors are enabled.
    oneshot_config: Option<MeasurementConfig>,
}

struct CalibrationData {
    // temperature coefficients
    par_t1: u16,
    par_t2: u16,
    par_t3: i8,
    // pressure coefficients
    par_p1: u16,
    par_p2: u16,
    par_p3: i8,
    par_p4: i8,
    par_p5: i16,
    par_p6: i16,
    par_p7: i8,
    par_p8: i8,
    par_p9: i16,
    par_p10: i8,
    par_p11: i8,
    // humidity coefficients
    par_h1: i16,
    par_h2: i8,
    par_h3: u8,
    par_h4: i8,
    par_h5: i16,
    par_h6: u8,
    // gas coefficients
    par_g1: i8,
    par_g2: i16,
    par_g3: i8,
    res_heat_range: u8,
    res_heat_val: i8,
}

impl CalibrationData {
    fn new(
        data0: self::registers::CalibData0,
        data1: self::registers::CalibData1,
        res_heat: self::registers::ResHeat,
    ) -> Self {
        let data0 = data0.unpack();
        let data1 = data1.unpack();
        let res_heat = res_heat.unpack();

        // https://github.com/boschsensortec/BME690_SensorAPI/blob/bca5893f097e3fa6017bf268ad146eff8f5f1bd7/bme69x.c#L1760-L1765
        let mut par_h1 = data1.humidity.par_h1;
        if par_h1 >= 2048 {
            par_h1 -= 4096;
        }

        // https://github.com/boschsensortec/BME690_SensorAPI/blob/bca5893f097e3fa6017bf268ad146eff8f5f1bd7/bme69x.c#L1751-L1755
        let mut par_h5 = data1.humidity.par_h5;
        if par_h5 >= 2048 {
            par_h5 -= 4096;
        }

        Self {
            par_t1: data1.temp1.par_t1,
            par_t2: data0.temp2_3.par_t2,
            par_t3: data0.temp2_3.par_t3,

            par_p1: data0.pressure.par_p1,
            par_p2: data0.pressure.par_p2,
            par_p3: data0.pressure.par_p3,
            par_p4: data0.pressure.par_p4,
            par_p5: data0.pressure.par_p5,
            par_p6: data0.pressure.par_p6,
            par_p7: data0.pressure.par_p7,
            par_p8: data0.pressure.par_p8,
            par_p9: data0.pressure.par_p9,
            par_p10: data0.pressure.par_p10,
            par_p11: data0.pressure.par_p11,

            par_h1,
            par_h2: data1.humidity.par_h2,
            par_h3: data1.humidity.par_h3,
            par_h4: data1.humidity.par_h4,
            par_h5,
            par_h6: data1.humidity.par_h6,

            par_g1: data1.gas.par_g1,
            par_g2: data1.gas.par_g2,
            par_g3: data1.gas.par_g3,
            res_heat_range: res_heat.res_heat_range,
            res_heat_val: res_heat.res_heat_val,
        }
    }

    const COMP_TEMP_TLIN_FAC: i64 = 25;
    const COMP_TEMP_TLIN_DIV: i64 = 16_384;

    fn compensate_temperature(&self, temp_adc: u32) -> (ThermodynamicTemperature, CompTemp) {
        let par_t1 = self.par_t1 as i64;
        let par_t2 = self.par_t2 as i64;
        let par_t3 = self.par_t3 as i64;

        // TODO: investigate if where this left shift comes from
        let temp_adc_s = (temp_adc as i64) << 4;

        let v1 = temp_adc_s - par_t1 * 128;
        let v2 = v1 * par_t2;
        let v3 = v1 * v1;
        let v4 = v3 * par_t3;
        let v5 = v4 + v2 * 262_144;

        let t_lin = v5 / (4_294_967_296 * 100);
        let t_fine = ((t_lin * Self::COMP_TEMP_TLIN_FAC) / Self::COMP_TEMP_TLIN_DIV) as i16;
        let temperature = t_fine as f64 / 100.0;

        (
            ThermodynamicTemperature::new::<degree_celsius>(temperature),
            CompTemp { t_lin, t_fine },
        )
    }

    fn compensate_pressure(&self, press_adc: u32, t_lin: i64) -> Pressure {
        let par_p1 = self.par_p1 as i64;
        let par_p2 = self.par_p2 as i64;
        let par_p3 = self.par_p3 as i64;
        let par_p4 = self.par_p4 as i64;
        let par_p5 = self.par_p5 as i32;
        let par_p6 = self.par_p6 as i32;
        let par_p7 = self.par_p7 as i64;
        let par_p8 = self.par_p8 as i64;
        let par_p9 = self.par_p9 as i64;
        let par_p10 = self.par_p10 as i64;
        let par_p11 = self.par_p11 as i64;

        let press_adc = press_adc as i64;

        let v1 = t_lin * t_lin;
        let v2 = v1 >> 6;
        let v3 = (t_lin * v2) >> 8;
        let v4 = (par_p4 * v3) >> 5;
        let v5 = (par_p3 * v1) << 4;
        let v6 = (par_p2 * t_lin) << 22;

        let offset = (par_p1 << 47) + v4 + v5 + v6;

        let v2 = (par_p8 * v3) >> 5;
        let v4 = (par_p7 * v1) << 2;
        let v5 = ((par_p6 - 16_384i32) as i64 * t_lin) << 21;

        let sensitivity = (((par_p5 - 16_384i32) as i64) << 46) + v2 + v4 + v5;

        let v1 = (sensitivity >> 24) * press_adc;
        let v2 = par_p10 * t_lin;
        let v3 = v2 + (par_p9 << 16);
        let v4 = (press_adc * v3) >> 13;
        let v5 = (press_adc * (v4 / 10)) >> 9;
        let v5 = v5 * 10;
        let v6 = press_adc * press_adc;
        let v7 = (par_p11 * v6) >> 16;
        let v8 = (press_adc * v7) >> 7;
        let v9 = (offset / 4) + v1 + v5 + v8;

        let pressure = (((v9 as u64) * 25) >> 40) as f64 / 100.0;

        Pressure::new::<pascal>(pressure)
    }

    fn compensate_humidity(&self, hum_adc: u32, t_fine: i16) -> Ratio {
        let par_h1 = self.par_h1 as i64;
        let par_h2 = self.par_h2 as i64;
        let par_h3 = self.par_h3 as i64;
        let par_h4 = self.par_h4 as i64;
        let par_h5 = self.par_h5 as i64;
        let par_h6 = self.par_h6 as i64;

        let hum_adc = hum_adc as i64;
        let t_fine = (((t_fine as i64) << 8) - 128) / 5;

        let v1 = t_fine - 76_800;
        let v2 = ((hum_adc << 14) - (par_h1 << 20) - (par_h2 * v1)) + 16_384;
        let v3 = ((par_h4 * v1) >> 10) * (((par_h3 * v1) >> 11) + 32_768);
        let v3 = ((v3 >> 10) + 2_097_152) * par_h5 + 8_192;
        let v1 = ((v2 >> 15) * v3) >> 14;

        let v2 = v1 >> 15;
        let v1 = v1 - ((((v2 * v2) >> 7) * par_h6) >> 4);

        let v1 = v1.clamp(0, 419_430_400);
        let humidity = (v1 >> 12) as f64 / 1024.0;

        Ratio::new::<percent>(humidity)
    }

    fn calculate_res_heat(&self, target_temp: u16, ambient_temp: i8) -> u8 {
        let par_g1 = self.par_g1 as i32;
        let par_g2 = self.par_g2 as i32;
        let par_g3 = self.par_g3 as i32;
        let res_heat_range = self.res_heat_range as i32;
        let res_heat_val = self.res_heat_val as i32;

        let target_temp = target_temp.min(400);

        let v1 = ((ambient_temp as i32 * par_g3) / 1000) << 8;
        let v2 = (par_g1 + 784) * (((((par_g2 + 154_009) * target_temp as i32 * 5) / 100) + 3_276_800) / 10);
        let v3 = v1 + (v2 >> 1);
        let v4 = v3 / (res_heat_range + 4);
        let v5 = res_heat_val * 131 + 65_536;

        let res_heat_x100 = ((v4 / v5) - 250) * 34;
        ((res_heat_x100 + 50) / 100) as u8
    }
}

fn calculate_gas_resistance(gas_adc: u32, gas_range: u8) -> ElectricalResistance {
    let v1 = 262_144u32 >> gas_range;
    let v2 = gas_adc as i32 - 512;
    let v2 = v2 * 3 + 4_096;

    let gas_resistance = 1_000_000.0 * f64::from(v1) / f64::from(v2);

    ElectricalResistance::new::<ohm>(gas_resistance)
}

pub trait BME690Register {}

#[maybe_async_cfg::maybe(
    idents(hal(sync = "embedded_hal", async = "embedded_hal_async"), I2cDevice),
    sync(feature = "sync"),
    async(feature = "async")
)]
impl<D, I> BME690<D, embedded_interfaces::i2c::I2cDevice<I, hal::i2c::SevenBitAddress>>
where
    I: hal::i2c::I2c<hal::i2c::SevenBitAddress> + hal::i2c::ErrorType,
    D: hal::delay::DelayNs,
{
    /// Initializes a new device with the given address on the specified bus.
    /// This consumes the I2C bus `I`.
    ///
    /// Before using this device, you must call the [`Self::init`] method which
    /// initializes the device and ensures that it is working correctly.
    #[inline]
    pub fn new_i2c(delay: D, interface: I, address: Address) -> Self {
        Self {
            delay,
            interface: embedded_interfaces::i2c::I2cDevice::new(interface, address.into()),
            calibration_data: None,
            oneshot_config: None,
        }
    }
}

// TODO: SPI codec + new_spi method

#[derive(Debug, Clone, Copy)]
pub struct MeasurementConfig {
    pressure_oversampling: Oversampling,
    humidity_oversampling: Oversampling,
    temp_oversampling: Oversampling,
    iir_filter: IIRFilter,

    gas_config: Option<GasMeasurementConfig>,
}

#[derive(Debug, Clone, Copy)]
struct GasMeasurementConfig {
    heating_time_ms: u16,
    target_temperature: u16,
    ambient_temperature: i8,
}

impl MeasurementConfig {
    pub const fn new() -> Self {
        Self {
            pressure_oversampling: Oversampling::Disabled,
            humidity_oversampling: Oversampling::Disabled,
            temp_oversampling: Oversampling::Disabled,
            iir_filter: IIRFilter::Disabled,
            gas_config: None,
        }
    }

    pub const fn with_pressure_oversampling(mut self, oversampling: Oversampling) -> Self {
        self.pressure_oversampling = oversampling;
        self
    }

    pub const fn with_humidity_oversampling(mut self, oversampling: Oversampling) -> Self {
        self.humidity_oversampling = oversampling;
        self
    }

    pub const fn with_temp_oversampling(mut self, oversampling: Oversampling) -> Self {
        self.temp_oversampling = oversampling;
        self
    }

    pub const fn with_iir_filter(mut self, filter: IIRFilter) -> Self {
        self.iir_filter = filter;
        self
    }

    pub const fn with_gas(mut self, heating_time_ms: u16, target_temperature: u16, ambient_temperature: i8) -> Self {
        self.gas_config = Some(GasMeasurementConfig {
            heating_time_ms,
            target_temperature,
            ambient_temperature,
        });
        self
    }

    fn measurement_duration_us(&self) -> u32 {
        // https://github.com/boschsensortec/BME690_SensorAPI/blob/bca5893f097e3fa6017bf268ad146eff8f5f1bd7/bme69x.c#L472-L518
        const TPH_CYCLE_TIME_US: u32 = 1963;
        const TIME_STEP_US: u32 = 477;

        let temp_fac = self.temp_oversampling.fac();
        let pressure_fac = self.pressure_oversampling.fac();
        let humidity_fac = self.humidity_oversampling.fac();

        let cycles = temp_fac + pressure_fac + humidity_fac;

        let mut res = TPH_CYCLE_TIME_US * cycles as u32;
        res += TIME_STEP_US * 4; // TPH switching

        if let Some(gas_config) = &self.gas_config {
            res += (gas_config.heating_time_ms as u32) * 1000;
            res += TIME_STEP_US * 5; // Gas measurement
        }

        res += 1000; // Sleep wakeup time

        res
    }
}

impl Default for MeasurementConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[forward_register_fns]
#[sensor(Temperature, Pressure, RelativeHumidity, GasResistance)]
#[maybe_async_cfg::maybe(
    idents(
        hal(sync = "embedded_hal", async = "embedded_hal_async"),
        RegisterInterface,
        ResettableDevice
    ),
    sync(feature = "sync"),
    async(feature = "async")
)]
impl<D: hal::delay::DelayNs, I: embedded_interfaces::registers::RegisterInterface> BME690<D, I> {
    /// Initialize the sensor by performing a soft-reset, verifying its
    /// chip id, chip variant and reading calibration data.
    ///
    /// Beware that by default all internal sensors are disabled.
    /// Please call [`Self::configure_oneshot`] after initialization
    /// to enable the sensors you want and set other configuration options.
    pub async fn init(&mut self) -> Result<(), InitError<I::BusError>> {
        use crate::device::ResettableDevice;

        // Soft-reset device
        self.reset().await?;

        // Verify chip id
        let chip = self.read_register::<Id>().await?.read_chip();
        if let self::registers::Chip::Invalid(x) = chip {
            return Err(InitError::InvalidChip(x));
        }

        // Verify variant id
        let variant = self.read_register::<Variant>().await?.read_variant();
        if let self::registers::ChipVariant::Invalid(x) = variant {
            return Err(InitError::InvalidChipVariant(x));
        }

        // Read calibration data
        self.calibrate().await?;
        Ok(())
    }

    /// Reads the calibration registers from the device to
    /// compensate measurements. It is required to call this once
    /// before taking any measurements. Calling [`Self::init`] will
    /// automatically do this.
    pub async fn calibrate(&mut self) -> Result<(), TransportError<(), I::BusError>> {
        let data0 = self.read_register::<CalibData0>().await?;
        let data1 = self.read_register::<CalibData1>().await?;
        let res_heat = self.read_register::<ResHeat>().await?;

        self.calibration_data = Some(CalibrationData::new(data0, data1, res_heat));

        Ok(())
    }

    /// Configures the sensor for one-shot measurements with the provided configuration.
    ///
    /// <div class="warning">Calling this function resets the sensor data history used by the IIR filter.</div>
    pub async fn configure_oneshot(
        &mut self,
        config: MeasurementConfig,
    ) -> Result<(), ConfigurationError<I::BusError>> {
        use registers::{CombinedConfigForced, GasHeaterResistance0, GasWait0, GasWaitTime, HeaterStep};

        let mut combined_config = self.read_register::<CombinedConfigForced>().await?;

        if combined_config.read_ctrl_measure_mode() != SensorMode::Sleep {
            return Err(ConfigurationError::ActiveMeasurement);
        }

        if let Some(gas_config) = &config.gas_config {
            let Some(calib_data) = &self.calibration_data else {
                return Err(ConfigurationError::NotCalibrated);
            };

            let heating_time = GasWait0::default().with_val(GasWaitTime::new(gas_config.heating_time_ms));
            let res_heat = GasHeaterResistance0::default().with_res_heat(
                calib_data.calculate_res_heat(gas_config.target_temperature, gas_config.ambient_temperature),
            );

            self.write_register(heating_time).await?;
            self.write_register(res_heat).await?;

            combined_config.write_ctrl_gas_1_nb_conv(HeaterStep::Step0);
            combined_config.write_ctrl_gas_1_run_gas(true);
        } else {
            combined_config.write_ctrl_gas_1_run_gas(false);
        }

        combined_config.write_ctrl_hum_osrs_h(config.humidity_oversampling);
        combined_config.write_ctrl_measure_osrs_t(config.temp_oversampling);
        combined_config.write_ctrl_measure_osrs_p(config.pressure_oversampling);
        combined_config.write_config_filter(config.iir_filter);

        self.write_register(combined_config.read_ctrl_hum()).await?;
        self.write_register(combined_config.read_ctrl_measure()).await?;
        self.write_register(combined_config.read_config()).await?;
        self.write_register(combined_config.read_ctrl_gas_1()).await?;
        self.write_register(combined_config.read_ctrl_gas_0()).await?;

        self.oneshot_config = Some(config);

        Ok(())
    }
}

#[maybe_async_cfg::maybe(
    idents(
        hal(sync = "embedded_hal", async = "embedded_hal_async"),
        RegisterInterface,
        ResettableDevice
    ),
    sync(feature = "sync"),
    async(feature = "async")
)]
impl<D: hal::delay::DelayNs, I: embedded_interfaces::registers::RegisterInterface> crate::device::ResettableDevice
    for BME690<D, I>
{
    type Error = TransportError<(), I::BusError>;

    /// Performs a soft-reset of the device. The datasheet specifies a start-up time
    /// of 2ms, which is automatically awaited before allowing further communication.
    async fn reset(&mut self) -> Result<(), Self::Error> {
        // In SPI mode no extra logic to reset the remembered page is required since
        // the reset register is on page 0 which is also the default page after startup.
        self.write_register(self::registers::Reset::default()).await?;
        self.oneshot_config = None;
        self.delay.delay_ms(2).await;
        Ok(())
    }
}

#[maybe_async_cfg::maybe(
    idents(
        hal(sync = "embedded_hal", async = "embedded_hal_async"),
        RegisterInterface,
        OneshotSensor
    ),
    sync(feature = "sync"),
    async(feature = "async")
)]
impl<D: hal::delay::DelayNs, I: embedded_interfaces::registers::RegisterInterface> crate::sensor::OneshotSensor
    for BME690<D, I>
{
    type Error = MeasurementError<I::BusError>;
    type Measurement = Measurement;

    /// Performs a one-shot measurement.
    async fn measure(&mut self) -> Result<Self::Measurement, Self::Error> {
        use registers::{BurstMeasurementsPTH0, CtrlMeasure, Gas0, SensorMode, Status0};

        if self.calibration_data.is_none() {
            return Err(MeasurementError::NotCalibrated);
        }

        let Some(config) = self.oneshot_config else {
            return Err(MeasurementError::NotConfigured);
        };

        let measurement_duration_us = config.measurement_duration_us();

        let mut ctrl_measure = self.read_register::<CtrlMeasure>().await?;
        ctrl_measure.write_mode(SensorMode::Forced);
        self.write_register(ctrl_measure).await?;

        self.delay.delay_us(measurement_duration_us).await;

        loop {
            let status = self.read_register::<Status0>().await?;
            if !status.read_status_measuring() && status.read_status_new_data() {
                break;
            }

            self.delay.delay_us(500).await;
        }

        let mut measurement = Measurement {
            temperature: None,
            pressure: None,
            humidity: None,
            gas_resistance: None,
            gas_valid: false,
            heater_stable: false,
        };

        let pth_data = self.read_register::<BurstMeasurementsPTH0>().await?;

        let Some(calib_data) = &self.calibration_data else {
            return Err(MeasurementError::NotCalibrated);
        };

        let comp_temp = if config.temp_oversampling != Oversampling::Disabled {
            let temp_adc = pth_data.read_temperature_val();
            let (temperature, comp_temp) = calib_data.compensate_temperature(temp_adc);
            measurement.temperature = Some(temperature);

            Some(comp_temp)
        } else {
            config
                .gas_config
                .as_ref()
                .map(|gas_config| CompTemp::from_ambient(gas_config.ambient_temperature))
        };

        if config.pressure_oversampling != Oversampling::Disabled {
            let Some(comp_temp) = comp_temp else {
                return Err(MeasurementError::TemperatureUnknown);
            };

            let press_adc = pth_data.read_pressure_val();
            let pressure = calib_data.compensate_pressure(press_adc, comp_temp.t_lin);
            measurement.pressure = Some(pressure);
        }

        if config.humidity_oversampling != Oversampling::Disabled {
            let Some(comp_temp) = comp_temp else {
                return Err(MeasurementError::TemperatureUnknown);
            };

            let hum_adc = pth_data.read_humidity() as u32; // TODO: pass as u16 instead?
            let humidity = calib_data.compensate_humidity(hum_adc, comp_temp.t_fine);
            measurement.humidity = Some(humidity);
        }

        if config.gas_config.is_some() {
            let gas_data = self.read_register::<Gas0>().await?;

            let gas_adc = gas_data.read_gas_gas_r() as u32; // TODO: pass as u16 instead?
            let gas_range = gas_data.read_gas_gas_range_r();
            let gas_resistance = calculate_gas_resistance(gas_adc, gas_range);
            measurement.gas_resistance = Some(gas_resistance);
            measurement.gas_valid = gas_data.read_gas_gas_valid_r();
            measurement.heater_stable = gas_data.read_gas_heat_stab_r();
        }

        Ok(measurement)
    }
}
