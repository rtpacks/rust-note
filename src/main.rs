use std::{
    sync::{
        mpsc::{self, Sender, SyncSender},
        Arc, Condvar, Mutex, RwLock,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use tokio::sync::Semaphore;

#[tokio::main]
async fn main() {
    /*
     *
     * ## 线程同步：锁、Condvar 和信号量
     * **同步性指的是通过协调不同线程或任务的执行顺序来安全地共享数据和资源**。
     * 同步性是并发编程中的一个重要概念，涉及到如何保证多个执行单元（如线程或异步任务）之间正确且安全地访问共享资源，而不会导致数据竞争、死锁等问题。
     *
     * 借助 Rust 强大的类型系统和所有权模型，在编写多线程代码，需要使用同步性时，可以通过互斥锁(Mutex)、读写锁(RwLock)、原子类型(Atomic Types)和通道(Channel)等机制，编写高效且安全的并发程序。
     *
     * 在多线程间有多种方式可以共享和传递数据，最常用有两种：
     * - 消息传递
     * - 锁和 Arc 联合使用
     *
     * 对于消息传递，在编程界有一个大名鼎鼎的 **Actor 线程模型**为其背书，典型的有 Erlang 语言、Go 语言。
     *
     * ### 如何选择数据共享方式
     *
     * **共享内存**是同步的灵魂，消息传递的底层也是通过共享内存来实现的：
     * - 消息传递类似一个单所有权的系统，一个值同时只能有一个所有者，如果另一个线程需要该值的所有权，需要将所有权通过消息传递进行转移，可以做到传递引用和传递值
     * - 而共享内存类似于一个多所有权的系统，多个线程可以同时访问同一个值，用锁来控制哪个线程可以在当前时刻访问，可以做到直接访问同一个内存
     *
     * 对比两种方式：
     * - 锁和 Arc 联合使用的共享内存相对消息传递能节省多次内存拷贝的成本
     * - 共享内存的实现简洁的多
     * - 共享内存的锁竞争更多
     *
     * 消息传递适用的场景很多，几个主要的使用场景:
     * - 需要可靠和简单的(简单不等于简洁)实现多线程编程
     * - 需要模拟现实世界，例如用消息去通知某个目标执行相应的操作时（事件触发）
     * - 需要一个任务处理流水线(管道)时，等等
     *
     * 而使用共享内存(并发原语)的场景往往就比较简单粗暴：需要**简洁的实现以及更高的性能**。
     *
     * ### 互斥锁 Mutex
     * > Mutex 在之前章节已经用过，这里的介绍有点繁琐，精简了一下学习过程
     * > https://course.rs/advance/concurrency-with-threads/sync1.html#互斥锁-mutex
     *
     * 在之前章节介绍中提到过，Mutex 是一个并发原语，它能让多个线程并发的访问同一个值变成了排队访问，同一时间只允许一个线程 A 访问该值，其它线程需要等待 A 访问完成后才能访问。
     *
     * 使用 Mutex 时，需要先锁定它访问数据，然后再解锁让其他线程可以访问该数据。
     * 锁定和解锁的过程通常是自动的，通过 Rust 的作用域管理来实现。当 Mutex 的锁超出作用域时，它会自动释放。
     *
     * 不同于线程局部变量的每一个线程都有单独的数据拷贝，**Mutex 用于多线程访问同一个实例**，因为用于多线程，所以常常和 **Arc** 搭配使用：
     * ```rust
     * // Mutex 需要手动上锁，超过作用于后自动解锁
     * let count = 5;
     * let mutex = Arc::new(Mutex::new(String::from("Hello")));
     * let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(count);
     * for i in 0..count {
     *     let _mutex = Arc::clone(&mutex);
     *     handles.push(thread::spawn(move || {
     *         // lock 方法申请一个锁, 该方法会阻塞当前线程，直到获取到锁，因此当多个线程同时访问该数据时，只有一个线程能获取到锁
     *         // 其它线程只能阻塞着等待，这样就保证了数据能被安全的修改！
     *         let mut s = _mutex.lock().unwrap();
     *         s.push_str(i.to_string().as_str())
     *         // 锁自动被drop
     *     }))
     * }
     *
     * for h in handles {
     *     h.join().unwrap();
     * }
     * println!("{}", mutex.lock().unwrap());
     * ```
     *
     * lock 方法申请一个锁, 该方法会阻塞当前线程，直到获取到锁，因此当多个线程同时访问该数据时，只有一个线程能获取到锁，其它线程只能阻塞着等待，这样就保证了数据能被安全的修改！
     * lock 方法也有可能报错，例如当前正在持有锁的线程 panic 了，在这种情况下，其它线程不可能再获得锁，因此 lock 方法会返回一个错误。
     *
     * `Mutex<T>` 是一个智能指针（结构体），它的方法 lock 返回另外一个智能指针（结构体） `MutexGuard<T>`，`MutexGuard<T>` 实现两个非常便捷的特征，Deref 和 Drop：
     * - Deref 特征，会被自动解引用后获得一个引用类型，该引用指向 Mutex 内部的数据
     * - Drop 特征，在超出作用域后，自动释放锁，以便其它线程能继续获取锁
     *
     * 使用 Mutex 时注意避免形成死锁：
     * ```rust
     * // 使用 mutex 注意避免形成死锁
     * let mutex = Mutex::new(3);
     * let num = mutex.lock().unwrap(); // 上锁
     * {
     *     // 由于在上一行给mutex上锁了，因此这里会一直阻塞，等待获取值的所有权，但是因为 num 没有释放，所以线程一直在阻塞，这就是死锁
     *     let _num = mutex.lock().unwrap();
     * }
     * println!("{}", num);
     * ```
     * #### 小心使用 Mutex
     * - 在使用数据前必须先获取锁
     * - 在数据使用完成后，必须及时的释放锁，例如增加作用域
     *
     * 例如：当一个操作试图锁住两个资源，然后两个线程各自获取其中一个锁，并试图获取另一个锁时，就会造成死锁（deadlock）。
     *
     * #### 内部可变性
     * 内部可变性是指当前**变量/值的空间存储的内容发生改变**的行为。
     *
     * Cell 与 RefCell 的可变借用行为并不完全一致，这是由于存储的数据类型不一样决定的：
     * Cell 和 RefCell 都是智能指针，用一个栈上的新空间存储被管理的值，不同的是 Cell 存储 Copy 类型的值，而 RefCell 存储的是非 Copy 类型的栈上指针信息（通过栈上指针信息管理堆上实际数据）。
     *
     * `Rc<T>/RefCell<T>` 用于单线程内部可变性， `Arc<T>/Mutex<T>` 用于多线程内部可变性。
     *
     * ### 死锁 deadlock
     * 死锁形成的根本原因是**带有阻塞性访问带有锁，并且已经处于锁定中的变量**，具体来看，死锁分为单线程死锁和多线程死锁。
     *
     * #### 单线程死锁
     * 单线程死锁非常容易形成，只要访问当前线程中处于锁定中的变量就会形成单线程死锁。
     * ```rust
     * // 单线程死锁
     * let mutex = Mutex::new(3);
     * // 上锁
     * let num = mutex.lock().unwrap();
     * // 由于在上一行给mutex上锁了，因此这里会一直阻塞，等待获取值的所有权，但是因为 num 没有释放，所以线程一直在阻塞，这就是死锁
     * let _num = mutex.lock().unwrap();
     * println!("{}", num);
     * ```
     *
     * #### 多线程死锁
     * 多线程死锁发生在两个线程上，有两个带锁的变量，两个线程各自使用锁定其中的一个变量后，再尝试访问另外一个锁时，就可能形成死锁。
     * 此时就形成了一线程访问锁定状态的 A 被阻塞，二线程访问锁定状态的 B 被阻塞。
     *
     * ```rust
     * // 多线程死锁
     * let count = 100;
     * let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(count);
     * let mutex1 = Arc::new(Mutex::new(1));
     * let mutex2 = Arc::new(Mutex::new(2));
     * for i in 0..count {
     *     let _mutex1 = Arc::clone(&mutex1);
     *     let _mutex2 = Arc::clone(&mutex2);
     *     handles.push(thread::spawn(move || {
     *         if i % 2 == 0 {
     *             // 锁住 mutex1 后去锁 mutex2
     *             let num1 = _mutex1.lock().unwrap();
     *             println!("线程 {} 锁住 mutex1，尝试锁住 mutex2", i);
     *             let num2 = _mutex2.lock().unwrap();
     *         } else {
     *             // 锁住 mutex2 后去锁 mutex1
     *             let num2 = _mutex2.lock().unwrap();
     *             println!("线程 {} 锁住 mutex2，尝试锁住 mutex1", i);
     *             let num1 = _mutex1.lock().unwrap();
     *         }
     *     }));
     * }
     * for h in handles {
     *     h.join().unwrap();
     * }
     * println!("没有发生死锁");
     *
     * ```
     *
     * 为何某些时候，死锁不会发生？
     * 原因很简单，线程 2 在线程 1 锁 MUTEX1 之前，就已经全部执行完了，随之线程 2 的 MUTEX2 和 MUTEX1 被全部释放，线程 1 对锁的获取将不再有竞争者，也就意味着不会被一直阻塞。
     * 同理，线程 1 若全部被执行完，那线程 2 也不会被锁一直阻塞，可以在线程 1 中间加一个睡眠，增加死锁发生的概率。如果在线程 2 中同样的位置也增加一个睡眠，那死锁将必然发生!
     *
     *
     * #### try_lock
     * 死锁的形成是因为**带有阻塞性访问带有锁，并且已经处于锁定中的变量**的阻塞，如果访问时不阻塞就意味着不会形成死锁，try_lock 就是不带阻塞的方法。
     *
     * 与 lock 方法不同，try_lock 会尝试去获取一次锁，如果无法获取会返回一个错误。
     *
     * > 一个有趣的命名规则：在 Rust 标准库中，使用 try_xxx 都会尝试进行一次操作，如果无法完成，就立即返回，不会发生阻塞。
     * > 例如消息传递章节中的 try_recv 以及本章节中的 try_lock
     *
     *
     * ### 读写锁 RwLock
     * Mutex 会对每次读写都进行加锁（即使不修改数据），但某些时候需要大量的并发读，Mutex 就无法满足需求了，此时就可以使用 RwLock。
     * RwLock 在使用上和 Mutex 区别不大，只有在多个读的情况下不阻塞程序，其他如读写、写读、写写情况下均会对后获取锁的操作进行阻塞。
     * - 同一时间允许多个读，不允许出现写
     * - 同一时间只允许一个写，不允许第二个读或写
     * 即不允许出现数据在读的过程中被改变。
     *
     * ```rust
     * let count = 100;
     * let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(count);
     * let rwlock1 = Arc::new(RwLock::new(1));
     * let rwlock2 = Arc::new(RwLock::new(2));
     * for i in 0..count {
     *     let _rwlock1 = Arc::clone(&rwlock1);
     *     let _rwlock2 = Arc::clone(&rwlock2);
     *     handles.push(thread::spawn(move || {
     *         if i % 2 == 0 {
     *             let num2 = _rwlock2.write().unwrap();
     *             println!("线程 {} 读取 rwlock1，尝试写 rwlock2", i);
     *             let num1 = _rwlock1.read().unwrap();
     *         } else {
     *             let num1 = _rwlock1.write().unwrap();
     *             println!("线程 {} 读取 rwlock2，尝试写 rwlock1", i);
     *             let num2 = _rwlock2.read().unwrap();
     *         }
     *     }));
     * }
     * for h in handles {
     *     h.join().unwrap();
     * }
     * println!("没有发生死锁");
     * ```
     *
     * 也可以使用 try_write 和 try_read 来尝试进行一次写/读，若失败则返回错误。
     *
     * 简单总结下 RwLock:
     * - 读和写不能同时存在
     * - 同一时刻允许多个读，但最多只能有一个写，且读写不能同时存在
     * - 读可以使用 read、try_read，写 write、try_write, 在实际项目中，try_xxx 会更安全
     *
     * ### Mutex 和 RwLock
     * 使用上，Mutex 比 RwLock 更简单，因为 RwLock 需要着重关注几个问题：
     * - 读和写不能同时发生，如果使用 try_xxx 解决，需要做大量的错误处理和失败重试机制
     * - 当读多写少时，写操作可能会因为一直无法获得锁导致连续多次失败 (writer starvation)
     * - RwLock 其实是操作系统提供的，实现原理要比 Mutex 复杂的多，因此单就锁的性能而言，比不上原生实现的 Mutex
     *
     * **Mutex 和 RwLock 的使用场景**
     * - 追求高并发读取时，可以使用 RwLock，因为 Mutex 一次只允许一个线程读取
     * - 如果要保证写操作的成功性，使用 Mutex
     * - 不知道哪个合适，统一使用 Mutex
     *
     * 当然，确定使用哪个锁的最好方式是做一个 benchmark。
     *
     * 使用 RwLock 要确保满足以下两个条件：**并发读和需要对读到的资源进行"长时间"的操作**。
     *
     * 所以一个常见的错误使用 RwLock 的场景就是使用 HashMap 进行简单读写。
     * 这是因为 HashMap 的读和写都非常快，HashMap 也许满足了并发读的需求，但是往往并不能满足 "长时间" 的操作这个需求，RwLock 的复杂实现和相对低的性能反而会导致整体性能的降低。
     *
     * ### 第三方库
     * 标准库在设计时总会存在取舍，因为往往性能并不是最好的，如果你追求性能，可以使用三方库提供的并发原语:
     * - parking_lot, 功能更完善、稳定，社区较为活跃，star 较多，更新较为活跃
     * - spin, 在多数场景中性能比parking_lot高一点，最近没怎么更新
     *
     * ### 条件变量控制线程同步
     * Mutex 与 Arc 的搭配可以解决多线程下资源安全访问的问题，在这个基础上 rust 还提供了一个条件变量（Condition Variable）用于控制资源的访问顺序。
     *
     * 条件变量（Condition Variable）搭配 Mutex 和 Arc，可以做到控制线程执行流程，让线程挂起直至某个条件满足后再继续运行。
     *
     * 比如，让两个线程内部的循环交替输出相同的序号，这里先用一个条件变量和线程休眠实现一个简单版本：
     * ```rust
     * // 用一个条件变量和线程休眠实现一个简单版本的交替输出
     * let cond = Arc::new(Condvar::new());
     * let mutex = Arc::new(Mutex::new(true));
     * let _cond = Arc::clone(&cond);
     * let _mutex = Arc::clone(&mutex);
     * let count = 3;
     *
     * let handle = thread::spawn(move || {
     *     let mut lock = _mutex.lock().unwrap();
     *
     *     for i in 0..count {
     *         while *lock == false {
     *             lock = _cond.wait(lock).unwrap(); // 阻塞线程，等待条件变量的通知后继续运行，并将最新的值赋值给锁
     *         }
     *
     *         *lock = false; // 重置条件，重新进入阻塞等待条件变量的调度
     *         println!("inner index = {}", i);
     *     }
     * });
     *
     * for i in 0..count {
     *     // 用线程休眠模拟另外一个条件，阻塞当前运行，然后恢复继续运行
     *     // 这里先休眠是为了让子线程进入条件阻塞状态
     *     println!("outer index = {}", i);
     *     thread::sleep(Duration::from_millis(100));
     *     // println!("outer index = {}", i); 调整输出位置，可以观察到交替顺序变换
     *     let mut lock = mutex.lock().unwrap();
     *     *lock = true;
     *     cond.notify_one(); // 通知另外一个线程可以继续运行
     * }
     *
     * handle.join().unwrap();
     * ```
     *
     * 一个条件变量的版本：
     * ```rust
     * // 一个互斥锁和条件变量对，用于线程间的同步
     * let count = 3;
     * let pair1 = Arc::new((Mutex::new(false), Condvar::new()));
     * let pair2 = Arc::clone(&pair1);
     * let handle = thread::spawn(move || {
     *     let (lock, cvar) = &*pair2;
     *     for i in 0..count {
     *         let mut started = lock.lock().unwrap();
     *         while !*started {
     *             // 等待主线程的通知
     *             started = cvar.wait(started).unwrap();
     *         }
     *         println!("inner index = {}", i);
     *         *started = false; // 重置条件
     *         cvar.notify_one(); // 通知主线程
     *     }
     * });
     * let (lock, cvar) = &*pair1;
     * for i in 0..count {
     *     let mut started = lock.lock().unwrap();
     *     *started = true; // 设置条件
     *     cvar.notify_one(); // 通知子线程，需要放在条件变量阻塞之前，否则会造成死锁
     *     while *started {
     *         // 等待子线程通知
     *         started = cvar.wait(started).unwrap();
     *     }
     *     println!("outer index = {}", i);
     * }
     * handle.join().unwrap(); // 等待子线程完成
     * ```
     *
     * ### 信号量 Semaphore
     * 在多线程中，另一个重要的概念就是信号量，使用它可以让我们精准的控制当前正在运行的任务最大数量。信号量可以看成一个池，如常见的线程池、连接池等。
     *
     * 想象一下，当一个新游戏刚开服时，往往会控制游戏内玩家的同时在线数，一旦超过某个临界值，就开始进行排队进服。
     * 而在实际使用中，有很多时候需要通过信号量来控制最大并发数，防止服务器资源被撑爆。
     *
     * 本来 Rust 在标准库中有提供一个信号量实现, 但是由于各种原因这个库现在已经不再推荐使用了，推荐使用 tokio 中提供的 Semaphore 实现 tokio::sync::Semaphore。
     *
     * 这里先认识 `async move {}` 和 `async move || {}` 的区别：
     * - `async move {}`：直接定义一个异步块，立即捕获环境变量并生成 Future，适用于需要单次执行的异步操作。
     * - `async move || {}`：定义一个异步闭包，每次调用该闭包时生成一个新的 Future，并捕获当前调用环境中的变量,适用于需要多次调用的异步函数。
     *
     * 选择哪种形式取决于具体需求以及代码的应用场景。如果需要创建可复用的异步函数，async move || {} 更合适；如果只需要一次性执行的异步逻辑，async move {} 会更简洁。
     *
     * > Future 是一个核心概念，用于表示一个异步操作的结果，它可能在将来某个时刻完成。Future 可以被视为一种承诺（promise），它将在未来某个时间点提供一个值或错误。
     * > 当编写异步代码时，如果希望某些操作能够在不阻塞当前线程的情况下执行，并且在这些操作完成后获得其结果。Future 提供了一种机制，可以描述这些异步操作，并在它们完成时得到通知。
     * >
     * > 这个概念和 JavaScript 的 Promise 非常相似，表示一个将来可能会产生结果的异步操作，优点：
     * > - 非阻塞：Future 允许异步代码在不阻塞线程的情况下执行，使得应用程序可以处理更多并发任务
     * > - 组合性：可以通过组合多个 Future 来构建复杂的异步控制流
     * > - 可读性：使用 async/await 语法，使得异步代码看起来像同步代码，更加易读和易维护
     *
     * ```rust
     * #[tokio::main]
     * async fn main() {
     *     let semaphore = Arc::new(Semaphore::new(3));
     *     let mut handles = Vec::new();
     *     let count = 5;
     *     for i in 0..count {
     *         // acquire_owned 申请许可，申请通过则线程运行，否则线程被阻塞，直至获得许可后才会解除阻塞继续运行
     *         let _semaphore = Arc::clone(&semaphore);
     *         handles.push(tokio::spawn(async move {
     *             println!("{} 未获取 permit 许可", i);
     *             let permit = _semaphore.acquire().await.unwrap();
     *             println!("{} 已获取 permit 许可", i);
     *             tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
     *             println!("{} 运行结束", i);
     *             // drop(permit); 在离开作用域时自动释放
     *         }));
     *     }
     *     for h in handles {
     *         h.await.unwrap();
     *     }
     * }
     * ```
     *
     * 使用信号量过程中需要申请和归还，使用前需要申请信号量，如果容量满了，就需要等待；使用后需要释放信号量，以便其它等待者可以继续。
     * 
     * 
     * ### 总结
     * 在很多时候消息传递都是优雅解决并发问题的方式，但是它也并不能优雅的解决所有问题，因为面临的真实世界是非常复杂的，无法用某一种银弹统一解决。
     * 当面临消息传递不太适用的场景时，或者需要更好的性能和简洁性时，往往需要用锁来解决这些问题，因为锁允许多个线程同时访问同一个资源，简单粗暴。
     * 
     *
     */
    let count = 5;
    let mutex = Arc::new(Mutex::new(String::from("Hello")));
    let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(count);
    for i in 0..count {
        let _mutex = Arc::clone(&mutex);
        handles.push(thread::spawn(move || {
            // lock 方法申请一个锁, 该方法会阻塞当前线程，直到获取到锁，因此当多个线程同时访问该数据时，只有一个线程能获取到锁
            // 其它线程只能阻塞着等待，这样就保证了数据能被安全的修改！
            let mut s: std::sync::MutexGuard<String> = _mutex.lock().unwrap();
            s.push_str(i.to_string().as_str());
            // 锁自动被drop
        }))
    }

    for h in handles {
        h.join().unwrap();
    }
    println!("{}", mutex.lock().unwrap());

    // 使用 mutex 注意避免形成死锁
    let mutex = Mutex::new(3);
    let num = mutex.lock().unwrap(); // 上锁
    {
        // 由于在上一行给mutex上锁了，因此这里会一直阻塞，等待获取值的所有权，但是因为 num 没有释放，所以线程一直在阻塞，这就是死锁
        // let _num = mutex.lock().unwrap();
    }
    println!("{}", num);

    // 单线程死锁
    let mutex = Mutex::new(3);
    // 上锁
    let num = mutex.lock().unwrap();
    // 由于在上一行给mutex上锁了，因此这里会一直阻塞，等待获取值的所有权，但是因为 num 没有释放，所以线程一直在阻塞，这就是死锁
    // let _num = mutex.lock().unwrap();
    println!("{}", num);

    // 多线程死锁
    // let count = 100;
    // let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(count);
    // let mutex1 = Arc::new(Mutex::new(1));
    // let mutex2 = Arc::new(Mutex::new(2));
    // for i in 0..count {
    //     let _mutex1 = Arc::clone(&mutex1);
    //     let _mutex2 = Arc::clone(&mutex2);
    //     handles.push(thread::spawn(move || {
    //         if i % 2 == 0 {
    //             // 锁住 mutex1 后去锁 mutex2
    //             let num1 = _mutex1.lock().unwrap();
    //             println!("线程 {} 锁住 mutex1，尝试锁住 mutex2", i);
    //             let num2 = _mutex2.lock().unwrap();
    //         } else {
    //             // 锁住 mutex2 后去锁 mutex1
    //             let num2 = _mutex2.lock().unwrap();
    //             println!("线程 {} 锁住 mutex2，尝试锁住 mutex1", i);
    //             let num1 = _mutex1.lock().unwrap();
    //         }
    //     }));
    // }
    // for h in handles {
    //     h.join().unwrap();
    // }
    // println!("没有发生死锁");

    // try_lock 不阻塞的方法
    // let count = 100;
    // let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(count);
    // let mutex1 = Arc::new(Mutex::new(1));
    // let mutex2 = Arc::new(Mutex::new(2));
    // for i in 0..count {
    //     let _mutex1 = Arc::clone(&mutex1);
    //     let _mutex2 = Arc::clone(&mutex2);
    //     handles.push(thread::spawn(move || {
    //         if i % 2 == 0 {
    //             // 锁住 mutex1 后去锁 mutex2
    //             let num1 = _mutex1.try_lock().unwrap();
    //             println!("线程 {} 锁住 mutex1，尝试锁住 mutex2", i);
    //             let num2 = _mutex2.try_lock().unwrap();
    //         } else {
    //             // 锁住 mutex2 后去锁 mutex1
    //             let num2 = _mutex2.try_lock().unwrap();
    //             println!("线程 {} 锁住 mutex2，尝试锁住 mutex1", i);
    //             let num1 = _mutex1.try_lock().unwrap();
    //         }
    //     }));
    // }
    // for h in handles {
    //     h.join().unwrap();
    // }
    // println!("没有发生死锁");

    // RwLock 读写锁支持并发读
    // let count = 10000;
    // let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(count);
    // let rwlock1 = Arc::new(RwLock::new(1));
    // let rwlock2 = Arc::new(RwLock::new(2));
    // for i in 0..count {
    //     let _rwlock1 = Arc::clone(&rwlock1);
    //     let _rwlock2 = Arc::clone(&rwlock2);
    //     handles.push(thread::spawn(move || {
    //         if i % 2 == 0 {
    //             let num2 = _rwlock2.write().unwrap();
    //             println!("线程 {} 读取 rwlock1，尝试写 rwlock2", i);
    //             let num1 = _rwlock1.read().unwrap();
    //         } else {
    //             let num1 = _rwlock1.write().unwrap();
    //             println!("线程 {} 读取 rwlock2，尝试写 rwlock1", i);
    //             let num2 = _rwlock2.read().unwrap();
    //         }
    //     }));
    // }
    // for h in handles {
    //     h.join().unwrap();
    // }
    // println!("没有发生死锁");

    // 用一个条件变量和线程休眠实现一个简单版本的交替输出
    // let cond = Arc::new(Condvar::new());
    // let mutex = Arc::new(Mutex::new(true));
    // let _cond = Arc::clone(&cond);
    // let _mutex = Arc::clone(&mutex);
    // let count = 3;
    // let handle = thread::spawn(move || {
    //     let mut lock = _mutex.lock().unwrap();
    //     for i in 0..count {
    //         while *lock == false {
    //             lock = _cond.wait(lock).unwrap(); // 阻塞线程，等待条件变量的通知后继续运行，并将最新的值赋值给锁
    //         }
    //         *lock = false; // 重置条件，重新进入阻塞等待条件变量的调度
    //         println!("inner index = {}", i);
    //     }
    // });
    // for i in 0..count {
    //     // 用线程休眠模拟另外一个条件，阻塞当前运行，然后恢复继续运行
    //     // 这里先休眠是为了让子线程进入条件阻塞状态
    //     println!("outer index = {}", i);
    //     thread::sleep(Duration::from_millis(100));
    //     // println!("outer index = {}", i); 调整输出位置，可以观察到交替顺序变换
    //     let mut lock = mutex.lock().unwrap();
    //     *lock = true;
    //     cond.notify_one(); // 通知另外一个线程可以继续运行
    // }
    // handle.join().unwrap();

    // 一个互斥锁和条件变量对，用于线程间的同步
    let count = 3;
    let pair1 = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = Arc::clone(&pair1);
    let handle = thread::spawn(move || {
        let (lock, cvar) = &*pair2;
        for i in 0..count {
            let mut started = lock.lock().unwrap();
            while !*started {
                // 等待主线程的通知
                started = cvar.wait(started).unwrap();
            }
            println!("inner index = {}", i);
            *started = false; // 重置条件
            cvar.notify_one(); // 通知主线程
        }
    });
    let (lock, cvar) = &*pair1;
    for i in 0..count {
        let mut started = lock.lock().unwrap();
        *started = true; // 设置条件
        cvar.notify_one(); // 通知子线程，需要放在条件变量阻塞之前，否则会造成死锁
        while *started {
            // 等待子线程通知
            started = cvar.wait(started).unwrap();
        }
        println!("outer index = {}", i);
    }
    handle.join().unwrap(); // 等待子线程完成

    // tokio 的信号量，可以视为连接池，通过 acquire_owned 申请许可，申请通过则线程运行，否则线程被阻塞，直至获得许可后才会解除阻塞继续运行。
    let semaphore = Arc::new(Semaphore::new(3));
    let mut handles = Vec::new();
    let count = 5;
    for i in 0..count {
        // acquire_owned 申请许可，申请通过则线程运行，否则线程被阻塞，直至获得许可后才会解除阻塞继续运行
        let _semaphore = Arc::clone(&semaphore);
        handles.push(tokio::spawn(async move {
            println!("{} 未获取 permit 许可", i);
            let permit = _semaphore.acquire().await.unwrap();
            println!("{} 已获取 permit 许可", i);
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            println!("{} 运行结束", i);
            // drop(permit); 在离开作用域时自动释放
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
}
