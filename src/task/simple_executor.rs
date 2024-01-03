use alloc::collections::VecDeque;
use core::task::{Context, Poll, Waker};

use super::Task;

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> Self {
        Self {
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = Waker::noop();
            let mut cx = Context::from_waker(&waker);
            match task.poll(&mut cx) {
                Poll::Ready(()) => {}
                Poll::Pending => {
                    self.task_queue.push_back(task);
                }
            }
        }
    }
}
