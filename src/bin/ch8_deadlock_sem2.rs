#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use user_lib::{
    enable_deadlock_detect, exit, semaphore_create, semaphore_down, semaphore_up, sleep,
};
use user_lib::{gettid, thread_create, waittid};

// 信号量 0：用于主线程与子线程同步
// 信号量 1-3：代表受控资源的数量

// 理想结果：未检测到死锁，子线程返回值均为 0

// 子线程总数量
const THREAD_N: usize = 4;
// 资源类别数量
const RES_TYPE: usize = 2;
// 每类资源的信号量容量
const RES_NUM: [usize; RES_TYPE] = [2, 2];
// 每个线程初始需要占有的资源
const ALLOC: [usize; THREAD_N] = [2, 1, 1, 2];
// 每个线程额外尝试申请的资源
const REQUEST: [Option<usize>; THREAD_N] = [Some(1), None, Some(2), None];

/// 尝试对指定信号量执行 P 操作并在失败时回滚资源
fn try_sem_down(sem_id: usize) {
    // 检测 P 操作是否因死锁预警而失败
    if semaphore_down(sem_id) == -0xdead {
        // 回滚当前线程已占用的资源信号量
        semaphore_up(ALLOC[(gettid() - 1) as usize]);
        // 以错误码退出表示未通过测试
        exit(-1);
    }
}

/// 子线程执行的资源申请流程，验证系统无死锁
fn deadlock_test() {
    // 计算当前线程对应的索引
    let id = (gettid() - 1) as usize;
    // 先占有分配表指向的资源信号量
    assert_eq!(semaphore_down(ALLOC[id]), 0);
    // 等待主线程通过屏障信号量统一放行
    semaphore_down(0);
    // 若配置要求额外资源，则尝试进一步申请
    if let Some(sem_id) = REQUEST[id] {
        // 尝试申请额外资源并检测死锁
        try_sem_down(sem_id);
        // 完成后立即归还额外资源
        semaphore_up(sem_id);
    }
    // 归还初始占有的资源信号量
    semaphore_up(ALLOC[id]);
    // 正常结束线程
    exit(0);
}

/// 程序入口：构造信号量无死锁的验证场景
#[no_mangle]
pub fn main() -> i32 {
    // 启用系统死锁检测功能
    enable_deadlock_detect(true);
    // 创建同步屏障信号量
    semaphore_create(THREAD_N);
    // 预先取走所有令牌以阻塞子线程
    for _ in 0..THREAD_N {
        semaphore_down(0);
    }

    // 为每类资源创建对应数量的信号量
    for n in RES_NUM {
        semaphore_create(n);
    }
    // 存放各子线程的线程标识符
    let mut tids = [0; THREAD_N];

    // 逐一创建子线程执行测试逻辑
    for item in tids.iter_mut().take(THREAD_N) {
        // 记录新建线程的标识符
        *item = thread_create(deadlock_test as usize, 0) as usize;
    }

    // 等待子线程全部进入就绪状态
    sleep(1000);
    // 释放屏障令牌以让子线程继续执行
    for _ in 0..THREAD_N {
        semaphore_up(0);
    }

    // 统计未能正常结束的线程数量
    let mut failed = 0;
    for tid in tids {
        // 等待子线程结束并检查返回值
        if waittid(tid) != 0 {
            failed += 1;
        }
    }

    // 确认所有线程均成功执行完毕
    assert_eq!(failed, 0);
    // 打印测试通过信息
    println!("deadlock test semaphore 2 OK!");
    // 正常退出程序
    0
}
