
mod drivers;
mod ui;
mod system;

use crate::drivers::display::DisplayManager;
use crate::ui::framework::ScreenManager;
use crate::ui::framework::Screen;
use crate::ui::screens::loading::LoadingScreen;
use crate::ui::screens::home::HomeScreen;
use crate::system::events::{EventQueue, ButtonEventSource, SystemTickSource};

use esp_idf_hal::{
    delay::FreeRtos,
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    prelude::*,
};
use esp_idf_svc::log::EspLogger;
use std::sync::Arc;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    EspLogger::initialize_default();
    log::info!("Starting visionHubOS");

    let peripherals = Peripherals::take()?;

    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio19;
    let i2c = peripherals.i2c0;
    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c_driver = I2cDriver::new(i2c, sda, scl, &config)?;

    let mut scroll_pin = PinDriver::input(peripherals.pins.gpio25)?;
    let mut select_pin = PinDriver::input(peripherals.pins.gpio26)?;
    scroll_pin.set_pull(esp_idf_hal::gpio::Pull::Up)?;
    select_pin.set_pull(esp_idf_hal::gpio::Pull::Up)?;

    let display_manager = Arc::new(DisplayManager::new(i2c_driver)?);

    let event_queue = Arc::new(EventQueue::new());

    let mut scroll_button_source = ButtonEventSource::new(scroll_pin, 25, event_queue.clone());
    let mut select_button_source = ButtonEventSource::new(select_pin, 26, event_queue.clone());

    let mut loading_screen = LoadingScreen::new(display_manager.clone(), "visionHubOS", "Booting...");

    let mut screen_manager = ScreenManager::new(display_manager.clone(), event_queue.get_queue_clone());

    screen_manager.add_screen(loading_screen);

    screen_manager.switch_to_screen(0)?;

    for i in 0..=100 {
         let message = match i {
            0..=20 => "Initialising hardware...",
            21..=40 => "Loading drivers...",
            41..=60 => "Starting services...",
            61..=80 => "Initialising UI...",
            _ => "Almost ready...",
        };

        if let Some(screen) = screen_manager.get_screen_as_mut::<LoadingScreen>() {
            screen.set_message(message);
            screen.set_progress(i);
            screen.draw()?;
        }

        FreeRtos::delay_ms(30);
    }
    
    let home_screen = HomeScreen::new(display_manager.clone());
    screen_manager.add_screen(home_screen);

    screen_manager.switch_to_screen(1)?;

    loop {
        scroll_button_source.poll();
        select_button_source.poll();

        screen_manager.process_events()?;

        FreeRtos::delay_ms(10);
    }
}
