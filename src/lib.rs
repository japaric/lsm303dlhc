//! A WIP, no_std, generic driver for the LSM303DLHC (accelerometer + magnetometer)

#![deny(missing_docs)]
#![deny(warnings)]
#![feature(unsize)]
#![no_std]

extern crate cast;
extern crate embedded_hal as hal;

use core::marker::Unsize;
use core::mem;

use cast::u16;
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
        let buffer: [u8; 6] = self.read_accel_registers(accel::Register::OUT_X_L_A)?;

        Ok(I16x3 {
            x: (u16(buffer[0]) + (u16(buffer[1]) << 8)) as i16,
            y: (u16(buffer[2]) + (u16(buffer[3]) << 8)) as i16,
            z: (u16(buffer[4]) + (u16(buffer[5]) << 8)) as i16,
        })
    }

    /// Magnetometer measurements
    pub fn mag(&mut self) -> Result<I16x3, E> {
        let buffer: [u8; 6] = self.read_mag_registers(mag::Register::OUT_X_H_M)?;

        Ok(I16x3 {
            x: (u16(buffer[1]) + (u16(buffer[0]) << 8)) as i16,
            y: (u16(buffer[5]) + (u16(buffer[4]) << 8)) as i16,
            z: (u16(buffer[3]) + (u16(buffer[2]) << 8)) as i16,
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

    fn read_accel_registers<B>(&mut self, reg: accel::Register) -> Result<B, E>
    where
        B: Unsize<[u8]>,
    {
        let mut buffer: B = unsafe { mem::uninitialized() };

        {
            let buffer: &mut [u8] = &mut buffer;

            const MULTI: u8 = 1 << 7;
            self.i2c
                .write_read(accel::ADDRESS, &[reg.addr() | MULTI], buffer)?;
        }

        Ok(buffer)
    }

    fn read_mag_register(&mut self, reg: mag::Register) -> Result<u8, E> {
        let buffer: [u8; 1] = self.read_mag_registers(reg)?;
        Ok(buffer[0])
    }

    // NOTE has weird address increment semantics; use only with `OUT_X_H_M`
    fn read_mag_registers<B>(&mut self, reg: mag::Register) -> Result<B, E>
    where
        B: Unsize<[u8]>,
    {
        let mut buffer: B = unsafe { mem::uninitialized() };

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
