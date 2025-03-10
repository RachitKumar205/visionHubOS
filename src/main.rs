use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    prelude::*,
};
use esp_idf_sys as _;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

// Use a custom error wrapper to handle DisplayError
#[derive(Debug)]
enum AppError {
    DisplayError,
    EspError(esp_idf_sys::EspError),
    // Add other error types as needed
}

// Implement conversion from esp_idf_sys::EspError
impl From<esp_idf_sys::EspError> for AppError {
    fn from(error: esp_idf_sys::EspError) -> Self {
        AppError::EspError(error)
    }
}

// Implement conversion from DisplayError
impl From<display_interface::DisplayError> for  AppError {
    fn from(_: display_interface::DisplayError) -> Self {
        AppError::DisplayError
    }
}

fn main() -> Result<(), AppError> {
    // Initialize logger
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Starting ESP32 Server Remote");
    // Get peripherals
    let peripherals = Peripherals::take()?;
    
    // Configure I2C
    // Note: Adjust these pin numbers based on your wiring
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio19;
    let i2c = peripherals.i2c0;
    
    // Configure I2C with 400kHz clock
    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c_driver = I2cDriver::new(i2c, sda, scl, &config)?;
    
    // Configure display interface
    let interface = I2CDisplayInterface::new(i2c_driver);
    
    // Create display object with dimensions 128x64
    // If your display has different dimensions, adjust accordingly
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate180,
    )
    .into_buffered_graphics_mode();
    
    // Initialize display
    display.init()?;
    display.clear(BinaryColor::Off)?;
    
    // Configure button(s)
    // Example with a button connected to GPIO0
    let mut button = PinDriver::input(peripherals.pins.gpio26)?;
    button.set_pull(esp_idf_hal::gpio::Pull::Up)?;
    
    // Configure text style
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    
    // Draw initial text
    Text::with_baseline(
        "Services online",
        Point::new(0, 16),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)?;
    
    Text::with_baseline(
        "System ready to go",
        Point::new(0, 32),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)?;
    
    // Update display
    display.flush()?;
    
    // Main loop
    let mut counter = 0;
    loop {
        // Check button state
        if button.is_low() {
            // Button pressed
            counter += 1;
            
            // Clear display area for counter
            display.clear(BinaryColor::Off)?;
            
            // Redraw title
            Text::with_baseline(
                "ESP32 Server Remote",
                Point::new(0, 16),
                text_style,
                Baseline::Top,
            )
            .draw(&mut display)?;
            
            // Update counter text
            let counter_text = format!("Button presses: {}", counter);
            Text::with_baseline(
                &counter_text,
                Point::new(0, 32),
                text_style,
                Baseline::Top,
            )
            .draw(&mut display)?;
            
            // Update display
            display.flush()?;
            
            // Debounce
            FreeRtos::delay_ms(300);
        }
        
        // Small delay to prevent busy waiting
        FreeRtos::delay_ms(50);
    }
}
