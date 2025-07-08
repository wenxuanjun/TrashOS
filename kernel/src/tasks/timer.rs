use alloc::collections::BinaryHeap;
use core::{cmp::Reverse, time::Duration};
use derive_where::derive_where;
use spin::Mutex;

use super::scheduler::SCHEDULER;
use super::thread::WeakSharedThread;
use crate::drivers::hpet::HPET;

pub static TIMER: Mutex<Timer> = Mutex::new(Timer::default());

#[derive(Default)]
pub struct Timer(BinaryHeap<TimerInfo>);

#[derive_where(PartialOrd, Ord, PartialEq, Eq)]
struct TimerInfo(Reverse<u64>, #[derive_where(skip)] WeakSharedThread);

impl Timer {
    const fn default() -> Self {
        Self(BinaryHeap::new())
    }

    fn update_timer(&mut self) {
        if let Some(timer_info) = self.0.peek() {
            let TimerInfo(Reverse(target_tick), _) = timer_info;
            HPET.set_timer(*target_tick);
        }
    }
}

impl Timer {
    pub fn add(&mut self, duration: Duration) {
        let target_tick = HPET.estimate(duration);
        let current_thread = SCHEDULER.lock().current();
        self.0.push(TimerInfo(Reverse(target_tick), current_thread));
        self.update_timer();
    }

    pub fn wakeup(&mut self) {
        if let Some(TimerInfo(_, thread)) = self.0.pop() {
            if thread.upgrade().is_some() {
                SCHEDULER.lock().add(thread);
                self.update_timer();
            }
        }
    }
}
