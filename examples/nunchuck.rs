// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.

// Reads data from Wii Nunchuck

extern crate i2cdev;
extern crate docopt;

use i2cdev::*;
use std::io::prelude::*;
use std::env::args;
use docopt::Docopt;
use std::thread;

const NUNCHUCK_SLAVE_ADDR: u16 = 0x52;

const USAGE: &'static str = "
Reading Wii Nunchuck data via Linux i2cdev.

Usage:
  nunchuck <device>
  nunchuck (-h | --help)
  nunchuck --version

Options:
  -h --help    Show this help text.
  --version    Show version.
";

#[derive(Debug)]
struct NunchuckReading {
    joystick_x: u8,
    joystick_y: u8,
    accel_x: u16,  // 10-bit
    accel_y: u16,  // 10-bit
    accel_z: u16,  // 10-bit
    c_button_pressed: bool,
    z_button_pressed: bool,
}

impl NunchuckReading {
    fn from_data(data: &[u8; 6]) -> NunchuckReading {
        NunchuckReading {
            joystick_x: data[0],
            joystick_y: data[1],
            accel_x: (data[2] as u16) << 2 | ((data[5] as u16 >> 6) & 0b11),
            accel_y: (data[3] as u16) << 2 | ((data[5] as u16 >> 4) & 0b11),
            accel_z: (data[4] as u16) << 2 | ((data[5] as u16 >> 2) & 0b11),
            c_button_pressed: (data[5] & 0b10) == 0,
            z_button_pressed: (data[5] & 0b01) == 0,
        }
    }
}


fn read_nunchuck_data(dev: &mut i2cdev::I2CDevice)
                      -> Result<(), &'static str> {
    // Set the address of the device we are trying to talk to
    try!(dev.set_slave_address(NUNCHUCK_SLAVE_ADDR)
         .or_else(|_| Err("Could not set slave address")));

    // init sequence.  TODO: figure out what this magic is
    try!(dev.write_all(&[0xF0, 0x55]).or_else(|_| Err("Writing init sequence 1 failed")));
    try!(dev.write_all(&[0xFB, 0x00]).or_else(|_| Err("Writing init sequence 2 failed")));
    thread::sleep_ms(100);

    loop {
        try!(dev.write_all(&[0x00]).or_else(|_| Err("Error preparing read")));
        thread::sleep_ms(10);

        let mut buf: [u8; 6] = [0; 6];
        try!(match dev.read(&mut buf) {
            Ok(_) => {
                let reading = NunchuckReading::from_data(&buf);
                println!("{:?}", reading);
                Ok(())
            }
            Err(_) => { Err("Error reading nunchuck data buffer") },
        });
    }
}

fn main() {
    let args = Docopt::new(USAGE)
        .and_then(|d| d.argv(args().into_iter()).parse())
        .unwrap_or_else(|e| e.exit());
    let device = args.get_str("<device>");
    match I2CDevice::new(device) {
        Ok(mut i2cdev) => { read_nunchuck_data(&mut i2cdev).unwrap() },
        Err(err) => { println!("Unable to open {:?}, {:?}", device, err); }
    }
}