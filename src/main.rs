use std::{
    ops::Sub,
    sync::{
        atomic::{AtomicI32, AtomicI64, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Instant,
};

fn main() {
    /*
     *
     * ## 线程同步：Atomic 原子类型与内存顺序
     * 在多线程环境下访问共享数据时，需要确保数据一致性和完整性。传统的方法是使用锁（如互斥锁 Mutex）来保护共享数据，但锁可能导致性能开销和潜在的死锁问题。
     * 在 rust 中，Mutex 用起来简单但是无法并发读，RwLock 可以并发读但是使用场景较为受限且性能不够。
     *
     * 原子类型（atomic types）是并发编程中用于实现**无锁（lock-free）同步**的一种工具。它们提供了对**基本数据类型**的原子操作。
     *
     * 原子指的是一系列不可被 CPU 上下文交换的机器指令，这些指令组合在一起就形成了原子操作。
     * 在多核 CPU 下，当某个 CPU 核心开始运行原子操作时，会先暂停其它 CPU 内核对内存的操作，以保证原子操作不会被其它 CPU 内核所干扰。
     * 由于原子操作是通过指令提供的支持，因此它的性能相比锁和消息传递会好很多。
     *
     * 相比较于锁而言，原子类型不需要开发者处理加锁和释放锁的问题，同时支持修改，读取等操作，还具备较高的并发性能，几乎所有的语言都支持原子类型。
     *
     * 虽然原子类型是无锁类型，但是无锁不代表无需等待，因为原子类型内部使用了 CAS 循环，当大量的冲突发生时，还是需要等待，但是比锁要好。
     *
     * > CAS 全称是 Compare and swap, 它通过一条指令读取指定的内存地址，然后判断其中的值是否等于给定的前置值，如果相等，则将其修改为新的值。
     *
     * 原子类型提供了一组原子操作方法，如 load、store、swap、compare_and_swap、fetch_add 等。
     * ```rust
     * //  计算 1-5000000 的和，分为 5 个线程完成，最终总数为 x + (1 + 5000) * 5000 / 2
     * let counter = Arc::new(AtomicI64::new(0));
     * let thread_count = 5;
     * let mut handles: Vec<JoinHandle<()>> = Vec::new();
     * let start_time = Instant::now();
     * for i in 0..thread_count {
     *     let _counter = Arc::clone(&counter);
     *     handles.push(thread::spawn(move || {
     *         for j in 1..1000001 {
     *             // 使用 fetch_add 增加数据
     *             _counter.fetch_add(i * j, Ordering::SeqCst);
     *         }
     *         // println!("{} 当前计算累加和 = {}", i, _counter.load(Ordering::SeqCst));
     *     }));
     * }
     * for h in handles {
     *     h.join().unwrap();
     * }
     * let end_time = Instant::now();
     * // 使用 load 获取数据
     * println!(
     *     "counter = {}, time = {:?}",
     *     counter.load(Ordering::SeqCst),
     *     end_time.sub(start_time)
     * );
     * ```
     *
     * 在这种情况下，使用互斥锁计算，会比原子类型花费更长的时间
     * ```rust
     * // 使用 Mutex 计算 1-5000000 的和，分为五个线程，统计耗时
     * let counter: Arc<Mutex<i64>> = Arc::new(Mutex::new(0));
     * let thread_count = 5;
     * let mut handles: Vec<JoinHandle<()>> = Vec::new();
     * let start_time = Instant::now();
     * for i in 0..thread_count {
     *     let _counter = Arc::clone(&counter);
     *     handles.push(thread::spawn(move || {
     *         for j in 1..1000001 {
     *             // 使用 fetch_add 增加数据
     *             let mut num = _counter.lock().unwrap();
     *             *num += i * j;
     *         }
     *         // println!("{} 当前计算累加和 = {}", i, _counter.load(Ordering::SeqCst));
     *     }));
     * }
     * for h in handles {
     *     h.join().unwrap();
     * }
     * let end_time = Instant::now();
     * println!(
     *     "counter = {}, time = {:?}",
     *     counter.lock().unwrap(),
     *     end_time.sub(start_time)
     * );
     * ```
     *
     * 输出结果为：
     * ```shell
     * counter = 5000005000000, time = 226.9952ms
     * counter = 5000005000000, time = 1.1381521s
     * ```
     *
     * ### 内存顺序
     * https://course.rs/advance/concurrency-with-threads/sync2.html#内存顺序
     *
     * ### Atomic 和互斥锁
     *
     * 原子类型并不能完全替代锁：
     * - 对于复杂的场景下，锁的使用简单粗暴，不容易有坑
     * - std::sync::atomic 包中仅提供了数值类型的原子操作：AtomicBool, AtomicIsize, AtomicUsize, AtomicI8, AtomicU16等，而锁可以应用于各种类型
     * - 在有些情况下，必须使用锁来配合，例如使用 Mutex 配合 Condvar
     *
     * ### Atomic 的应用场景
     * 事实上，Atomic虽然对于用户不太常用，但是对于高性能库的开发者、标准库开发者都非常常用，它是并发原语的基石，除此之外，还有一些场景适用：
     * - 无锁(lock free)数据结构
     * - 全局变量，例如全局自增 ID, 在后续章节会介绍
     * - 跨线程计数器，例如可以用于统计指标
     *
     */

    //  计算 1-5000000 的和，分为 5 个线程完成，最终总数为 x + (1 + 5000) * 5000 / 2
    let counter = Arc::new(AtomicI64::new(0));
    let thread_count = 5;
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    let start_time = Instant::now();
    for i in 0..thread_count {
        let _counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for j in 1..1000001 {
                // 使用 fetch_add 增加数据
                _counter.fetch_add(i * j, Ordering::SeqCst);
            }
            // println!("{} 当前计算累加和 = {}", i, _counter.load(Ordering::SeqCst));
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let end_time = Instant::now();
    // 使用 load 获取数据
    println!(
        "counter = {}, time = {:?}",
        counter.load(Ordering::SeqCst),
        end_time.sub(start_time)
    );

    // 使用 Mutex 计算 1-5000000 的和，分为五个线程，统计耗时
    let counter: Arc<Mutex<i64>> = Arc::new(Mutex::new(0));
    let thread_count = 5;
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    let start_time = Instant::now();
    for i in 0..thread_count {
        let _counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for j in 1..1000001 {
                // 使用 fetch_add 增加数据
                let mut num = _counter.lock().unwrap();
                *num += i * j;
            }
            // println!("{} 当前计算累加和 = {}", i, _counter.load(Ordering::SeqCst));
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let end_time = Instant::now();
    println!(
        "counter = {}, time = {:?}",
        counter.lock().unwrap(),
        end_time.sub(start_time)
    );
}
