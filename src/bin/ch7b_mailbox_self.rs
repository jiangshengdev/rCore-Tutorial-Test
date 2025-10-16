#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{getpid, mail_read, mail_write, yield_};

const MAILBOX_CAPACITY: usize = 16;

fn recv_probe(buf: &mut [u8]) -> isize {
    loop {
        let len = mail_read(buf);
        if len >= 0 {
            return len;
        }
        yield_();
    }
}

fn main_impl() -> i32 {
    let self_pid = getpid() as usize;
    println!("[self] pid = {}", self_pid);

    let mut empty: [u8; 0] = [];
    assert_eq!(mail_read(&mut empty), -1);
    println!("[self] empty read probe returned -1");

    for i in 0..MAILBOX_CAPACITY {
        let payload = [i as u8];
        assert_eq!(mail_write(self_pid, &payload), 1);
        println!("[self] enqueued message {}", payload[0]);
    }

    assert_eq!(mail_write(self_pid, &[0xff]), -1);
    println!("[self] write rejected when mailbox full");
    assert_eq!(mail_write(self_pid, &empty), -1);
    println!("[self] zero-length write rejected when full");
    assert_eq!(mail_read(&mut empty), 0);
    println!("[self] zero-length read reports mailbox not empty");

    let mut single = [0u8; 1];
    assert_eq!(recv_probe(&mut single), 1);
    assert_eq!(single[0], 0);
    println!("[self] first payload dequeued = {}", single[0]);

    assert_eq!(mail_write(self_pid, &empty), 0);
    println!("[self] zero-length write succeeds after dequeue");

    for expected in 1..MAILBOX_CAPACITY {
        assert_eq!(recv_probe(&mut single), 1);
        assert_eq!(single[0], expected as u8);
        println!("[self] dequeued payload {}", single[0]);
    }

    assert_eq!(mail_read(&mut single), -1);
    assert_eq!(mail_read(&mut empty), -1);
    println!("[self] mailbox drained, further reads fail");

    println!("\x1b[32mch7b_mailbox_self passed\x1b[0m");
    0
}

#[no_mangle]
pub fn main() -> i32 {
    main_impl()
}
