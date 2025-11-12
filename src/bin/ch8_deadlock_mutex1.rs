#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use user_lib::{enable_deadlock_detect, mutex_blocking_create, mutex_lock, mutex_unlock};

// 理想结果：检测到死锁

/// 启动互斥量死锁检测的用户程序入口
#[no_mangle]
pub fn main() -> i32 {
    // 启用内核提供的死锁检测功能
    enable_deadlock_detect(true);
    // 创建一个阻塞式互斥量并取得句柄
    let mid = mutex_blocking_create() as usize;
    // 首次加锁应当成功返回 0
    assert_eq!(mutex_lock(mid), 0);
    // 再次加锁会被判定为死锁并返回特定错误码
    assert_eq!(mutex_lock(mid), -0xdead);
    // 解锁互斥量以恢复资源状态
    mutex_unlock(mid);
    // 输出测试成功信息
    println!("deadlock test mutex 1 OK!");
    // 正常返回 0 表示程序结束
    0
}
