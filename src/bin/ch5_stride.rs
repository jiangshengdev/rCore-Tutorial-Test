#![no_std]
#![no_main]

extern crate user_lib;

static TESTS: &[&str] = &[
    "ch5_stride0\0",
    "ch5_stride1\0",
    "ch5_stride2\0",
    "ch5_stride3\0",
    "ch5_stride4\0",
    "ch5_stride5\0",
];

use user_lib::{set_priority, spawn, waitpid};

#[no_mangle]
pub fn main() -> i32 {
    let mut pid = [0; 6];
    for (slot, &test) in pid.iter_mut().zip(TESTS.iter()) {
        *slot = spawn(test);
    }
    set_priority(4);
    for &child in &pid {
        let mut xstate: i32 = Default::default();
        let wait_pid = waitpid(child as usize, &mut xstate);
        assert_eq!(child, wait_pid);
    }
    0
}
