#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

const TESTS: &[&str] = &[
    "ch7b_mailbox_self\0",
    "ch7b_mailbox_ipc\0",
    "ch7b_mailbox_limits\0",
];

use user_lib::{spawn, waitpid};

#[no_mangle]
pub fn main() -> i32 {
    for test in TESTS {
        println!("Mailbox tests: Running {}", test);
        let pid = spawn(test);
        let mut status: i32 = 0;
        let waited = waitpid(pid as usize, &mut status);
        assert_eq!(waited, pid);
        println!(
            "\x1b[32mMailbox tests: {} exited with code {}\x1b[0m",
            test, status
        );
    }
    println!("Mailbox aggregate passed!");
    0
}
