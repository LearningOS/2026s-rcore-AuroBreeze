//! Process management syscalls
use crate::{
    config::PAGE_SIZE,
    mm::{translated_byte_buffer, PTEFlags, PageTable, VirtPageNum},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_cnt_calls,
        suspend_current_and_run_next, TASK_MANAGER,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let time = get_time_us();
    let timeval = TimeVal {
        sec: time / 1000000,
        usec: time % 1000000,
    };

    let token = current_user_token();
    let mut buf = translated_byte_buffer(token, _ts as *const u8, core::mem::size_of::<TimeVal>());
    let time_val = unsafe {
        core::slice::from_raw_parts(
            &timeval as *const _ as *const u8,
            core::mem::size_of::<TimeVal>(),
        )
    };

    let mut current = 0;
    for buffer in buf.iter_mut() {
        let len = buffer.len();
        buffer.copy_from_slice(&time_val[current..current + len]);
        current += len;
    }

    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");

    if _trace_request == 2 {
        return get_cnt_calls(_id);
    }

    let pg = PageTable::from_token(current_user_token());
    let vpn = VirtPageNum::from(_id / PAGE_SIZE);
    let offset = _id % PAGE_SIZE;

    if let Some(pte) = pg.find_pte(vpn) {
        if pte.is_valid() && pte.flags().contains(PTEFlags::U){
            let ppn = pte.ppn();
            let bytes_array = ppn.get_bytes_array();

            match _trace_request {
                0 => {
                    if pte.readable() {
                        return bytes_array[offset] as isize;
                    } else {
                        return -1;
                    }
                }
                1 => {
                    if pte.writable() {
                        bytes_array[offset] = _data as u8;
                        return 0;
                    }else{
                        return -1;
                    }

                }
                _ => return -1,
            }
        }
    }
    -1
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _prot: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    let task = &mut inner.tasks[current];

    task.memory_set.mmap(_start, _len, _prot)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    let task = &mut inner.tasks[current];

    task.memory_set.munmap(_start, _len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
