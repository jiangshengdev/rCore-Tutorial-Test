#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::exit;
use user_lib::{
    condvar_create, condvar_signal, condvar_wait, mutex_blocking_create, mutex_lock, mutex_unlock,
};
use user_lib::{sleep_blocking, thread_create, waittid};

// 共享状态：受互斥锁保护的条件变量检查目标
static mut A: usize = 0;

// 条件变量 ID（创建顺序决定编号，此处期望为 0）
const CONDVAR_ID: usize = 0;
// 互斥锁 ID（创建顺序决定编号，此处期望为 0）
const MUTEX_ID: usize = 0;

/// 线程一：加锁修改条件并 signal(发送信号) 唤醒等待者
unsafe fn first() -> ! {
    // 工作模拟：先休眠一段时间
    sleep_blocking(10);
    println!("First work, Change A --> 1 and wakeup Second");
    // 持锁修改条件
    mutex_lock(MUTEX_ID);
    A = 1;
    // 发出唤醒信号（若等待队列非空）
    condvar_signal(CONDVAR_ID);
    mutex_unlock(MUTEX_ID);
    exit(0)
}

/// 线程二：持锁检查条件，不满足则在条件变量(condvar)上等待并自动释放互斥
unsafe fn second() -> ! {
    println!("Second want to continue,but need to wait A=1");
    // 持锁进入检查
    mutex_lock(MUTEX_ID);
    // 使用 while(循环) 防止虚假唤醒：醒来后需再次检查条件
    while A == 0 {
        println!("Second: A is {}", A);
        condvar_wait(CONDVAR_ID, MUTEX_ID);
    }
    mutex_unlock(MUTEX_ID);
    println!("A is {}, Second can work now", A);
    exit(0)
}

#[no_mangle]
/// 程序入口：创建条件变量(condvar)与互斥锁(mutex)，启动两个线程并等待结束
pub fn main() -> i32 {
    // 创建条件变量与互斥锁
    assert_eq!(condvar_create() as usize, CONDVAR_ID);
    assert_eq!(mutex_blocking_create() as usize, MUTEX_ID);
    // 创建线程
    let threads = [
        thread_create(first as usize, 0),
        thread_create(second as usize, 0),
    ];
    // 等待所有线程结束
    for &thread in &threads {
        waittid(thread as usize);
    }
    println!("test_condvar passed!");
    0
}
