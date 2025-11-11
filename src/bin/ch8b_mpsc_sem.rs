#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::vec::Vec;
use user_lib::exit;
use user_lib::{semaphore_create, semaphore_down, semaphore_up};
use user_lib::{thread_create, waittid};

const SEM_MUTEX: usize = 0;
const SEM_EMPTY: usize = 1;
const SEM_EXISTED: usize = 2;
const BUFFER_SIZE: usize = 8;
static mut BUFFER: [usize; BUFFER_SIZE] = [0; BUFFER_SIZE];
static mut FRONT: usize = 0;
static mut TAIL: usize = 0;
const PRODUCER_COUNT: usize = 4;
const NUMBER_PER_PRODUCER: usize = 100;

/// 生产者线程：向环形缓冲写入自身 id，总共写入 NUMBER_PER_PRODUCER 次
unsafe fn producer(id: *const usize) -> ! {
    let id = *id;
    // 循环生产固定数量的元素
    for _ in 0..NUMBER_PER_PRODUCER {
        // 确保存在空槽：若计数为 0 则阻塞
        semaphore_down(SEM_EMPTY);
        // 获取互斥：独占访问 FRONT 与 BUFFER
        semaphore_down(SEM_MUTEX);
        // 写入数据：此处用生产者 id 作为简单负载
        BUFFER[FRONT] = id;
        // 推进写指针：保持环形不变量 FRONT < BUFFER_SIZE
        FRONT = (FRONT + 1) % BUFFER_SIZE;
        // 释放互斥
        semaphore_up(SEM_MUTEX);
        // 增加“已有元素”计数，可能唤醒消费者
        semaphore_up(SEM_EXISTED);
    }
    exit(0)
}

/// 消费者线程：不断从环形缓冲取出元素直到总数耗尽
unsafe fn consumer() -> ! {
    // 需要消费 PRODUCER_COUNT * NUMBER_PER_PRODUCER 个元素
    for _ in 0..PRODUCER_COUNT * NUMBER_PER_PRODUCER {
        // 确保至少存在一个已生产元素
        semaphore_down(SEM_EXISTED);
        // 互斥访问缓冲与 TAIL 指针
        semaphore_down(SEM_MUTEX);
        // 读取当前元素
        print!("{} ", BUFFER[TAIL]);
        // 推进读指针
        TAIL = (TAIL + 1) % BUFFER_SIZE;
        // 释放互斥
        semaphore_up(SEM_MUTEX);
        // 归还一个空槽
        semaphore_up(SEM_EMPTY);
    }
    println!("");
    exit(0)
}

#[no_mangle]
/// 程序入口：初始化三个信号量并启动多个生产者与一个消费者
pub fn main() -> i32 {
    // create semaphores
    assert_eq!(semaphore_create(1) as usize, SEM_MUTEX);
    assert_eq!(semaphore_create(BUFFER_SIZE) as usize, SEM_EMPTY);
    assert_eq!(semaphore_create(0) as usize, SEM_EXISTED);
    // create threads
    // 预生成生产者 id 列表
    let ids: Vec<_> = (0..PRODUCER_COUNT).collect();
    let mut threads = Vec::new();
    // 创建所有生产者线程
    for i in 0..PRODUCER_COUNT {
        threads.push(thread_create(
            producer as usize,
            &ids.as_slice()[i] as *const _ as usize,
        ));
    }
    // 创建消费者线程
    threads.push(thread_create(consumer as usize, 0));
    // wait for all threads to complete
    // 依次等待所有线程结束
    for thread in threads.iter() {
        waittid(*thread as usize);
    }
    println!("mpsc_sem passed!");
    0
}
