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
// 信号量 1-3：代表各类受限资源

// 理想结果：检测到死锁，至少有一个子线程返回值不为 0

// 屏障信号量在内核中的编号
const SEM_BARRIER: usize = 0;
// 子线程总数量
const THREAD_N: usize = 3;
// 资源类型数量
const RES_TYPE: usize = 3;
// 每类资源对应的信号量容量
const RES_NUM: [usize; RES_TYPE] = [1, 2, 1];
// 每个线程额外请求的资源编号
const REQUEST: [Option<usize>; THREAD_N] = [Some(1), Some(3), Some(2)];

/// 尝试对指定信号量执行 P 操作并检测死锁
fn try_sem_down(sem_id: usize) {
    // 检测信号量 P 操作的返回值是否为死锁错误码
    if semaphore_down(sem_id) == -0xdead {
        // 若发生死锁则先释放当前线程已经占有的资源
        sem_dealloc(gettid() as usize);
        // 输出死锁诊断信息
        println!("Deadlock detected. Test 08_sem1 failed!");
        // 异常退出当前线程
        exit(-1);
    }
}

/// 根据线程编号预先占用所需的资源信号量
fn sem_alloc(tid: usize) {
    // 匹配线程编号以决定占用的资源组合
    match tid {
        // 线程 1 仅需占有资源 2
        1 => assert_eq!(semaphore_down(2), 0),
        2 => {
            // 线程 2 依次占有资源 1 与 2
            assert_eq!(semaphore_down(1), 0);
            assert_eq!(semaphore_down(2), 0);
        }
        // 线程 3 仅需占有资源 3
        3 => assert_eq!(semaphore_down(3), 0),
        // 非法编号直接退出
        _ => exit(1),
    }
    // 完成资源预占后等待主线程进一步调度
    semaphore_down(SEM_BARRIER);
}

/// 释放线程已持有的资源并通知主线程
fn sem_dealloc(tid: usize) {
    // 先释放同步屏障供主线程继续推进
    semaphore_up(SEM_BARRIER);
    // 依据线程编号释放对应的资源组
    match tid {
        // 线程 1 归还资源 2
        1 => semaphore_up(2),
        2 => {
            // 线程 2 依次归还资源 1 与 2
            semaphore_up(1);
            semaphore_up(2);
        }
        // 线程 3 归还资源 3
        3 => semaphore_up(3),
        // 非法编号直接退出
        _ => exit(1),
    }
}

/// 子线程函数：执行互斥资源申请以构造死锁场景
fn deadlock_test() {
    // 获取当前线程编号（从 1 开始）
    let tid = gettid() as usize;
    // 打印线程启动信息
    println!("thread {} running", tid);
    // 预先占用基础资源并等待主线程释放屏障
    sem_alloc(tid);
    // 若线程仍需额外资源则尝试获取
    if let Some(sem_id) = REQUEST[tid - 1] {
        // 对额外资源尝试 P 操作并检测死锁
        try_sem_down(sem_id);
        // 正常场景下立即释放额外资源
        semaphore_up(sem_id);
    }
    // 完成测试后释放所有持有资源
    sem_dealloc(tid);
    // 打印线程退出信息
    println!("thread {} exited", tid);
    // 正常退出线程
    exit(0);
}

/// 程序入口：构造信号量死锁检测测试
#[no_mangle]
pub fn main() -> i32 {
    // 开启内核层面的死锁检测功能
    enable_deadlock_detect(true);
    // 创建同步屏障信号量并确认编号与预期一致
    assert_eq!(semaphore_create(THREAD_N) as usize, SEM_BARRIER);
    // 预先消耗屏障令牌以阻塞子线程
    for _ in 0..THREAD_N {
        semaphore_down(SEM_BARRIER);
    }

    // 根据资源配置创建各类型资源信号量
    for n in RES_NUM {
        semaphore_create(n);
    }
    // 记录子线程的线程标识符
    let mut tids = [0; THREAD_N];

    // 创建多个子线程执行死锁构造逻辑
    for item in tids.iter_mut().take(THREAD_N) {
        // 保存新创建线程的标识符
        *item = thread_create(deadlock_test as usize, 0) as usize;
    }

    // 暂停一段时间以确保子线程阻塞在屏障上
    sleep(500);
    // 释放屏障令牌触发资源竞争
    for _ in 0..THREAD_N {
        semaphore_up(SEM_BARRIER);
    }
    // 统计检测到死锁的线程数量
    let mut failed = 0;
    for tid in tids {
        // 等待线程结束并检查返回码
        if waittid(tid) != 0 {
            failed += 1;
        }
    }
    // 确认至少有一个线程因死锁检测而失败
    assert!(failed > 0);
    // 输出测试成功信息
    println!("deadlock test semaphore 1 OK!");
    // 正常返回
    0
}
