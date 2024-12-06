use alloc::collections::BinaryHeap;
use core::{cmp::Reverse, time::Duration};
use derive_where::derive_where;
use spin::Mutex;

use super::{scheduler::SCHEDULER, thread::WeakSharedThread};
use crate::driver::hpet::HPET;

pub static TIMER: Mutex<Timer> = Mutex::new(Timer::default());

pub struct Timer(BinaryHeap<TimerInfo>);

#[derive_where(PartialOrd, Ord, PartialEq, Eq)]
struct TimerInfo(Reverse<u64>, #[derive_where(skip)] WeakSharedThread);

impl Timer {
    pub const fn default() -> Self {
        Self(BinaryHeap::new())
    }

    pub fn add(&mut self, duration: Duration) {
        let target_tick = HPET.estimate(duration);

        let current_thread = SCHEDULER.lock().current_thread();
        self.0.push(TimerInfo(Reverse(target_tick), current_thread));

        let TimerInfo(Reverse(target_tick), _) = self.0.peek().unwrap();
        HPET.set_timer(*target_tick);
    }

    pub fn wakeup(&mut self) {
        if let Some(TimerInfo(_, thread)) = self.0.pop() {
            if thread.upgrade().is_some() {
                SCHEDULER.lock().add(thread);
            }
        }
    }
}
