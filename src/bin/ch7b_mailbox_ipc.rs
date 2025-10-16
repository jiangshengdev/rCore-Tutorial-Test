#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::mem::size_of;
use user_lib::{fork, getpid, mail_read, mail_write, waitpid, yield_};

const MAX_MSG_LEN: usize = 256;
const STAGE_HANDSHAKE: u8 = 0;
const STAGE_LARGE: u8 = 1;
const STAGE_TRUNC: u8 = 2;
const LARGE_FILL: u8 = 0x5a;
const LARGE_SEND_LEN: usize = MAX_MSG_LEN + 32;
const TRUNC_TOTAL_LEN: usize = 6;

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
    let parent_pid = getpid() as usize;
    println!("[parent] pid = {}, child = {}", parent_pid, child_pid);

    let mut handshake = [0u8; 1 + size_of::<usize>()];
    handshake[0] = STAGE_HANDSHAKE;
    handshake[1..].copy_from_slice(&parent_pid.to_le_bytes());
    assert_eq!(mail_write(child_pid, &handshake), handshake.len() as isize);
    println!("[parent] sent handshake ({} bytes)", handshake.len());

    let mut ack = [0u8; 3];
    let got = recv_blocking(&mut ack);
    println!(
        "[parent] got handshake ack: len={}, bytes={:02x} {:02x} {:02x}",
        got, ack[0], ack[1], ack[2]
    );
    assert_eq!(got, ack.len());
    assert_eq!(ack[0], STAGE_HANDSHAKE);
    assert_eq!(&ack[1..], b"OK");

    let mut large_payload = [LARGE_FILL; LARGE_SEND_LEN];
    large_payload[0] = STAGE_LARGE;
    let wrote = mail_write(child_pid, &large_payload);
    println!(
        "[parent] sent large payload: requested={}, actual={}",
        LARGE_SEND_LEN, wrote
    );
    assert_eq!(wrote, MAX_MSG_LEN as isize);

    let mut large_ack = [0u8; 3];
    let got = recv_blocking(&mut large_ack);
    println!(
        "[parent] got large ack: len={}, bytes={:02x} {:02x} {:02x}",
        got, large_ack[0], large_ack[1], large_ack[2]
    );
    assert_eq!(got, large_ack.len());
    assert_eq!(large_ack[0], STAGE_LARGE);
    let len_report = u16::from_le_bytes([large_ack[1], large_ack[2]]) as usize;
    assert_eq!(len_report, MAX_MSG_LEN);

    let trunc_message = [STAGE_TRUNC, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4];
    let wrote = mail_write(child_pid, &trunc_message);
    println!(
        "[parent] sent trunc payload: requested={}, actual={}",
        TRUNC_TOTAL_LEN, wrote
    );
    assert_eq!(wrote, TRUNC_TOTAL_LEN as isize);

    let mut trunc_ack = [0u8; 4];
    let got = recv_blocking(&mut trunc_ack);
    println!(
        "[parent] got trunc ack: len={}, bytes={:02x} {:02x} {:02x} {:02x}",
        got, trunc_ack[0], trunc_ack[1], trunc_ack[2], trunc_ack[3]
    );
    assert_eq!(got, trunc_ack.len());
    assert_eq!(trunc_ack[0], STAGE_TRUNC);
    assert_eq!(trunc_ack[1], trunc_message[1]);
    assert_eq!(trunc_ack[2], trunc_message[2]);
    assert_eq!(trunc_ack[3], trunc_message[3]);
}

fn run_child() -> i32 {
    let mut buf = [0u8; MAX_MSG_LEN];
    let handshake_len = recv_blocking(&mut buf);
    println!(
        "[child] received handshake len={}, bytes[0]={:02x}",
        handshake_len, buf[0]
    );
    assert_eq!(handshake_len, 1 + size_of::<usize>());
    assert_eq!(buf[0], STAGE_HANDSHAKE);
    let mut parent_bytes = [0u8; size_of::<usize>()];
    parent_bytes.copy_from_slice(&buf[1..handshake_len]);
    let parent_pid = usize::from_le_bytes(parent_bytes);
    println!("[child] parent pid = {}", parent_pid);

    let ack = [STAGE_HANDSHAKE, b'O', b'K'];
    assert_eq!(mail_write(parent_pid, &ack), ack.len() as isize);
    println!("[child] sent handshake ack");

    let large_len = recv_blocking(&mut buf);
    println!("[child] received large payload len={}", large_len);
    assert_eq!(large_len, MAX_MSG_LEN);
    assert_eq!(buf[0], STAGE_LARGE);
    for &byte in buf[1..large_len].iter() {
        assert_eq!(byte, LARGE_FILL);
    }
    let len_bytes = (large_len as u16).to_le_bytes();
    let large_ack = [STAGE_LARGE, len_bytes[0], len_bytes[1]];
    assert_eq!(mail_write(parent_pid, &large_ack), large_ack.len() as isize);
    println!("[child] sent large ack len={}", large_ack.len());

    let mut small_buf = [0u8; 4];
    let trunc_len = recv_blocking(&mut small_buf);
    println!("[child] received trunc payload len={}", trunc_len);
    assert_eq!(trunc_len, TRUNC_TOTAL_LEN);
    assert_eq!(small_buf[0], STAGE_TRUNC);
    assert_eq!(mail_write(parent_pid, &small_buf), small_buf.len() as isize);
    println!("[child] sent trunc ack len={}", small_buf.len());

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
    println!("\x1b[32mch7b_mailbox_ipc passed\x1b[0m");
    0
}
