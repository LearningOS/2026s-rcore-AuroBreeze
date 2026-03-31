//! Mutex (spin-like and blocking(sleep))

use super::UPSafeCell;
use crate::task::TaskControlBlock;
use crate::task::{block_current_and_run_next, suspend_current_and_run_next};
use crate::task::{current_task, wakeup_task};
use alloc::vec::Vec;
use alloc::{collections::VecDeque, sync::Arc};

/// Mutex trait
pub trait Mutex: Sync + Send {
    /// Lock the mutex
    fn lock(&self);
    /// Unlock the mutex
    fn unlock(&self);

    /// Check if the mutex is locked
    fn is_locked(&self) -> bool;

    /// Get the owner of the mutex
    fn get_owner(&self) -> isize;

    /// Check if the mutex is waiting
    fn is_waiting(&self, _tid: usize) -> bool;
}

/// Spinlock Mutex struct
pub struct MutexSpin {
    inner: UPSafeCell<MutexSpinInner>,
}

pub struct MutexSpinInner {
    owner: isize,
    locked: bool,
    wait_tids: Vec<usize>,
}

impl MutexSpin {
    /// Create a new spinlock mutex
    pub fn new() -> Self {
        Self {
            inner: unsafe {
                UPSafeCell::new(MutexSpinInner {
                    owner: -1,
                    locked: false,
                    wait_tids: Vec::new(),
                })
            },
        }
    }
}

impl Mutex for MutexSpin {
    fn is_waiting(&self, _tid: usize) -> bool {
        self.inner.exclusive_access().wait_tids.contains(&_tid)
    }
    /// Get the owner of the mutex
    fn get_owner(&self) -> isize {
        self.inner.exclusive_access().owner
    }
    /// Check if the mutex is locked
    fn is_locked(&self) -> bool {
        self.inner.exclusive_access().locked
    }
    /// Lock the spinlock mutex
    fn lock(&self) {
        trace!("kernel: MutexSpin::lock");

        let current_tid = current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid;

        self.inner.exclusive_access().wait_tids.push(current_tid);

        loop {
            let mut inner = self.inner.exclusive_access();
            if inner.locked {
                drop(inner);
                suspend_current_and_run_next();
                continue;
            } else {
                inner.locked = true;
                inner.owner = current_tid as isize;
                inner.wait_tids.retain(|&t| t != current_tid);
                return;
            }
        }
    }

    fn unlock(&self) {
        trace!("kernel: MutexSpin::unlock");
        let mut inner = self.inner.exclusive_access();
        inner.locked = false;
        inner.owner = -1; 
    }
}

/// Blocking Mutex struct
pub struct MutexBlocking {
    inner: UPSafeCell<MutexBlockingInner>,
}

pub struct MutexBlockingInner {
    locked: bool,
    wait_queue: VecDeque<Arc<TaskControlBlock>>,
    owner: isize, // tid
}

impl MutexBlocking {
    /// Create a new blocking mutex
    pub fn new() -> Self {
        trace!("kernel: MutexBlocking::new");
        Self {
            inner: unsafe {
                UPSafeCell::new(MutexBlockingInner {
                    locked: false,
                    wait_queue: VecDeque::new(),
                    owner: -1,
                })
            },
        }
    }
}

impl Mutex for MutexBlocking {
    /// Check if the mutex is waiting
    fn is_waiting(&self, _tid: usize) -> bool {
        let mutex_inner = self.inner.exclusive_access();
        for task in mutex_inner.wait_queue.iter() {
            if task
                .inner_exclusive_access()
                .res
                .as_ref()
                .map(|r| r.tid)
                .unwrap_or(0)
                == _tid
            {
                return true;
            }
        }
        false
    }

    /// Check if the mutex is locked
    fn get_owner(&self) -> isize {
        self.inner.exclusive_access().owner
    }
    /// Check if the mutex is locked
    fn is_locked(&self) -> bool {
        self.inner.exclusive_access().locked
    }
    /// lock the blocking mutex
    fn lock(&self) {
        trace!("kernel: MutexBlocking::lock");
        let mut mutex_inner = self.inner.exclusive_access();
        if mutex_inner.locked {
            mutex_inner.wait_queue.push_back(current_task().unwrap());
            drop(mutex_inner);
            block_current_and_run_next();
        } else {
            mutex_inner.owner = current_task()
                .unwrap()
                .inner_exclusive_access()
                .res
                .as_ref()
                .unwrap()
                .tid as isize;
            mutex_inner.locked = true;
        }
    }

    /// unlock the blocking mutex
    fn unlock(&self) {
        trace!("kernel: MutexBlocking::unlock");
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            mutex_inner.owner = waking_task
                .inner_exclusive_access()
                .res
                .as_ref()
                .unwrap()
                .tid as isize;
            wakeup_task(waking_task);
        } else {
            mutex_inner.owner = -1;
            mutex_inner.locked = false;
        }
    }
}
