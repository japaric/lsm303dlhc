pub const ADDRESS: u8 = 0b0011110;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Register {
    CRA_REG_M = 0x00,
    CRB_REG_M = 0x01,
    MR_REG_M = 0x02,
    OUT_X_H_M = 0x03,
    OUT_X_L_M = 0x04,
    OUT_Z_H_M = 0x05,
    OUT_Z_L_M = 0x06,
    OUT_Y_H_M = 0x07,
    OUT_Y_L_M = 0x08,
    SR_REG_M = 0x09,
    IRA_REG_M = 0x0A,
    IRB_REG_M = 0x0B,
    IRC_REG_M = 0x0C,
    TEMP_OUT_H_M = 0x31,
    TEMP_OUT_L_M = 0x32,
}

impl Register {
    pub fn addr(&self) -> u8 {
        *self as u8
    }
}
