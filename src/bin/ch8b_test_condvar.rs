#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::exit;
use user_lib::{
    condvar_create, condvar_signal, condvar_wait, mutex_blocking_create, mutex_lock, mutex_unlock,
};
use user_lib::{sleep_blocking, thread_create, waittid};

static mut A: usize = 0;

const CONDVAR_ID: usize = 0;
const MUTEX_ID: usize = 0;

/// 线程一：加锁修改条件并 signal 唤醒等待者
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

/// 线程二：持锁检查条件，不满足则在 condvar 上等待并自动释放互斥
unsafe fn second() -> ! {
    println!("Second want to continue,but need to wait A=1");
    // 持锁进入检查
    mutex_lock(MUTEX_ID);
    // 用 while 防止虚假唤醒：醒来后需再次检查条件
    while A == 0 {
        println!("Second: A is {}", A);
        condvar_wait(CONDVAR_ID, MUTEX_ID);
    }
    mutex_unlock(MUTEX_ID);
    println!("A is {}, Second can work now", A);
    exit(0)
}

#[no_mangle]
/// 程序入口：创建 condvar 与 mutex，启动两个线程并等待结束
pub fn main() -> i32 {
    // create condvar & mutex
    assert_eq!(condvar_create() as usize, CONDVAR_ID);
    assert_eq!(mutex_blocking_create() as usize, MUTEX_ID);
    // create threads
    let threads = [
        thread_create(first as usize, 0),
        thread_create(second as usize, 0),
    ];
    // wait for all threads to complete
    for &thread in &threads {
        waittid(thread as usize);
    }
    println!("test_condvar passed!");
    0
}
