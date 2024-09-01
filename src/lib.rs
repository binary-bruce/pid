#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use lazy_static::*;
use up_safe_cell::UPSafeCell;

lazy_static! {
    pub static ref PID_ALLOCATOR: UPSafeCell<PidAllocator> =
        unsafe { UPSafeCell::new(PidAllocator::new()) };
}

pub struct PidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl PidAllocator {
    pub fn new() -> Self {
        Self {
            current: 0,
            recycled: Vec::new(),
        }
    }

    pub fn alloc(&mut self) -> PidHandle {
        if let Some(pid) = self.recycled.pop() {
            PidHandle(pid)
        } else {
            self.current += 1;
            PidHandle(self.current - 1)
        }
    }

    fn dealloc(&mut self, pid: usize) {
        assert!(pid < self.current);
        assert!(
            !self.recycled.iter().any(|ppid| *ppid == pid),
            "pid {} has been deallocated!",
            pid
        );

        self.recycled.push(pid);
    }
}

pub struct PidHandle(pub usize);

impl Drop for PidHandle {
    fn drop(&mut self) {
        //println!("drop pid {}", self.0);
        PID_ALLOCATOR.exclusive_access().dealloc(self.0);
    }
}

pub fn pid_alloc() -> PidHandle {
    PID_ALLOCATOR.exclusive_access().alloc()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_alloc_and_dealloc() {
        let pid = pid_alloc();
        assert_eq!(0, pid.0);

        {
            let pid = pid_alloc();
            assert_eq!(1, pid.0)
        }

        let pid = pid_alloc();
        assert_eq!(1, pid.0);

        let pid = pid_alloc();
        assert_eq!(2, pid.0);
    }
}
