use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::text::*;
use esp_idf_hal::delay;
use esp_idf_hal::gpio::{AnyIOPin, PinDriver, Pull};
use esp_idf_hal::i2c::*;
use esp_idf_hal::prelude::*;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use ssd1306;
use ssd1306::prelude::DisplayConfig;
use std::time::{Duration, SystemTime};

pub struct Board<I2C> {
    pub display: DisplayPeripheral<I2C>,
    pub button: ButtonPeripheral,
}

#[cfg(t_display_s3)]
impl Board<I2C0> {
    pub fn take() -> Self {
        let peripherals = Peripherals::take().unwrap();

        Board {
            display: DisplayPeripheral {
                scl: peripherals.pins.gpio2.into(),
                sda: peripherals.pins.gpio1.into(),
                i2c: peripherals.i2c0,
            },
            button: ButtonPeripheral {
                pin: peripherals.pins.gpio14.into(),
            },
        }
    }
}

#[cfg(roarbms)]
impl Board<I2C0> {
    pub fn take() -> Self {
        let peripherals = Peripherals::take().unwrap();

        Board {
            display: DisplayPeripheral {
                scl: peripherals.pins.gpio9.into(),
                sda: peripherals.pins.gpio8.into(),
                i2c: peripherals.i2c0,
            },
            button: ButtonPeripheral {
                pin: peripherals.pins.gpio3.into(),
            },
        }
    }
}

pub struct DisplayPeripheral<I2C> {
    pub scl: AnyIOPin,
    pub sda: AnyIOPin,
    pub i2c: I2C,
}

pub struct ButtonPeripheral {
    pub pin: AnyIOPin,
}

fn test_render() -> () {
    let board = Board::take();

    let mut button = PinDriver::input(board.button.pin).unwrap();
    button.set_pull(Pull::Up).unwrap();

    let driver = I2cDriver::new(
        board.display.i2c,
        board.display.sda,
        board.display.scl,
        &I2cConfig::new().baudrate(400.kHz().into()),
    )
    .unwrap();
    let di = ssd1306::I2CDisplayInterface::new(driver);

    let mut display = ssd1306::Ssd1306::new(
        di,
        ssd1306::size::DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    display.init().expect("error initializing display");

    display.clear();

    {
        let boot_logo_data = include_bytes!("../assets/roarpower_boot.bin"); // need to slice off first 4 bytes, CF_ALPHA_1_BIT -> Binary, https://lvgl.io/tools/imageconverter
        let image: ImageRaw<BinaryColor> = ImageRaw::new(&boot_logo_data[4..], 128);

        Image::new(&image, Point::zero())
            .draw(&mut display)
            .unwrap();
        display.flush().unwrap();
        delay::Ets::delay_ms(5000);
    }

    {
        for i in 0..100 {
            display.clear();
            Text::new(
                format!("Roar {}", i).as_str(),
                Point::new(10, i % (display.bounding_box().size.height as i32)),
                MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
            )
            .draw(&mut display)
            .unwrap();

            display.flush().unwrap();

            delay::Ets::delay_ms(1);
        }
    }

    let mut count = 1;
    let mut last_press: SystemTime = SystemTime::now();
    let debounce = Duration::from_millis(200);
    let mut dirty = true;

    loop {
        delay::Ets::delay_ms(1);
        let now = SystemTime::now();

        if button.is_low() && now.duration_since(last_press).unwrap() > debounce {
            count += 1;
            dirty = true;
            last_press = SystemTime::now();
        }

        if dirty {
            display.clear();
            Text::new(
                format!("Roar {}", count).as_str(),
                Point::new(0, 20),
                MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
            )
            .draw(&mut display)
            .unwrap();

            display.flush().unwrap();

            dirty = false;
        }
    }
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    println!("Hello, world!");

    test_render();
}
