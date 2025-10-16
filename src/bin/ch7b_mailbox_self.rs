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
    println!("[自身] 进程号 = {}", self_pid);

    let mut empty: [u8; 0] = [];
    assert_eq!(mail_read(&mut empty), -1);
    println!("[自身] 空读取检测返回 -1");

    for i in 0..MAILBOX_CAPACITY {
        let payload = [i as u8];
        assert_eq!(mail_write(self_pid, &payload), 1);
        println!("[自身] 入队消息 {}", payload[0]);
    }

    assert_eq!(mail_write(self_pid, &[0xff]), -1);
    println!("[自身] 邮箱已满，写入被拒绝");
    assert_eq!(mail_write(self_pid, &empty), -1);
    println!("[自身] 邮箱已满，零长度写入被拒绝");
    assert_eq!(mail_read(&mut empty), 0);
    println!("[自身] 零长度读取报告邮箱非空");

    let mut single = [0u8; 1];
    assert_eq!(recv_probe(&mut single), 1);
    assert_eq!(single[0], 0);
    println!("[自身] 首个负载出队 = {}", single[0]);

    assert_eq!(mail_write(self_pid, &empty), 0);
    println!("[自身] 出队后零长度写入成功");

    for expected in 1..MAILBOX_CAPACITY {
        assert_eq!(recv_probe(&mut single), 1);
        assert_eq!(single[0], expected as u8);
        println!("[自身] 出队负载 {}", single[0]);
    }

    assert_eq!(mail_read(&mut single), -1);
    assert_eq!(mail_read(&mut empty), -1);
    println!("[自身] 邮箱已清空，继续读取失败");

    println!("\x1b[32mch7b_mailbox_self 测试通过\x1b[0m");
    0
}

#[no_mangle]
pub fn main() -> i32 {
    main_impl()
}
