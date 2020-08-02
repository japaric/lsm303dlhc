//! A platform agnostic driver to interface with the LSM303DLHC (accelerometer + compass)
//!
//! This driver was built using [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal/~0.2
//!
//! # Examples
//!
//! You should find at least one example in the [f3] crate.
//!
//! [f3]: https://docs.rs/f3/~0.6

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

extern crate cast;
extern crate embedded_hal as hal;
extern crate generic_array;

use core::mem;

use cast::u16;
use generic_array::typenum::consts::*;
use generic_array::{ArrayLength, GenericArray};
use hal::blocking::i2c::{Write, WriteRead};

mod accel;
mod mag;

/// LSM303DLHC driver
pub struct Lsm303dlhc<I2C> {
    i2c: I2C,
}

impl<I2C, E> Lsm303dlhc<I2C>
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
{
    /// Creates a new driver from a I2C peripheral
    pub fn new(i2c: I2C) -> Result<Self, E> {
        let mut lsm303dlhc = Lsm303dlhc { i2c };

        // TODO reset all the registers / the device

        // configure the accelerometer to operate at 400 Hz
        lsm303dlhc.write_accel_register(accel::Register::CTRL_REG1_A, 0b0111_0_111)?;

        // configure the magnetometer to operate in continuous mode
        lsm303dlhc.write_mag_register(mag::Register::MR_REG_M, 0b00)?;

        // enable the temperature sensor
        lsm303dlhc.write_mag_register(mag::Register::CRA_REG_M, 0b0001000 | (1 << 7))?;

        Ok(lsm303dlhc)
    }

    /// Accelerometer measurements
    pub fn accel(&mut self) -> Result<I16x3, E> {
        let buffer: GenericArray<u8, U6> = self.read_accel_registers(accel::Register::OUT_X_L_A)?;

        Ok(I16x3 {
            x: (u16(buffer[0]) + (u16(buffer[1]) << 8)) as i16,
            y: (u16(buffer[2]) + (u16(buffer[3]) << 8)) as i16,
            z: (u16(buffer[4]) + (u16(buffer[5]) << 8)) as i16,
        })
    }

    /// Sets the accelerometer output data rate
    pub fn accel_odr(&mut self, odr: AccelOdr) -> Result<(), E> {
        self.modify_accel_register(accel::Register::CTRL_REG1_A, |r| {
            r & !(0b1111 << 4) | ((odr as u8) << 4)
        })
    }

    /// Magnetometer measurements
    pub fn mag(&mut self) -> Result<I16x3, E> {
        let buffer: GenericArray<u8, U6> = self.read_mag_registers(mag::Register::OUT_X_H_M)?;

        Ok(I16x3 {
            x: (u16(buffer[1]) + (u16(buffer[0]) << 8)) as i16,
            y: (u16(buffer[5]) + (u16(buffer[4]) << 8)) as i16,
            z: (u16(buffer[3]) + (u16(buffer[2]) << 8)) as i16,
        })
    }

    /// Sets the magnetometer output data rate
    pub fn mag_odr(&mut self, odr: MagOdr) -> Result<(), E> {
        self.modify_mag_register(mag::Register::CRA_REG_M, |r| {
            r & !(0b111 << 2) | ((odr as u8) << 2)
        })
    }

    /// Temperature sensor measurement
    ///
    /// - Resolution: 12-bit
    /// - Range: [-40, +85]
    pub fn temp(&mut self) -> Result<i16, E> {
        let temp_out_l = self.read_mag_register(mag::Register::TEMP_OUT_L_M)?;
        let temp_out_h = self.read_mag_register(mag::Register::TEMP_OUT_H_M)?;

        Ok(((u16(temp_out_l) + (u16(temp_out_h) << 8)) as i16) >> 4)
    }

    /// Changes the `sensitivity` of the accelerometer
    pub fn set_accel_sensitivity(&mut self, sensitivity: Sensitivity) -> Result<(), E> {
        self.modify_accel_register(accel::Register::CTRL_REG4_A, |r| {
            r & !(0b11 << 4) | (sensitivity.value() << 4)
        })
    }

    fn modify_accel_register<F>(&mut self, reg: accel::Register, f: F) -> Result<(), E>
    where
        F: FnOnce(u8) -> u8,
    {
        let r = self.read_accel_register(reg)?;
        self.write_accel_register(reg, f(r))?;
        Ok(())
    }

    fn modify_mag_register<F>(&mut self, reg: mag::Register, f: F) -> Result<(), E>
    where
        F: FnOnce(u8) -> u8,
    {
        let r = self.read_mag_register(reg)?;
        self.write_mag_register(reg, f(r))?;
        Ok(())
    }

    fn read_accel_registers<N>(&mut self, reg: accel::Register) -> Result<GenericArray<u8, N>, E>
    where
        N: ArrayLength<u8>,
    {
        let mut buffer: GenericArray<u8, N> = unsafe { mem::MaybeUninit::uninit().assume_init() };

        {
            let buffer: &mut [u8] = &mut buffer;

            const MULTI: u8 = 1 << 7;
            self.i2c
                .write_read(accel::ADDRESS, &[reg.addr() | MULTI], buffer)?;
        }

        Ok(buffer)
    }

    fn read_accel_register(&mut self, reg: accel::Register) -> Result<u8, E> {
        self.read_accel_registers::<U1>(reg).map(|b| b[0])
    }

    fn read_mag_register(&mut self, reg: mag::Register) -> Result<u8, E> {
        let buffer: GenericArray<u8, U1> = self.read_mag_registers(reg)?;
        Ok(buffer[0])
    }

    // NOTE has weird address increment semantics; use only with `OUT_X_H_M`
    fn read_mag_registers<N>(&mut self, reg: mag::Register) -> Result<GenericArray<u8, N>, E>
    where
        N: ArrayLength<u8>,
    {
        let mut buffer: GenericArray<u8, N> = unsafe { mem::MaybeUninit::uninit().assume_init() };

        {
            let buffer: &mut [u8] = &mut buffer;

            self.i2c.write_read(mag::ADDRESS, &[reg.addr()], buffer)?;
        }

        Ok(buffer)
    }

    fn write_accel_register(&mut self, reg: accel::Register, byte: u8) -> Result<(), E> {
        self.i2c.write(accel::ADDRESS, &[reg.addr(), byte])
    }

    fn write_mag_register(&mut self, reg: mag::Register, byte: u8) -> Result<(), E> {
        self.i2c.write(mag::ADDRESS, &[reg.addr(), byte])
    }
}

