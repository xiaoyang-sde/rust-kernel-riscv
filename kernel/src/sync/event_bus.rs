use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use bitflags::bitflags;

use crate::sync::Mutex;

bitflags! {
  #[derive(Default, Copy, Clone)]
  pub struct Event: u32 {
    const CHILD_PROCESS_QUIT = 1 << 0;
  }
}

type EventCallback = Box<dyn Fn(Event) -> bool + Send>;

#[derive(Default)]
pub struct EventBus {
    event: Event,
    callback_list: Vec<EventCallback>,
}

impl EventBus {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }

    pub fn push(&mut self, event: Event) {
        self.event.set(event, true);
        for callback in &self.callback_list {
            callback(event);
        }
    }

    pub fn clear(&mut self, event: Event) {
        self.event.remove(event);
    }

    pub fn subscribe(&mut self, callback: EventCallback) {
        self.callback_list.push(callback);
    }
}

struct EventBusFuture {
    event_bus: Arc<Mutex<EventBus>>,
    subscribed_event: Event,
}

impl Future for EventBusFuture {
    type Output = Event;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Event> {
        let mut event_bus = self.event_bus.lock();
        if event_bus.event.contains(self.subscribed_event) {
            return Poll::Ready(event_bus.event);
        }

        let subscribed_event = self.subscribed_event;
        let waker = context.waker().clone();
        event_bus.subscribe(Box::new(move |event| {
            if event.contains(subscribed_event) {
                waker.wake_by_ref();
                true
            } else {
                false
            }
        }));
        Poll::Pending
    }
}

pub fn wait_for_event(
    event_bus: Arc<Mutex<EventBus>>,
    subscribed_event: Event,
) -> impl Future<Output = Event> {
    EventBusFuture {
        event_bus,
        subscribed_event,
    }
}
