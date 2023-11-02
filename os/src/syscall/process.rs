//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next,
        task_start_time,current_user_token,get_syscall_times,
        mmap,munmap,
        TaskStatus,
    },
    timer::{get_time_us},
    mm::{translated_ptr,VirtAddr},
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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
    let us = get_time_us();
    let _ts_phy_ptr:*mut TimeVal = translated_ptr(
        current_user_token(), 
        _ts);
    unsafe {
        *_ts_phy_ptr = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let _ti_phy_ptr: *mut TaskInfo = translated_ptr(
        current_user_token(), 
        _ti);
    unsafe{
        *_ti_phy_ptr = TaskInfo {
            status: TaskStatus::Running,
            time:(get_time_us()-task_start_time()) /1000,
            syscall_times:get_syscall_times(),
        };
    }
    return 0;
}

// YOUR JOB: Implement mmap.
/// 构建键值对放到memoryset中，每个TaskControlBlock会有一个memoryset，
/// 所以可以在TaskManager中新增加一个函数用于操作memoryset,
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap"); 
    if _port & !0x7 !=0 || _port & 0b0000_0111 == 0 { 
        debug!("error _port:{:#x}",_port);
        return -1;
    }
    if !VirtAddr(_start).aligned() {
        debug!("error _start: {:#x}",_start);
        return -1;
    }
    if _len == 0 {
        return 0;
    }
    mmap(_start,_len,_port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if !VirtAddr(_start).aligned() {
        debug!("error _start");
        return -1;
    }
    if _len == 0 {
        return 0;
    }
    munmap(_start,_len)
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
