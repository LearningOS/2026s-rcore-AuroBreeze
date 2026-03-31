//! Semaphore

use crate::sync::UPSafeCell;
use crate::task::{block_current_and_run_next, current_task, wakeup_task, TaskControlBlock};
use alloc::{collections::VecDeque, sync::Arc};

/// semaphore structure
pub struct Semaphore {
    /// semaphore inner
    pub inner: UPSafeCell<SemaphoreInner>,
}

pub struct SemaphoreInner {
    pub count: isize,
    pub wait_queue: VecDeque<Arc<TaskControlBlock>>,
    pub owner: isize,
}

impl Semaphore {
    /// Create a new semaphore
    pub fn new(res_count: usize) -> Self {
        trace!("kernel: Semaphore::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(SemaphoreInner {
                    count: res_count as isize,
                    wait_queue: VecDeque::new(),
                    owner: -1,
                })
            },
        }
    }

    /// Get the owner of the semaphore
    pub fn get_owner(&self) -> isize {
        self.inner.exclusive_access().owner
    }

    /// Check if a specific task is waiting in the queue
    pub fn is_waiting(&self, tid: usize) -> bool {
        let inner = self.inner.exclusive_access();
        for task in inner.wait_queue.iter() {
            if task.inner_exclusive_access().res.as_ref().map(|r| r.tid).unwrap_or(0) == tid {
                return true;
            }
        }
        false
    }

    /// Get the currently available count (ensures we don't return negative values for the matrix)
    pub fn available_count(&self) -> usize {
        let count = self.inner.exclusive_access().count;
        if count > 0 {
            count as usize
        } else {
            0
        }
    }

    /// up operation of semaphore
    pub fn up(&self) {
    trace!("kernel: Semaphore::up");
    let mut inner = self.inner.exclusive_access();
    inner.count += 1;
    if inner.count <= 0 {
        if let Some(task) = inner.wait_queue.pop_front() {
            inner.owner = task.inner_exclusive_access().res.as_ref().unwrap().tid as isize;
            wakeup_task(task);
        }
    } else {
        inner.owner = -1;
    }
}

    /// down operation of semaphore
    pub fn down(&self) {
        trace!("kernel: Semaphore::down");
        let mut inner = self.inner.exclusive_access();
        inner.count -= 1;
        if inner.count < 0 {
            inner.wait_queue.push_back(current_task().unwrap());
            drop(inner);
            block_current_and_run_next();
        }else {
            inner.owner = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid as isize;
        }
    }
}













