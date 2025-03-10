use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};

use esp_idf_hal::{
    i2c::{I2cConfig, I2cDriver},
    prelude::*,
};

use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum DisplayError {
    DriverError,
    DrawError,
    I2CError(esp_idf_hal::i2c::I2CError),
}

impl From<esp_idf_hal::i2c::I2cError> for DisplayError {
    fn from(error: esp_idf_hal::i2c::I2cError) -> Self {
        DisplayError::I2CError(error)
    }
}

impl From<display_interface::DisplayError> for DisplayError {
    fn from(_: display_interface::DisplayError) -> Self {
        DisplayError::DrawError
    }
}

pub struct DisplayManager {
    display: Arc<Mutex<Ssd1306<I2CInterface<I2cDriver<'static>>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>>>,
}

impl DisplayManager {
    pub fn new(i2c: I2cDrive<'static>) -> Result<Self, DisplayError> {
        let interface = I2CDisplayInterface::new(i2c);

        let mut display = Ssd1306::new(
            interface,
            DisplaySize128x64,
            DisplayRotation::Rotate180,
        )
        .into_buffered_graphics_mode();

        display.init().map_err(|_| DisplayError::DriverError)?;
        display.clear(BinaryColor::Off).map_err(|_| DisplayError::DrawError)?;

        Ok(Self {
            display: Arc::new(Mutex::new(display)),
        })
    }

    pub fn clear(&self) -> Result<(), DisplayError> {
        let mut display = self.display.lock().unwrap();
        display.clear(BinaryColor::Off).map_err(|_| DisplayError::DrawError)?;
        Ok(())
    }

    pub fn flush(&self) -> Result<(), DisplayError> {
        let mut display = self.display.lock().unwrap();
        display.flush().map_err(|_| DisplayError::DrawError)?;
        Ok(())
    }

    pub fn draw_text(&self, text:&str, x: i32, y: i32, size: TextSize) -> Result<(), DisplayError> {
        let mut display = self.display.lock().unwrap();

        let font = match size {
            TextSize::Small => &FONT_6X10,
            TextSize::Normal => &FONT_6X10,
            TextSize::Large => &FONT_6X10,
        };

        let text_style = MonoTextStyleBuilder::new()
            .font(font)
            .text_color(BinaryColor::On)
            .build();

        Text::with_baseline(
            text,
            Point::new(x, y),
            text_style,
            Baseline::Top,
        )
        .draw(&mut *display)
        .map_err(|_| DisplayError::DrawError)?;

        Ok(())
    }

    pub fn draw_rectangle(&self, x: i32, y:i32, width: u32, height: u32, filled: bool) -> Result<(), DisplayError> {
        let mut display = self.display.lock().unwrap();

        let rect = Rectange::new(
            Point::new(x, y),
            Size::new(widht, height),
        );

        if filled {
            rect.into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                .draw(&mut display)
                .map_err(|_| DisplayError::DrawError)?;
        } else {
            rect.into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                .draw(&mut display)
                .map_err(|_| DisplayError::DrawError)?;
        }

        Ok(())
    }

    pub fn draw_progress_bar(&self, x: i32, y: i32, width: u32, progress: u8) -> Result<(), DisplayError> {
        let height = 8u32;
        let progress = progress.min(100) as u32;
        let fill_width = (width*progress)/100;

        self.draw_rectangle(x, y, width, height, false)?;

        if fill_width > 0 {
            self.draw_rectangle(x + 1, y + 1, fill_width.saturating_sub(2), height.saturating_sub(2), true)?;
        }

        Ok(())
    }

    pub fn get_display_clone(&self) -> Arc<Mutex<Ssd1306<I2CInterface<I2cDriver<'static>>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>>> {
        self.display.clone()
    }

}

pub enum TextSize {
    Small,
    Normal,
    Large
}
