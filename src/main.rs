extern crate i2cdev;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

use sysfs_pwm::Pwm;

use std::{thread, time};

fn main() {
    let mut dev = match ADCDevice::new(0x4b) {
        Ok(res) => res,
        Err(e) => panic!("Error {}", e),
    };
    let led_pwm = match Pwm::new(0, 0) {
        Ok(res) => res,
        Err(e) => panic!("PWM error {}", e),
    };

    match led_pwm.with_exported(|| {
        let _ = led_pwm.enable(true);
        let _ = led_pwm.set_period_ns(20_000);
        loop {
            let value = match dev.analog_read(0) {
                Ok(res) => res as u32,
                Err(e) => panic!("Cannot get value from i2c device {}", e),
            };
            match led_pwm.set_duty_cycle_ns(value * 100 / 255) {
                Err(e) => panic!("Cannot set PWM duty cycle {}", e),
                _ => (),
            };
            match led_pwm.get_duty_cycle_ns() {
                Err(e) => panic!("Cannot read duty cycle value {}", e),
                Ok(_) => (),
            }
            thread::sleep(time::Duration::from_millis(100));
        }
    }) {
        Ok(_) => (),
        Err(e) => panic!("Something went wrong {}", e),
    }
}

struct ADCDevice {
    cmd: u8,
    dev: LinuxI2CDevice,
}

impl ADCDevice {

    fn new(addr: u16) -> Result<Self, LinuxI2CError> {
        let dev = LinuxI2CDevice::new("/dev/i2c-1", addr)?;

        Ok(Self {
            cmd: 0x84,
            dev,
        })
    }

    fn analog_read(&mut self, chn: u8) -> Result<u8, LinuxI2CError> {
        self.dev.smbus_read_byte_data(self.cmd|(((chn<<2 | chn>>1)&0x07)<<4))
    }
}
