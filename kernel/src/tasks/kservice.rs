use crate::syscall::r#yield;
use alloc::boxed::Box;
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, Waker};
use core::{future::Future, pin::Pin};
use crossbeam_queue::ArrayQueue;

pub fn kservice_thread() {
    let mut executor = Executor::default();
    executor.run();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ServiceId(u64);

impl ServiceId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ServiceId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Service {
    id: ServiceId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Service {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            id: ServiceId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

struct ServiceWaker {
    id: ServiceId,
    queue: Arc<ArrayQueue<ServiceId>>,
}

impl ServiceWaker {
    fn wake_service(&self) {
        self.queue.push(self.id).expect("Queue full");
    }
}

impl Wake for ServiceWaker {
    fn wake(self: Arc<Self>) {
        self.wake_service();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_service();
    }
}

pub struct Executor {
    services: BTreeMap<ServiceId, Service>,
    queue: Arc<ArrayQueue<ServiceId>>,
    waker_cache: BTreeMap<ServiceId, Waker>,
}

impl Default for Executor {
    fn default() -> Self {
        Executor {
            services: BTreeMap::new(),
            queue: Arc::new(ArrayQueue::new(128)),
            waker_cache: BTreeMap::new(),
        }
    }
}

impl Executor {
    pub fn spawn(&mut self, service: Service) {
        let id = service.id;
        if self.services.insert(id, service).is_some() {
            panic!("Service with same ID already in services");
        }
        self.queue.push(id).expect("Queue full");
    }

    pub fn run(&mut self) -> ! {
        loop {
            while let Some(id) = self.queue.pop() {
                let service = match self.services.get_mut(&id) {
                    Some(service) => service,
                    None => continue,
                };

                let waker = self.waker_cache.entry(id).or_insert_with(|| {
                    Waker::from(Arc::new(ServiceWaker {
                        id,
                        queue: self.queue.clone(),
                    }))
                });

                let mut context = Context::from_waker(waker);

                if let Poll::Ready(()) = service.poll(&mut context) {
                    self.services.remove(&id);
                    self.waker_cache.remove(&id);
                }
            }

            self.queue.is_empty().then(r#yield);
        }
    }
}
