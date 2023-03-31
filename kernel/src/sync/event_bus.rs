//! The `event_bus` module provides an [EventBus] that supports publish-subscribe-style
//! communication between different tasks.

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
    /// The `Event` struct represents events that can be subscribed to on [EventBus].
  pub struct Event: u32 {
    /// Indicates that a child process has quit.
    const CHILD_PROCESS_QUIT = 1 << 0;
  }
}

type EventCallback = Box<dyn Fn(Event) -> bool + Send>;

/// The `EventBus` structsupports publish-subscribe-style communication between different tasks.
#[derive(Default)]
pub struct EventBus {
    event: Event,
    callback_list: Vec<EventCallback>,
}

impl EventBus {
    /// Creates a new event bus.
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }

    /// Publishes an event on the event bus.
    pub fn push(&mut self, event: Event) {
        self.event.set(event, true);
        for callback in &self.callback_list {
            callback(event);
        }
    }

    /// Clears an event from the event bus.
    pub fn clear(&mut self, event: Event) {
        self.event.remove(event);
    }

    /// Subscribes to events on the event bus and executes the given callback function when an event
    /// is published.
    pub fn subscribe(&mut self, callback: EventCallback) {
        self.callback_list.push(callback);
    }
}

/// The `EventBusFuture` struct is a future that completes when a specified event is published on
/// an [EventBus].
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

/// Returns a future that completes when a specified event is published on an [EventBus].
pub fn wait_for_event(
    event_bus: Arc<Mutex<EventBus>>,
    subscribed_event: Event,
) -> impl Future<Output = Event> {
    EventBusFuture {
        event_bus,
        subscribed_event,
    }
}
