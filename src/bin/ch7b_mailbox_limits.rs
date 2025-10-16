#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::mem::size_of;
use user_lib::{fork, getpid, mail_read, mail_write, waitpid, yield_};

const MAX_MSG_LEN: usize = 256;
const OVERFLOW_LEN: usize = MAX_MSG_LEN + 64;
const STAGE_HANDSHAKE: u8 = 0;
const STAGE_OVERFLOW: u8 = 1;

fn recv_blocking(buf: &mut [u8]) -> usize {
    loop {
        let len = mail_read(buf);
        if len >= 0 {
            return len as usize;
        }
        yield_();
    }
}

fn run_parent(child_pid: usize) {
    let self_pid = getpid() as usize;
    println!("[父进程] 进程号 = {}，子进程 = {}", self_pid, child_pid);
    let mut handshake = [0u8; 1 + size_of::<usize>()];
    handshake[0] = STAGE_HANDSHAKE;
    handshake[1..].copy_from_slice(&self_pid.to_le_bytes());
    assert_eq!(mail_write(child_pid, &handshake), handshake.len() as isize);
    println!("[父进程] 已发送握手 ({} 字节)", handshake.len());

    let mut ack = [0u8; 2];
    let got = recv_blocking(&mut ack);
    println!(
        "[父进程] 收到握手确认 长度={}，字节={:02x} {:02x}",
        got, ack[0], ack[1]
    );
    assert_eq!(got, ack.len());
    assert_eq!(ack, [STAGE_HANDSHAKE, 0]);

    let mut payload = [0x55u8; OVERFLOW_LEN];
    payload[0] = STAGE_OVERFLOW;
    let written = mail_write(child_pid, &payload);
    println!(
        "[父进程] 发送溢出负载: 期望={}，实际={}",
        OVERFLOW_LEN, written
    );
    assert_eq!(written, MAX_MSG_LEN as isize);

    let mut overflow_ack = [0u8; 3];
    let got = recv_blocking(&mut overflow_ack);
    println!(
        "[父进程] 收到溢出确认 长度={}，字节={:02x} {:02x} {:02x}",
        got, overflow_ack[0], overflow_ack[1], overflow_ack[2]
    );
    assert_eq!(got, overflow_ack.len());
    assert_eq!(overflow_ack[0], STAGE_OVERFLOW);
    let reported = u16::from_le_bytes([overflow_ack[1], overflow_ack[2]]) as usize;
    assert_eq!(reported, MAX_MSG_LEN);

    assert_eq!(mail_write(usize::MAX, &[0x12, 0x34]), -1);
    println!("[父进程] 写入非法进程号被正确拒绝");
}

fn run_child() -> i32 {
    let mut buf = [0u8; 512];
    let handshake_len = recv_blocking(&mut buf);
    println!(
        "[子进程] 握手长度={}，首字节={:02x}",
        handshake_len, buf[0]
    );
    assert_eq!(handshake_len, 1 + size_of::<usize>());
    assert_eq!(buf[0], STAGE_HANDSHAKE);
    let mut parent_bytes = [0u8; size_of::<usize>()];
    parent_bytes.copy_from_slice(&buf[1..handshake_len]);
    let parent_pid = usize::from_le_bytes(parent_bytes);
    println!("[子进程] 父进程号 = {}", parent_pid);

    assert_eq!(mail_write(parent_pid, &[STAGE_HANDSHAKE, 0]), 2);
    println!("[子进程] 已发送握手确认");

    let overflow_len = recv_blocking(&mut buf);
    println!("[子进程] 收到溢出负载 长度={}", overflow_len);
    assert_eq!(overflow_len, MAX_MSG_LEN);
    assert_eq!(buf[0], STAGE_OVERFLOW);
    for &byte in buf[1..overflow_len].iter() {
        assert_eq!(byte, 0x55);
    }
    let len_bytes = (overflow_len as u16).to_le_bytes();
    let ack = [STAGE_OVERFLOW, len_bytes[0], len_bytes[1]];
    assert_eq!(mail_write(parent_pid, &ack), ack.len() as isize);
    println!("[子进程] 已发送溢出确认 长度={}", ack.len());

    0
}

#[no_mangle]
pub fn main() -> i32 {
    let pid = fork();
    if pid == 0 {
        return run_child();
    }
    run_parent(pid as usize);
    let mut code: i32 = 0;
    let waited = waitpid(pid as usize, &mut code);
    assert_eq!(waited, pid);
    assert_eq!(code, 0);
    println!("\x1b[32mch7b_mailbox_limits 测试通过\x1b[0m");
    0
}
