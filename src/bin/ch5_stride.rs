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
    for (i, test) in TESTS.iter().enumerate() {
        pid[i] = spawn(test);
    }
    set_priority(4);
    for &p in &pid {
        let mut xstate: i32 = Default::default();
        let wait_pid = waitpid(p as usize, &mut xstate);
        assert_eq!(p, wait_pid);
    }
    0
}
