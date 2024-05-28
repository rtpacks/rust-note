use std::{thread, time::Duration};

fn main() {
    /*
     *
     * ## 使用多线程
     *
     * ### 多线程编程的风险
     * 由于多线程的代码是同时运行的，所以无法保证线程间的执行顺序，这会导致一些问题：
     * - 竞态条件(race conditions)，多个线程以非一致性的顺序同时访问数据资源
     * - 死锁(deadlocks)，两个线程都想使用某个资源，但是又都在等待对方释放资源后才能使用，结果最终都无法继续执行
     * - 一些因为多线程导致的很隐晦的 BUG，难以复现和解决
     *
     * ### spawn 创建线程
     * 使用 thread::spawn 可以创建线程，它与主线程的执行顺序和次数依赖操作系统如何调度线程，总之，**千万不要依赖线程的执行顺序**：
     * ```rust
     * thread::spawn(|| {
     *     for i in 1..10 {
     *         println!("spawned thread, index = {}", i);
     *     }
     * });
     *
     * for j in 1..5 {
     *     println!("main thread, index = {}", j);
     *     thread::sleep(Duration::from_millis(1)); // thread::sleep() 可以强制线程停止执行一段时间
     * }
     * ```
     * 有几点值得注意：
     * - 线程内部的代码使用**闭包**来执行
     * - main 线程一旦结束，程序就立刻结束，如果其它子线程需要完成自己的任务，就需要保证主线程的存活
     * - thread::sleep 会让当前线程休眠指定的时间，随后其它线程会被调度运行，如果是在单核心处理上，那就会形成并发
     *
     * 输出
     * ```shell
     * main thread, index = 1
     * spawned thread, index = 1
     * spawned thread, index = 2
     * spawned thread, index = 3
     * spawned thread, index = 4
     * spawned thread, index = 5
     * spawned thread, index = 6
     * main thread, index = 2
     * spawned thread, index = 7
     * spawned thread, index = 8
     * main thread, index = 3
     * main thread, index = 4
     * ```
     *
     * ### join 等待线程结束
     * 在使用 spawn 创建线程中，由于主线程结束，导致依赖于主线程的新创建线程并没有执行完整。
     * 为了能让线程安全的结束执行，需要保证**被依赖线程**在依赖线程后结束，使用 join 可以达到目的。
     *
     * join 可以阻塞当前线程，直到 join 方法调用位置前的所有线程执行完成后才会解除当前线程的阻塞，同时 join 方法调用位置前的所有线程是不确定的轮换执行。
     *
     * ```rust
     * // 使用 join，可以使当前线程阻塞，直到 join 调用前的所有线程执行完成后才会放开限制
     * let handle1 = thread::spawn(|| {
     *     for i in 1..10 {
     *         println!("spawned1 thread, index = {}", i);
     *     }
     * });
     * let handle2 = thread::spawn(|| {
     *     for j in 1..10 {
     *         println!("spawned2 thread, index = {}", j);
     *     }
     * });
     * handle1.join().unwrap(); // spawned1 和 spawned2 不确定的轮换执行，直到两者结束后才会解除当前线程的阻塞限制
     *
     * for k in 1..5 {
     *     println!("main thread, index = {}", k);
     *     thread::sleep(Duration::from_millis(1));
     * }
     * ```
     *
     *
     *
     *
     *
     *
     */

    // 初步使用 thread
    // thread::spawn(|| {
    //     for i in 1..10 {
    //         println!("spawned thread, index = {}", i);
    //     }
    // });

    // for j in 1..5 {
    //     println!("main thread, index = {}", j);
    //     thread::sleep(Duration::from_millis(1)); // thread::sleep() 可以强制线程停止执行一段时间
    // }

    // 使用 join，可以使当前线程阻塞，直到 join 调用前的所有线程执行完成后才会放开限制
    let handle1 = thread::spawn(|| {
        for i in 1..10 {
            println!("spawned1 thread, index = {}", i);
        }
    });
    let handle2 = thread::spawn(|| {
        for j in 1..10 {
            println!("spawned2 thread, index = {}", j);
        }
    });
    handle1.join().unwrap();

    for k in 1..5 {
        println!("main thread, index = {}", k);
        thread::sleep(Duration::from_millis(1));
    }
}
