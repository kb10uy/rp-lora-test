use std::{thread::sleep, time::Duration};

use anyhow::Result;
use libftd2xx::{BitMode, BitsPerWord, Ftdi, FtdiCommon, Parity, StopBits};
use rand::prelude::*;
use time::{macros::format_description, OffsetDateTime};

const FT_SERIAL_NUMBER: &str = "DQ0005AE";

fn main() -> Result<()> {
    let mut ftdi = Ftdi::with_serial_number(FT_SERIAL_NUMBER)?;
    ftdi.set_baud_rate(9600)?;
    ftdi.set_data_characteristics(BitsPerWord::Bits8, StopBits::Bits1, Parity::No)?;
    println!("Device opened");

    // CBUS0/1 is output to M0/1, set to HH (configuration mode)
    // CBUS2 is input from AUX
    let mut cbus_mask_bits = 0b_0011_0011;
    ftdi.set_bit_mode(cbus_mask_bits, BitMode::CbusBitbang)?;
    println!("FTDI CBUS bitbang mode is set");

    // Read all register (9byte, response will be 12byte)
    let mut read_buffer = [0; 256];
    ftdi.write_all(&[0xC1, 0x00, 0x09])?;
    ftdi.read_all(&mut read_buffer[..12])?;

    println!("Parameters");
    let response = &read_buffer[3..12];
    for (index, value) in response.iter().enumerate() {
        println!("{index:02X}h => {value:02X}h");
    }

    cbus_mask_bits = 0b_0011_0000;
    ftdi.set_bit_mode(cbus_mask_bits, BitMode::CbusBitbang)?;

    let mut rng = thread_rng();
    let time_format = format_description!("[hour]:[minute]:[second]");

    loop {
        let now = OffsetDateTime::now_local()?;
        let random_value: u8 = rng.gen();
        let sending_text = format!("{} {random_value:02X}\n", now.format(time_format)?);
        println!("Sending \"{}\"", sending_text.trim());
        ftdi.write_all(sending_text.as_bytes())?;
        sleep(Duration::from_millis(500));
    }
}
