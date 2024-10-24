//!Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.

use super::__switch;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use crate::config::MAX_SYSCALL_NUM;
use crate::mm::{is_va_space_mapped, translated_byte_buffer, MapPermission, PTEFlags, VirtAddr};
use crate::sync::UPSafeCell;
use crate::timer::get_time_ms;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;

/// Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,

    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            if task_inner.start_time_ms == 0 {
                task_inner.start_time_ms = get_time_ms();
            }
            // release coming task_inner manually
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            trace!("switch to pid: {}", processor.current.as_ref().unwrap().pid.0);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            warn!("no tasks available in run_tasks");
        }
    }
}

/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get the current user token(addr of page table)
pub fn current_user_token() -> usize {
    // trace!("current_user_token");
    let task = current_task().unwrap();
    task.get_user_token()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    // trace!("current_trap_cx");
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}

/// Copy data to current user space
/// dst: the destination address in user space
/// src: the source address in kernel space
/// len: the length of data
pub fn copy_to_current_user(dst: *mut u8, src: *const u8, len: usize) {
    let token = current_user_token();
    let src_bytes: &[u8] = unsafe { core::slice::from_raw_parts(src, len) };
    let buffers = translated_byte_buffer(token, dst as *const u8, len);
    let mut offset = 0;
    for buf in buffers {
        let buf_len = buf.len();
        buf.copy_from_slice(&src_bytes[offset..offset + buf_len]);
        offset += buf_len;
    }
}

/// Add the number of syscalls
pub fn add_task_syscall_times(syscall_id: usize) {
    current_task()
        .unwrap()
        .add_syscall_times(syscall_id);
}

/// Get the number of syscalls
pub fn get_task_syscall_times() -> [u32; MAX_SYSCALL_NUM] {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .syscall_times
}

/// Get the start time of the task
pub fn get_task_start_time_ms() -> usize {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .start_time_ms
}

/// map a memory area in user space
/// start: the start address of the memory area
/// len: the length of the memory area
/// port: the attribute of the memory area
pub fn mmap(start: usize, len: usize, port: usize) -> isize {
    if port & !0x7 != 0 || port & 0x7 == 0 {
        error!("invalid port: {:?}", port);
        return -1;
    }
    let start_va: VirtAddr = VirtAddr::from(start);
    if !start_va.aligned() {
        error!("unaligned start address: {:?}", start);
        return -1;
    }
    if is_va_space_mapped(current_user_token(), start, len) {
        error!(
            "memory area already mapped: {:x?} {:x?}",
            start,
            start + len - 1
        );
        return -1;
    }
    let permission = MapPermission::from_bits((port << 1) as u8).unwrap() | MapPermission::U;
    let flags = PTEFlags::from_bits(permission.bits()).unwrap();

    current_task()
        .unwrap()
        .inner_exclusive_access()
        .memory_set
        .page_table
        .mmap(start, len, flags);
    0
}

/// unmap a memory area in user space
/// start: the start address of the memory area
/// len: the length of the memory area
/// return: 0 if success, -1 if failed
pub fn munmap(start: usize, len: usize) -> isize {
    let start_va: VirtAddr = VirtAddr::from(start);
    if !start_va.aligned() {
        error!("unaligned start address: {:?}", start);
        return -1;
    }
    if !is_va_space_mapped(current_user_token(), start, len) {
        error!(
            "memory area not mapped: {:x?} {:x?}",
            start,
            start + len - 1
        );
        return -1;
    }
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .memory_set
        .page_table
        .munmap(start, len)
}

/// Set the priority of the current task
pub fn set_task_priority(priority: isize) -> isize {
    if priority < 2 {
        return -1;
    }
    current_task()
        .unwrap()
        .set_priority(priority);
    priority
}