use core::pin::Pin;
use core::task::{Context, Poll};

use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures::stream::{Stream, StreamExt};
use futures::task::AtomicWaker;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

use crate::{print, println};

// similar to `lazy_static`, but ensure one initialisation only;
// thus preventing heap allocation in interrupt handler
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by keyboard interrupt handler
///
/// Must not block or allocate
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialised");
    }
}

pub struct ScancodeStream {
    _private: (), // prevent external construction
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialised");

        // optimistic fast path to reduce overhead of registering a waker
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }
        // queue is potentially empty

        WAKER.register(cx.waker());

        match queue.pop() {
            Some(scancode) => {
                // branch to account for race condition where queue is populated after previous pop
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    // effectively an endless loop
    // `poll_next` never returns `Poll::Ready(None)` so sream never ends
    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
