use hidapi::HidDevice;
#[cfg(feature = "units")]
use uom::si::{
    f64::Force,
    force::{kilogram_force, pound_force},
};

use crate::{HidScaleError, Result, ScaleDriver};

pub const VENDOR_ID: u16 = 0x0922;

pub const M_10_ID: u16 = 0x8009;

pub fn make_driver(product_id: u16, device: HidDevice) -> Option<Box<dyn ScaleDriver>> {
    match product_id {
        M_10_ID => Some(Box::new(DymoS100 { device })),
        _ => None,
    }
}

#[derive(Copy, Clone, Debug)]
enum DymoUnit {
    Kilograms(f64),
    Pounds(f64),
}

struct DymoS100 {
    device: HidDevice,
}

// 00 03 04 0B FF 31 00 end
fn parse_buffer(buf: &[u8]) -> Result<DymoUnit> {
    // I've seen situations where there's a 0 byte prepended, not sure why, but
    // 3 is a reliable beginning byte (AFAICT).
    let offset = buf.iter().take_while(|b| **b != 3).count();
    let exponent = buf[offset + 3] as i8;
    let scalar = 10.0_f64.powi(exponent as i32);
    let value = u16::from_le_bytes([buf[offset + 4], buf[offset + 5]]) as f64;
    let unit_code = buf[offset + 2];

    // All of these magic numbers were found by playing with hardware, no telling.
    match buf[offset + 1] {
        2 | 4 if unit_code == 3 => Ok(DymoUnit::Kilograms(value * scalar)),
        2 | 4 if unit_code == 12 => Ok(DymoUnit::Pounds(value * scalar)),
        5 => Err(HidScaleError::UnreportableReading),
        6 => Err(HidScaleError::Overloaded),
        _ => Err(HidScaleError::UnknownUnits),
    }
}

impl DymoS100 {
    fn read_raw(&self) -> Result<DymoUnit> {
        let mut buf = [0; 128];
        match self.device.read(&mut buf) {
            Ok(count) if count >= 6 => parse_buffer(&buf),
            Ok(_) => Err(HidScaleError::NotEnoughData),
            Err(e) => Err(HidScaleError::CantReadDueTo(format!("{:?}", e))),
        }
    }
}

impl ScaleDriver for DymoS100 {
    #[cfg(feature = "units")]
    fn read(&self) -> Result<Force> {
        match self.read_raw() {
            Ok(DymoUnit::Kilograms(kg)) => Ok(Force::new::<kilogram_force>(kg / 453.592)),
            Ok(DymoUnit::Pounds(lb)) => Ok(Force::new::<pound_force>(lb)),
            Err(e) => Err(e),
        }
    }

    fn read_kilograms(&self) -> Result<f64> {
        match self.read_raw() {
            Ok(DymoUnit::Kilograms(kg)) => Ok(kg / 100.0),
            Ok(DymoUnit::Pounds(lb)) => Ok(lb),
            Err(e) => Err(e),
        }
    }

    fn read_pounds(&self) -> Result<f64> {
        match self.read_raw() {
            Ok(DymoUnit::Kilograms(kg)) => Ok(kg / 453.592),
            Ok(DymoUnit::Pounds(lb)) => Ok(lb / 16.0),
            Err(e) => Err(e),
        }
    }
}
