//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    let mut mini_stride: usize;
    let first = TASK_MANAGER.exclusive_access().fetch();
    let mut ready_que: Vec<Arc<TaskControlBlock>> = Vec::new();

    if let Some(task) = first {
        let inner = task.clone();
        mini_stride = inner.inner_exclusive_access().stride;
        ready_que.push(task);
    }else{
        return None;
    }

    while let Some(task) = TASK_MANAGER.exclusive_access().fetch() {
        let inner = task.clone();
        let stride = inner.inner_exclusive_access().stride;
        if stride < mini_stride {
            mini_stride = stride;
        }
        ready_que.push(task);
    }


    let mut idx = 0;
    for (i,task) in ready_que.iter_mut().enumerate() {
        let mut inner = task.inner_exclusive_access();
        if inner.stride == mini_stride {
            idx = i;
            inner.stride += inner.pass;
            break;
        }
    }

    for (i,task) in ready_que.iter().enumerate() {
        if i != idx {
            TASK_MANAGER.exclusive_access().add(task.clone());
        }
    }

    let task = ready_que[idx].clone();
    Some(task)
}
