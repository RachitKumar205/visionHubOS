use std::collections::BinaryHeap;
use std::cmp::{Ord, Ordering, PartialOrd};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use crate::system::events::{Event, EventQueue};

#[derive(Clone, Eq, PartialEq)]
struct ScheduledTask {
    id: u32,
    next_run: Instant,
    interval: Option<Duration>,
    callback: Arc<dyn Fn() + Send + Sync>,
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &self) -> Ordering {
        other.next_run.cmp(&self.next_run)
    }
}

impl PartialOrd for ScheduledTask {
    fm partial_cmp(&self, other: &self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Scheduler {
    tasks: Arc<Mutex<BinaryHeap<ScheduledTask>>>,
    next_id: u32,
    event_queue: Arc<EventQueue>,
}

impl Scheduler {
    pub fn new(event_queue: Arc<EventQueue>) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(BinaryHeap::new())),
            next_id: 0,
            event_queue,
        }
    }

    pub fn schedule_once<F>(&mut self, delay: Duration, callback: F) -> u32
    where
        F: Fn() + Send + Sync + 'static,
    {
        let task_id = self.next_id;
        self.next_id += 1;

        let task = ScheduledTask {
            id: task_id,
            next_run: Instant::now() + delay,
            interval: None,
            callback: Arc::new(callback),
        };

        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(task);

        task_id
    }

    pub fn schedule_recurring<F>(&mut self, delay: Duration, callback: F) -> u32
    where 
        F: Fn() + Send + Sync + 'static,
    {
        let task_id = self.next_id;
        self.next_id += 1;

        let task = ScheduledTask {
            id: task_id,
            next_run: Instant::now + delay,
            interval: Some(interval),
            callback: Arc::new(callback),
        };

        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(task);

        task_id
    }

    pub fn cancel_task(&mut self, id: u32) -> bool {
        let mut tasks = self.tasks.lock().unwrap();
        let old_len = tasks.len();

        let mut new_heap = BinaryHeap::new();
        for task in tasks.drain() {
            if task.id != id {
                new_heap.push(task);
            }
        }

        *tasks = new_heap;

        old_len != tasks.len()
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let mut tasks_to_reschedule = Vec::new();

        {
            let mut tasks = self.tasks.lock().unwrap();

            while let Some(task) = tasks.peek() {
                if task.next_run <= now {
                    let task = tasks.pop().unwrap();

                    (task.callback)();

                    self.event_queue.push(Event::Timer(task_id));

                    if let Some(interval) = task.interval {
                        tasks_to_reschedule.push(ScheduledTask {
                            id: task.id,
                            next_run: Instant::now + interval,
                            interval: Some(interval),
                            callback: task.callback,
                        });
                    }
                } else {
                    break;
                }
            }
        }
        
        if !tasks_to_reschedule.is_empty() {
            let mut tasks = self.tasks.lock().unwrap();
            for task in tasks_to_reschedule {
                tasks.push(task);
            }
        }
    }
}


