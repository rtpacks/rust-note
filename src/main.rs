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
     * ### 线程的结束方式
     * main 线程是程序的主线程，一旦结束则程序随之结束，同时各个子线程也将被强行终止。
     * 如果父线程不是 main 线程，那么父线程的结束后子线程是继续运行还是被强行终止？
     *
     * 答案是**当父线程不是 main 线程时，父线程的结束不会强制终止子线程，只有线程的代码执行完，线程才会自动结束**。
     * 这是因为虽然在系统编程中，操作系统提供了直接杀死线程的接口，但是**粗暴地终止一个线程可能会引发资源没有释放、状态混乱等不可预期的问题**，所以 Rust 并没有提供功能。
     *
     * 因此**非主线程的子线程的代码不会执行完时(阻塞、死循环)，子线程就不会结束，此时只有主动关闭或结束主线程才能关闭子线程**。
     *
     * **阻塞和死循环**
     * - 线程的任务是一个循环 IO 读取，任务流程类似：IO 阻塞，等待读取新的数据 -\> 读到数据，处理完成 -\> 继续阻塞等待 ··· -\> 收到 socket 关闭的信号 -\> 结束线程，在此过程中，绝大部分时间线程都处于阻塞的状态，因此虽然看上去是循环，CPU 占用其实很小，也是网络服务中最常见的模型
     * - 线程的任务是一个循环，没有任何阻塞（也不包括休眠），此时如果没有设置终止条件，该线程将持续跑满一个 CPU 核心，并且不会被终止，直到 main 线程的结束
     *
     * ```rust
     * // 线程的关闭方式
     * let handle = thread::spawn(|| {
     *     thread::spawn(|| loop {
     *         println!("sub sub thread running");
     *     });
     *     println!("sub thread end");
     * });
     * handle.join().unwrap();
     *
     * // 睡眠一段时间，看子线程创建的子线程是否还在运行
     * thread::sleep(Duration::from_millis(10));
     * ```
     *
     * ### join 等待线程结束
     * 在使用 spawn 创建线程中，由于主线程结束，导致依赖主线程的新创建线程并没有执行完整。
     * 为了能让线程安全的结束执行，需要保证**主线程**在依赖线程后结束，使用 join 可以达到目的。
     *
     * join 可以阻塞当前线程，直到 join 方法调用位置前的所有的**同级线程**执行完成后才会解除当前线程的阻塞，同时 join 方法调用位置前的所有**同级线程**是不确定的轮换执行。
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
     * for k in 1..5 {
     *     println!("main thread, index = {}", k);
     *     thread::sleep(Duration::from_millis(1));
     * }
     * ```
     *
     * ### move 和多线程
     * 线程的启动时间点和结束时间点是不确定的，与代码的创建点无关。
     * 由于这种无序和不确定性，**被依赖线程**不一定在依赖线程后结束，换句话说，存在**被依赖线程结束运行，而依赖线程仍在运行**的情况。
     *
     * 这意味依赖线程访问**被依赖线程**的环境时，需要有一个限制条件：
     * 当依赖线程的闭包访问被依赖线程的变量时，需要将被依赖线程的变量所有权转移到依赖线程的闭包内，否则容易出现被依赖线程结束运行后变量被释放，而依赖线程还在访问该变量的问题。
     *
     * 因此使用 move 关键字拿走访问变量的所有权，被依赖线程就无法再使用该变量，也就是不会再释放该变量的值。
     * ```rust
     * let v = vec![1, 2, 3];
     * let handle = thread::spawn(move || {
     *     println!("spawned thread, index = {:?}", v);
     * });
     * handle.join().unwrap();
     * ```
     *
     * ### 多线程的性能
     * 不精确估算，创建一个线程大概需要 0.24 毫秒，随着线程的变多，这个值会变得更大，因此线程的创建耗时是不可忽略的，**只有真的值得用线程去处理的任务时，才使用线程**。
     *
     * #### 创建合适的线程数量
     * 合适的线程数是提高并发性能的关键，并发性能关键在于 CPU 负载利用率的高低。
     * 
     * 当任务是 CPU 密集型时，就算线程数超过了 CPU 核心数，也不能获得更好的性能，因为每个线程的任务都可以轻松让 CPU 的某个核心跑满，CPU 负载利用率的高，此时让线程数等于 CPU 核心数是最好的。
     * 
     * 当任务大部分时间都处于阻塞状态时，就可以考虑增多线程数量，这样当某个线程处于阻塞状态时会被切走，系统运行其它的线程。
     * 典型就是网络 IO 操作，可以为每一个进来的用户连接创建一个线程去处理，该连接绝大部分时间都是处于 IO 读取阻塞状态，因此有限的 CPU 核心完全可以处理成百上千的用户连接线程。
     * 但事实上网络 IO 一般都不再使用多线程的方式了，毕竟操作系统的线程数是有限的，意味着并发数也很容易达到上限，而且过多的线程也会导致线程上下文切换的代价过大，使用 async/await 的 M:N 并发模型就没有这个烦恼。
     * 
     * 如果不确定任务是 CPU 密集型还是阻塞型，就需要通过测试来确定选择合适的线程数量。
     * 
     * CPU 密集型任务尽量让线程数量等于 CPU 核心数，阻塞型任务可以选择 async/await 的 M:N 模型。
     *
     * #### 多线程的开销
     * 阅读：https://course.rs/advance/concurrency-with-threads/thread.html#多线程的开销
     *
     * 性能不会随着线程数的增加而线性增长，锁、数据竞争、缓存失效这些限制了现代化软件系统随着 CPU 核心的增多性能也线性增加的野心。
     * - 线程过多时，CPU 缓存的命中率会显著下降，同时多个线程竞争一个 CPU Cache-line 的情况也会经常发生
     * - 大量读写可能会让内存带宽也成为瓶颈
     * - 读和写不一样，无锁数据结构的读往往可以很好地线性增长，但是写不行，因为写竞争太大
     *
     */

    // 一、初步使用 thread
    // thread::spawn(|| {
    //     for i in 1..10 {
    //         println!("spawned thread, index = {}", i);
    //     }
    // });
    // for j in 1..5 {
    //     println!("main thread, index = {}", j);
    //     thread::sleep(Duration::from_millis(1)); // thread::sleep() 可以强制线程停止执行一段时间
    // }

    // 二、线程的关闭方式
    // let handle = thread::spawn(|| {
    //     thread::spawn(|| loop {
    //         println!("sub sub thread running");
    //     });
    //     println!("sub thread end");
    // });
    // handle.join().unwrap();
    // 睡眠一段时间，看子线程创建的子线程是否还在运行
    // thread::sleep(Duration::from_millis(10));

    // 三、使用 join，可以使当前线程阻塞，直到 join 调用前的所有线程执行完成后才会放开阻塞限制
    // let handle1 = thread::spawn(|| {
    //     for i in 1..10 {
    //         println!("spawned1 thread, index = {}", i);
    //     }
    // });
    // let handle2 = thread::spawn(|| {
    //     for j in 1..10 {
    //         println!("spawned2 thread, index = {}", j);
    //     }
    // });
    // handle1.join().unwrap();
    // for k in 1..5 {
    //     println!("main thread, index = {}", k);
    //     thread::sleep(Duration::from_millis(1));
    // }

    // 四、move 和闭包
    // let v = vec![1, 2, 3];
    // let handle = thread::spawn(move || {
    //     println!("spawned thread, index = {:?}", v);
    // });
    // handle.join().unwrap();
}