/// XYZ triple
#[derive(Debug)]
pub struct I16x3 {
    /// X component
    pub x: i16,
    /// Y component
    pub y: i16,
    /// Z component
    pub z: i16,
}

/// Accelerometer Output Data Rate
pub enum AccelOdr {
    /// 1 Hz
    Hz1 = 0b0001,
    /// 10 Hz
    Hz10 = 0b0010,
    /// 25 Hz
    Hz25 = 0b0011,
    /// 50 Hz
    Hz50 = 0b0100,
    /// 100 Hz
    Hz100 = 0b0101,
    /// 200 Hz
    Hz200 = 0b0110,
    /// 400 Hz
    Hz400 = 0b0111,
}

/// Magnetometer Output Data Rate
pub enum MagOdr {
    /// 0.75 Hz
    Hz0_75 = 0b000,
    /// 1.5 Hz
    Hz1_5 = 0b001,
    /// 3 Hz
    Hz3 = 0b010,
    /// 7.5 Hz
    Hz7_5 = 0b011,
    /// 15 Hz
    Hz15 = 0b100,
    /// 30 Hz
    Hz30 = 0b101,
    /// 75 Hz
    Hz75 = 0b110,
    /// 220 Hz
    Hz220 = 0b111,
}

/// Acceleration sensitivity
#[derive(Clone, Copy)]
pub enum Sensitivity {
    /// Range: [-2g, +2g]. Sensitivity ~ 1 g / (1 << 14) LSB
    G1,
    /// Range: [-4g, +4g]. Sensitivity ~ 2 g / (1 << 14) LSB
    G2,
    /// Range: [-8g, +8g]. Sensitivity ~ 4 g / (1 << 14) LSB
    G4,
    /// Range: [-16g, +16g]. Sensitivity ~ 12 g / (1 << 14) LSB
    G12,
}

impl Sensitivity {
    fn value(&self) -> u8 {
        *self as u8
    }
}
