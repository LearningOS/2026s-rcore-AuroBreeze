//! Synchronization and interior mutability primitives

mod condvar;
mod mutex;
mod semaphore;
mod up;

pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::UPSafeCell;

use crate::task::{current_process, current_task};

/// check deadlock
pub fn deadlock_check(target_mutex_id: usize) -> bool {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();

    let n = process_inner.tasks.len();
    let m = process_inner.mutex_list.len();

    if target_mutex_id >= m || process_inner.mutex_list[target_mutex_id].is_none() {
        return true; 
    }
    let current_tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;

    if let Some(mutex) = process_inner.mutex_list[target_mutex_id].as_ref() {
        if mutex.is_locked() && mutex.get_owner() == current_tid as isize {
            return true;
        }
    }   
    let mut available = alloc::vec![0; m];
    let mut allocation = alloc::vec![alloc::vec![0; m]; n];
    let mut request = alloc::vec![alloc::vec![0; m]; n];

    for (i, mutex_opt) in process_inner.mutex_list.iter().enumerate() {
        if let Some(mutex) = mutex_opt {
            if !mutex.is_locked() {
                available[i] = 1;
            } else {
                available[i] = 0;
                
                let owner_tid = mutex.get_owner();
                if owner_tid >= 0 && (owner_tid as usize) < n {
                    allocation[owner_tid as usize][i] = 1;
                }
            }

            for tid in 0..n {
                if process_inner.tasks[tid].is_some() && mutex.is_waiting(tid) {
                    request[tid][i] = 1;
                }
            }
        }
    }

    let current_tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
        
    request[current_tid][target_mutex_id] = 1;

    let mut work = available;
    let mut finish = alloc::vec![false; n];

    for i in 0..n {
        if process_inner.tasks[i].is_none() {
            finish[i] = true;
        } else {
            let is_requesting = request[i].iter().any(|&req| req > 0);
            if !is_requesting {
                finish[i] = true;
                for j in 0..m {
                    work[j] += allocation[i][j];
                }
            }
        }
    }

    drop(process_inner);

    loop {
        let mut found = false;
        for i in 0..n {
            if !finish[i] {
                let mut can_satisfy = true;
                for j in 0..m {
                    if request[i][j] > work[j] {
                        can_satisfy = false;
                        break;
                    }
                }

                if can_satisfy {
                    for j in 0..m {
                        work[j] += allocation[i][j];
                    }
                    finish[i] = true;
                    found = true;
                }
            }
        }
        if !found {
            break;
        }
    }

    !finish.iter().all(|&f| f)
}

/// check semaphore deadlock
pub fn deadlock_check_semaphore(target_semaphore_id: usize) -> bool {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();

    let n = process_inner.tasks.len();
    // Assumes `semaphore_list` exists in your process control block
    let m = process_inner.semaphore_list.len(); 

    if target_semaphore_id >= m || process_inner.semaphore_list[target_semaphore_id].is_none() {
        return true;
    }

    let mut available = alloc::vec![0; m];
    let mut allocation = alloc::vec![alloc::vec![0; m]; n];
    let mut request = alloc::vec![alloc::vec![0; m]; n];

    for (i, sem_opt) in process_inner.semaphore_list.iter().enumerate() {
        if let Some(sem) = sem_opt {
            available[i] = sem.available_count();

            // If the semaphore is fully acquired (count <= 0)
            if sem.available_count() == 0 {
                let owner_tid = sem.get_owner();
                if owner_tid >= 0 && (owner_tid as usize) < n {
                    // Mark 1 resource allocated to the owner
                    allocation[owner_tid as usize][i] = 1; 
                }
            }

            for tid in 0..n {
                if process_inner.tasks[tid].is_some() && sem.is_waiting(tid) {
                    request[tid][i] = 1;
                }
            }
        }
    }

    let current_tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
        
    request[current_tid][target_semaphore_id] = 1;

    let mut work = available;
    let mut finish = alloc::vec![false; n];

    for i in 0..n {
        if process_inner.tasks[i].is_none() {
            finish[i] = true;
        } else {
            let is_requesting = request[i].iter().any(|&req| req > 0);
            if !is_requesting {
                finish[i] = true;
                for j in 0..m {
                    work[j] += allocation[i][j];
                }
            }
        }
    }

    drop(process_inner); // Drop lock before spinning

    loop {
        let mut found = false;
        for i in 0..n {
            if !finish[i] {
                let mut can_satisfy = true;
                for j in 0..m {
                    if request[i][j] > work[j] {
                        can_satisfy = false;
                        break;
                    }
                }

                if can_satisfy {
                    for j in 0..m {
                        work[j] += allocation[i][j];
                    }
                    finish[i] = true;
                    found = true;
                }
            }
        }
        if !found {
            break;
        }
    }

    !finish.iter().all(|&f| f)
}
