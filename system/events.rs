use esp_idf_hal::gpio::{Pin, PinDriver};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub enum Event {
    ButtonPressed(u32),
    ButtonReleased(u32),
    Timer(u32),
    SystemTick,
    AppLaunched(String),
    AppClosed(String),
    Custom(String),
}

pub struct EventQueue {
    queue: Arc<Mutex<VecDeque<Event>>>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn push(&self, event: Event) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(event);
    }

    pub fn pop(&self) -> Option(Event) {
        let mut queue = self.queue.lock().unwrap();
        queue.pop_front()
    }

    pub fn get_queue_clone(&self) -> Arc<Mutex<VecDeque<Event>>> {
        self.queue.clone()
    }
}

pub trait EventHandler {
    fn handle_event(&mut self, event: &Event) -> bool;
}

pub struct ButtonEventSource<'a, T: Pin> {
    pin: PinDriver<'a, T, esp_idf_hal::gpio::Input>,
    pin_number: u32,
    event_queue: Arc<EventQueue>,
    last_state: bool,
    debounce_time: Duration,
    last_event: Instant,
}

impl <'a, T: Pin> ButtonEventSource<'a, T> {
    pub fn new(
        pin: PinDriver<'a, T, esp_idf_hal::gpio::Input>,
        pin_number: u32,
        event_queue: Arc<EventQueue>,
    ) -> Self {
        Self {
            pin,
            pin_number,
            event_queue,
            last_state: true,
            debounce_time: Duration::from_millis(50),
            last_evemt: Instant::now(),
        }
    }


    pub fn poll(&mut self) {
        let current_state = self.pin.is_high();
        let now = Instant::now();

        if now.duraction_since(self.last_event) >= self.debounce_time {
            if current_state != self.last_state {

                if current_state {
                    self.event_queue.push(Event::ButtonReleased(self.pin_number));
                }else{
                    self.event_queue.push(Event::ButtonPressed(self.pin_number));
                }

                self.last_state = current_state;
                self.last_event = now;

            }
        }
    }
}

pub struct TimerEventSource {
    timer_id: u32,
    event_queue: Arc<EventQueue>,
    interval: Duration,
    last_triggered: Instant,
}

impl TimerEventSource {
    pub fn new(
        timer_id: u32,
        interval: Duration,
        event_queue: Arc<EventQueue>,
    ) -> Self {
        Self {
            timer_id,
            event_queue,
            interval, 
            last_triggered: Instant::now(),
        }
    }

    pub fn poll(&mut self) {
        let now = Instant::now();

        if now.duraction_since(self.last_triggered) >= self.interval {
            self.event_queue.push(Event::Timer(self.timer_id));
            self.last_triggered = now;
        }
    }
}

pub struct SystemTickSource {
    event_queue: Arc<EventQueue>,
    interval: Duration,
    last_triggered: Instant,
}

impl SystemTickSource {
    pub fn new(
        interval: Duration,
        event_queue: Arc<EventQueue>,
    ) -> Self {
        Self {
            event_queue,
            interval,
            last_triggered: Instant::now(),
        }
    }

    pub fn poll(&mut self) {
        let now = Instant::now();

        if now.duration_since(self.last_triggered) >= self.interval {
            self.event_queue.push(Event::SystemTick);
            self.last_triggered = now;
        }
    }
}
