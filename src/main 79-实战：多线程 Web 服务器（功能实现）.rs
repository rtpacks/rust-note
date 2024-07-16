use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net, thread,
    time::Duration,
};

use ilearn::threadpool::ThreadPool;

use futures::stream;

fn main() {
    /*
     *
     * ## 实战：多线程 Web 服务器（功能实现）
     * 单线程版本只能依次处理用户的请求：同一时间只能处理一个请求连接。随着用户的请求数增多，可以预料的是排在后面的用户可能要等待数十秒甚至超时！
     *
     * ### 模拟慢请求
     * 在单线程版本中，为一个请求增加 5 秒阻塞，前一个请求发起后，再一次发起访问请求，第二个请求就需要等待两个阻塞时间间隔，也就是 10s。
     *
     * ```rust
     * let listener =
     *     net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");
     * for stream in listener.incoming() {
     *     let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
     *     println!("Connection established!");
     *
     *     let buf_reader = BufReader::new(&stream);
     *     let http_request: Vec<_> = buf_reader
     *         .lines()
     *         .map(|line| line.unwrap())
     *         .take_while(|line| !line.is_empty())
     *         .collect();
     *
     *     let (status_line, html) = if &http_request[0] == "GET / HTTP/1.1" {
     *         (
     *             "HTTP/1.1 200 OK",
     *             fs::read_to_string(r"public/http-response-index.html").unwrap(),
     *         )
     *     } else {
     *         (
     *             "HTTP/1.1 404 NOT FOUND",
     *             fs::read_to_string(r"public/http-response-404.html").unwrap(),
     *         )
     *     };
     *
     *     let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
     *     let response_body = html;
     *     let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");
     *
     *     thread::sleep(Duration::from_secs(5));
     *     stream.write_all(http_response.as_bytes());
     * }
     * ```
     *
     * 单线程是处理是很不合理的，需要解决这个问题。
     *
     * ### 多线程提高吞吐量
     * 线程池（thread pool）是一组预先分配的等待或准备处理任务的线程。线程池允许程序并发处理连接，增加 server 的吞吐量。
     * 当程序收到一个新任务时就会指派线程池中的一个线程离开并处理该任务。
     * 当线程仍在处理任务时，新来的任务会交给池中剩余的线程进行处理。当线程处理完任务时，它会返回空闲线程池中等待处理新任务。
     *
     * 同时，线程池需要限制为较少的数量的线程，以防拒绝服务攻击（Denial of Service，DoS）。
     * 假设程序为每一个接收的请求都新建一个线程，那么某人向 server 发起千万级的请求时会耗尽服务器的资源并导致所有请求的处理都被终止。
     *
     * 当然，线程池依然是较为传统的提升吞吐方法，比较新的有单线程异步 IO，例如 redis；多线程异步 IO，例如 Rust 的主流 web 框架。
     *
     * 为每个请求生成一个线程，这种做法难以控制且消耗资源：
     * ```rust
     * let listener =
     *     net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");
     *
     * fn handle_request(mut stream: net::TcpStream) {
     *     let buf_reader = BufReader::new(&stream);
     *     let http_request: Vec<_> = buf_reader
     *         .lines()
     *         .map(|line| line.unwrap())
     *         .take_while(|line| !line.is_empty())
     *         .collect();
     *
     *     let (status_line, html) = if &http_request[0] == "GET / HTTP/1.1" {
     *         (
     *             "HTTP/1.1 200 OK",
     *             fs::read_to_string(r"public/http-response-index.html").unwrap(),
     *         )
     *     } else {
     *         (
     *             "HTTP/1.1 404 NOT FOUND",
     *             fs::read_to_string(r"public/http-response-404.html").unwrap(),
     *         )
     *     };
     *
     *     let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
     *     let response_body = html;
     *     let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");
     *
     *     thread::sleep(Duration::from_secs(5));
     *     stream.write_all(http_response.as_bytes());
     * }
     *
     * for stream in listener.incoming() {
     *     let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
     *     println!("Connection established!");
     *
     *     // 每个请求都生成一个新线程去处理任务，这种做法开销过大，在请求量大时，很容易造成资源不足
     *     let handle = thread::spawn(move || {
     *         handle_request(stream);
     *     });
     * }
     * ```
     *
     * 设想给出一个线程池，存储已经生成好的线程，当任务到达后可以直接从线程池中取出线程运行任务，这样避免了等待线程生成的时间。同时在任务结束后不会销毁线程，而是将线程归还给线程池，用于下一次任务处理。
     * 此外为避免线程数量急速增加，可以设置线程池的线程数量。通过线程池，可以避免每个请求都生成一个新线程方案的很多问题。
     *
     * 在开始之前，这里有一种开发模式，与前后端先接口约定后具体开发的模式、设计数据库表画出 ER 图的流程是类似的，都是先设想整体与局部的功能划分，然后再具体实现局部的功能。
     *
     * 模式描述：在最初就约定客户端接口有助于指导代码设计。以期望的调用方式来构建 API 代码的结构，接着在这个结构之内实现功能。
     * 这种模式称为编译器驱动开发（compiler-driven development）。
     *
     * 具体行为：编写调用所期望的函数的代码，接着观察编译器错误然后决定接下来需要修改什么使得代码可以工作。
     * 这一种方式并不会探索作为起点的技术，而是按照业务流程一步一步补齐。
     *
     * ```rust
     * let listener =
     *     net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");
     *
     * // 生成有 5 个线程的线程池
     * let pool = ThreadPool::new(5);
     *
     * ...
     * ```
     *
     * > 为什么使用 new 而不是 build？
     * > new 往往用于简单初始化一个实例，而 build 往往会完成更加复杂的构建工作。因此这里更适合使用 new 名称。
     *
     * 在 lib.rs 中声明线程池结构体和 new 方法，并导入使用：
     * ```rust
     * // lib.rs
     * pub struct ThreadPool {}
     *
     * impl ThreadPool {
     *     pub fn new(size: usize) -> Self {
     *         ThreadPool {}
     *     }
     * }
     * ```
     *
     * 以上的代码还少了一个步骤：当有任务到达时，线程池需要一个**方法**去调用线程执行任务。
     * 类比多线程函数 `thread::spawn`，推测线程池提供的执行方法参数应该是一个闭包，闭包内部执行 `handle_request` 函数。
     * ```rust
     * fn execute(closure: F) {}
     *
     * pool.execute(|| { handle_request() });
     * ```
     *
     * 其中 `execute` 函数的闭包参数类型可以参考 `thread::spawn` 函数的闭包声明：
     * ```rust
     * pub fn spawn<F, T>(f: F) -> JoinHandle<T>
     * where
     *     F: FnOnce() -> T,
     *     F: Send + 'static,
     *     T: Send + 'static,
     * {
     *     Builder::new().spawn(f).expect("failed to spawn thread")
     * }
     *
     * pub fn execute<F>(&self, f: F)
     * where
     *     // 泛型参数形式
     *     // 泛型参数：编译时确定闭包类型，性能更好，无需动态分发。
     *     // 特征对象：运行时确定闭包类型，灵活但有额外开销。
     *     F: FnOnce() + Send + 'static,
     * {
     * }
     * ```
     * 这个类型并不是一个具体的数据类型，它只是一个泛型的限制，也就是可以看成一个特征，在之前提到过，数据通过数据类型来限制，数据类型通过泛型来限制，泛型通过特征来限制。
     *
     * #### 存储线程
     * 以上梳理了整体框架，现在考虑线程池怎么存储线程。`thread::spawn` 是创建线程的函数，观察该函数的返回值就可以得到线程的类型 `JoinHandle<T>`。
     * ```rust
     * pub fn spawn<F, T>(f: F) -> JoinHandle<T>
     * where
     *     F: FnOnce() -> T,
     *     F: Send + 'static,
     *     T: Send + 'static,
     * {
     *     Builder::new().spawn(f).expect("failed to spawn thread")
     * }
     * ```
     *
     * 使用一个 `Vec<E>` 存储线程，在合适时取出线程让其执行任务。其中 `E` 是 `JoinHandle<T>`，T 在这个案例中为单元类型 `()`，即返回值为单元类型的线程。
     *
     * ```rust
     * pub struct ThreadPool {
     *     threads: Vec<JoinHandle<()>>,
     * }
     *
     * impl ThreadPool {
     *     /// Create a new ThreadPool.
     *     ///
     *     /// The size is the number of threads in the pool.
     *     ///
     *     /// ## Panics
     *     ///
     *     /// The `new` function will panic if the size is zero.
     *     pub fn new(size: usize) -> Self {
     *         assert!(size > 0);
     *
     *         let mut threads = Vec::with_capacity(size);
     *         ThreadPool { threads }
     *     }
     *
     *     pub fn execute<F>(&self, f: F)
     *     where
     *         // 泛型参数形式
     *         // 泛型参数：编译时确定闭包类型，性能更好，无需动态分发。
     *         // 特征对象：运行时确定闭包类型，灵活但有额外开销。
     *         F: FnOnce() + Send + 'static,
     *     {
     *     }
     * }
     * ```
     * 现在线程池已经可以存储线程，但是还剩下几个关键问题：
     * 1. 在生成 `ThreadPool` 时没有生成线程，即没有调用 `thread::spawn` 函数，`ThreadPool::threads` 还是空的
     * 2. `thread::spawn` 创建线程时是立即执行闭包的，直接传递给 `thread::spawn` 的闭包无法修改
     * 3. 在主线程受到任务时，怎么将任务传递给线程池中的线程
     *
     * 问题 1 和问题 2 说明需要在生成线程池时需要一个“写死”的立即执行闭包。
     * 问题 2 和 问题3 说明这个立即执行闭包要具有从主线程接收新任务的能力，它是带有循环的，当前任务执行完成后会等待新任务。
     *
     * 在生成线程时“写死”的立即执行闭包代码与立即执行闭包具有接收新任务的能力，两者的结合点在“写死”的代码具有从某处循环接收任务的逻辑，这样就能做到既“写死”又可动态接收。
     *
     * 生成线程时，让线程处于循环接收的状态中：
     * ```rust
     * pub fn new(size: usize) -> Self {
     *     assert!(size > 0);
     *
     *     let mut threads = Vec::with_capacity(size);
     *
     *     for i in 0..size {
     *         threads.push(thread::spawn(|| {
     *             while true {
     *                 // 为了减缓轮询的压力，控制轮询时间
     *                 thread::sleep(Duration::from_secs(1));
     *
     *                 if (jobs.len() > 0) {
     *                     let job = jobs.pop();
     *                     job();
     *                 }
     *             }
     *         }))
     *     }
     *
     *     ThreadPool { threads }
     * }
     * ```
     *
     * 现在看起来是合理的，从一个 jobs 任务队列中获取任务，然后开始任务执行。当然在多线程的环境下需要考虑多线程所有权和多线程并发问题，即需要使用 `Arc` 和 `Mutex`。
     *
     * ### 消息通道通信
     * 轮询一个队列获取信息属于从共享内存中通信，在之前提到过：不要通过共享内存来通信，而是通过通信来共享内存，将轮询改成消息通道将一定程度上会降低代码实现的复杂度。
     *
     * > 在 Go 语言中有一句很经典的话：
     * > Do not communicate by sharing memory; instead, share memory by communicating
     * > 不要通过共享内存来进行通信，而是通过通信来共享内存
     * >
     * > 简单理解：尽量避免访问同一块内存空间来通信，因为它会造成的并发问题如竞争条件（Race condition），死锁（Deadlocks）等。
     * > 而是应该通过消息通知（触发）进行数据传递，例如消息队列、Socket 等方法。不同进程或线程之间通过这些通信机制共享数据，避免共享内存造成的并发问题。
     *
     *
     * 将构建线程池函数 `ThreadPool::new` 中的轮询逻辑转换成消息队列：
     * ```rust
     * pub fn new(size: usize) -> Self {
     *     assert!(size > 0);
     *
     *     let mut threads = Vec::with_capacity(size);
     *     let (mut sender, mut receiver) = mpsc::channel();
     *
     *     for i in 0..size {
     *         threads.push(thread::spawn(move || {
     *             for job in receiver.recv() {
     *
     *             }
     *         }))
     *     }
     *
     *     ThreadPool { threads }
     * }
     * ```
     *
     * 这里少了一个消息通道传递的数据类型，这个类型就是主线程传递的闭包类型，在线程池的 `execute` 函数中已经推出的闭包类型约束 `where F: FnOnce() + Send + 'static`。
     * 但是这个类型约束并不是一个具体的数据类型，它只是一个泛型的限制，也就是可以看成一个特征，在之前提到过，数据通过数据类型来限制，数据类型通过泛型来限制，泛型通过特征来限制。
     *
     * 消息通道传递的数据的类型要求是一个具体的数据类型，将特征改成特征对象即可：
     * ```rust
     * pub type Job = Box<dyn FnOnce() + Send + 'static>;
     *
     * // ThreadPool::new
     * pub fn new(size: usize) -> Self {
     *     assert!(size > 0);
     *
     *     let mut threads = Vec::with_capacity(size);
     *     let (mut sender, mut receiver) = mpsc::channel::<Job>();
     *
     *     for i in 0..size {
     *         threads.push(thread::spawn(move || {
     *             for job in receiver.recv() {
     *
     *             }
     *         }))
     *     }
     *
     *     ThreadPool { threads }
     * }
     * ```
     *
     * 除了消息通道类型，这里还有一个借用规则的限制。在生成的多个线程中都使用了接收者，需要在多线程中共享数据并且要保证内存安全，应该用 `Arc<Mutex>` 包裹起来：
     * ```rust
     * // ThreadPool::new
     * pub fn new(size: usize) -> Self {
     *     assert!(size > 0);
     *
     *     let mut threads = Vec::with_capacity(size);
     *     let (mut sender, mut receiver) = mpsc::channel::<Job>();
     *     let receiver = Arc::new(Mutex::new(receiver));
     *
     *     for i in 0..size {
     *         let _receiver = Arc::clone(&receiver);
     *         threads.push(thread::spawn(move || {
     *             for job in _receiver.lock().unwrap().recv() {
     *                 println!("Got a job; executing.");
     *                 job();
     *             }
     *         }))
     *     }
     *
     *     ThreadPool { threads }
     * }
     * ```
     *
     * 主线程通过 `ThreadPool::execute` 函数传递的需要执行的闭包，线程池中的线程通过消息通道接收需要执行的闭包，也就是 `ThreadPool::execute` 中需要一个发送闭包的发送者。
     * 但是发送者实在构造线程池中生成的，需要将其附加到线程池中，线程池的方法才能使用。
     *
     * > 为什么不在外部构造？因为没有必要对外暴露是通过消息通道实现的，如果将消息通道的生成放在外部，这个代码的拆分并没有太大的意义，因为这都是耦合代码。
     *
     * ```rust
     * pub type Job = Box<dyn FnOnce() + Send + 'static>;
     * pub struct ThreadPool {
     *     threads: Vec<JoinHandle<()>>,
     *     sender: Sender<Job>,
     * }
     *
     * impl ThreadPool {
     *     pub fn new(size: usize) -> Self {
     *        ...
     *     }
     *
     *     pub fn execute<F>(&self, f: F)
     *     where
     *         // 泛型参数形式
     *         // 泛型参数：编译时确定闭包类型，性能更好，无需动态分发。
     *         // 特征对象：运行时确定闭包类型，灵活但有额外开销。
     *         F: FnOnce() + Send + 'static,
     *     {
     *         // 传递特征对象，因为函要求定长类型，特征属于非定长的类型
     *         let box_f = Box::new(f);
     *         self.sender.send(box_f);
     *     }
     * }
     * ```
     *
     * 尝试一下线程池，lib.rs：
     * ```rust
     * // lib.rs
     * use std::{
     *     sync::{
     *         mpsc::{self, Sender},
     *         Arc, Mutex,
     *     },
     *     thread::{self, JoinHandle},
     * };
     *
     * pub type Job = Box<dyn FnOnce() + Send + 'static>;
     * pub struct ThreadPool {
     *     threads: Vec<JoinHandle<()>>,
     *     sender: Sender<Job>,
     * }
     *
     * impl ThreadPool {
     *     /// Create a new ThreadPool.
     *     ///
     *     /// The size is the number of threads in the pool.
     *     ///
     *     /// ## Panics
     *     ///
     *     /// The `new` function will panic if the size is zero.
     *     pub fn new(size: usize) -> Self {
     *         assert!(size > 0);
     *
     *         let mut threads = Vec::with_capacity(size);
     *         let (mut sender, mut receiver) = mpsc::channel::<Job>();
     *         let receiver = Arc::new(Mutex::new(receiver));
     *
     *         for i in 0..size {
     *             let _receiver = Arc::clone(&receiver);
     *             threads.push(thread::spawn(move || loop {
     *                 for job in _receiver.lock().unwrap().recv() {
     *                     println!("index: {i} got a job; executing.");
     *                     job();
     *                 }
     *             }))
     *         }
     *
     *         ThreadPool { threads, sender }
     *     }
     *
     *     pub fn execute<F>(&self, f: F)
     *     where
     *         // 泛型参数形式
     *         // 泛型参数：编译时确定闭包类型，性能更好，无需动态分发。
     *         // 特征对象：运行时确定闭包类型，灵活但有额外开销。
     *         F: FnOnce() + Send + 'static,
     *     {
     *         // 传递特征对象，因为函要求定长类型，特征属于非定长的类型
     *         let box_f = Box::new(f);
     *         self.sender.send(box_f);
     *     }
     * }
     * ```
     *
     * main.rs
     * ```rust
     * let listener =
     *     net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");
     *
     * // 生成有 5 个线程的线程池
     * let thread_pool = ThreadPool::new(5);
     *
     * fn handle_request(mut stream: net::TcpStream) {
     *     let buf_reader = BufReader::new(&stream);
     *     let http_request: Vec<_> = buf_reader
     *         .lines()
     *         .map(|line| line.unwrap())
     *         .take_while(|line| !line.is_empty())
     *         .collect();
     *
     *     let (status_line, html) = if &http_request[0] == "GET / HTTP/1.1" {
     *         (
     *             "HTTP/1.1 200 OK",
     *             fs::read_to_string(r"public/http-response-index.html").unwrap(),
     *         )
     *     } else {
     *         (
     *             "HTTP/1.1 404 NOT FOUND",
     *             fs::read_to_string(r"public/http-response-404.html").unwrap(),
     *         )
     *     };
     *
     *     let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
     *     let response_body = html;
     *     let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");
     *
     *     thread::sleep(Duration::from_secs(5));
     *     stream.write_all(http_response.as_bytes());
     * }
     *
     * for stream in listener.incoming() {
     *     let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
     *     println!("Connection established!");
     *
     *     thread_pool.execute(|| handle_request(stream))
     * }
     * ```
     *
     * ### Mutex 释放锁
     * 运行后会发现有一个非常大的问题，同一时间还是只有一个线程在运行任务，这个问题是由 `Mutex` 释放锁不正确导致的。
     * Mutex 没有提供显式的 unlock 方法，它依赖于作用域的结束去释放锁。`while let, for in` 他们形成的是作用域快，在当前用例中只有 job 结束之后才会释放锁。
     *
     * 这样导致的即使已经有新任务到达，但是因为 Mutex 锁住了 receiver，导致其他线程无法使用 receiver，无法接收运行任务，
     * 只有等当前线程结束后，离开作用域自动释放 Mutex，其他线程才有机会使用 receiver，才能运行任务。
     *
     * 所以使用 `while let, for in` 这种方式还是类似单线程，同时运行的只有一个线程，因为接收者的锁没有正确的及时释放。
     * ```rust
     * let _receiver = Arc::clone(&receiver);
     * threads.push(thread::spawn(move || loop {
     *     for job in _receiver.lock().unwrap().recv() {
     *         println!("index: {i} got a job; executing.");
     *         job();
     *     }
     * }))
     * ```
     *
     * 改造释放锁的逻辑：
     * ```rust
     * let _receiver = Arc::clone(&receiver);
     * threads.push(thread::spawn(move || loop {
     *     let job = _receiver.lock().unwrap().recv();
     *     println!("index: {i} got a job; executing.");
     *     job.unwrap()();
     * }));
     * ```
     *
     * 问题探索与解答：https://github.com/sunface/rust-course/discussions/1193#discussioncomment-6097170
     *
     * >《The Rust Programming Language》中的第 20.2 章节《Turning Our Single-Threaded Server into a Multithreaded Server》：
     * > The code in Listing 20-20 that uses `let job = receiver.lock().unwrap().recv().unwrap();` works because with let,
     * > any temporary values used in the expression on the right hand side of the equals sign are **immediately dropped** when the let statement ends.
     * >
     * > However, while let (and if let and match) does not drop temporary values until the end of the associated block.
     * > In Listing 20-21, the lock remains held for the duration of the call to job(), meaning other workers cannot receive jobs.
     * >
     * > 中文
     * > 示例 20-20 中的代码使用的 let job = receiver.lock().unwrap().recv().unwrap(); 之所以可以工作是因为对于 let 来说，当 let 语句结束时任何表达式中等号右侧使用的临时值都会立即被丢弃。然而 while let（if let 和 match）直到相关的代码块结束都不会丢弃临时值。在示例 20-21 中，job() 调用期间锁一直持续，这也意味着其他的 worker 无法接受任务。
     *
     * 总结：单独使用 let 声明一个变量，它会丢弃等号右边除最后一个值外的其它所有的临时变量；
     * 而对于 if let、while let 或 match，只有当它的整个作用域结束时，才会丢弃等号右边除最后一个值外的其它所有的临时变量。
     *
     * 如果要用一个终极规则或语法来说的话，就是无论是 let，还是 if let、while let 、match，只是在它的作用域结束时，才会丢弃等号右边除最后一个值外的其它所有的临时变量。
     * - 对于 let，它并不开启一个子作用域，而是使用它所在的作用域范围；
     * - 对于 if let、while let 、match，它们会开启一个新的子作用域，所以要等到子作用域结束。
     *
     * reference：https://github.com/sunface/rust-course/discussions/1193#discussioncomment-9452236
     *
     */

    // {
    //     let listener =
    //         net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");
    //     for stream in listener.incoming() {
    //         let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
    //         println!("Connection established!");

    //         let buf_reader = BufReader::new(&stream);
    //         let http_request: Vec<_> = buf_reader
    //             .lines()
    //             .map(|line| line.unwrap())
    //             .take_while(|line| !line.is_empty())
    //             .collect();

    //         let (status_line, html) = if &http_request[0] == "GET / HTTP/1.1" {
    //             (
    //                 "HTTP/1.1 200 OK",
    //                 fs::read_to_string(r"public/http-response-index.html").unwrap(),
    //             )
    //         } else {
    //             (
    //                 "HTTP/1.1 404 NOT FOUND",
    //                 fs::read_to_string(r"public/http-response-404.html").unwrap(),
    //             )
    //         };

    //         let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
    //         let response_body = html;
    //         let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");

    //         thread::sleep(Duration::from_secs(5));
    //         stream.write_all(http_response.as_bytes());
    //     }
    // }

    // {
    //     let listener =
    //         net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");

    //     fn handle_request(mut stream: net::TcpStream) {
    //         let buf_reader = BufReader::new(&stream);
    //         let http_request: Vec<_> = buf_reader
    //             .lines()
    //             .map(|line| line.unwrap())
    //             .take_while(|line| !line.is_empty())
    //             .collect();

    //         let (status_line, html) = if &http_request[0] == "GET / HTTP/1.1" {
    //             (
    //                 "HTTP/1.1 200 OK",
    //                 fs::read_to_string(r"public/http-response-index.html").unwrap(),
    //             )
    //         } else {
    //             (
    //                 "HTTP/1.1 404 NOT FOUND",
    //                 fs::read_to_string(r"public/http-response-404.html").unwrap(),
    //             )
    //         };

    //         let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
    //         let response_body = html;
    //         let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");

    //         thread::sleep(Duration::from_secs(5));
    //         stream.write_all(http_response.as_bytes());
    //     }

    //     for stream in listener.incoming() {
    //         let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
    //         println!("Connection established!");

    //         // 每个请求都生成一个新线程去处理任务，这种做法开销过大，在请求量大时，很容易造成资源不足
    //         let handle = thread::spawn(move || {
    //             handle_request(stream);
    //         });
    //     }
    // }

    {
        let listener =
            net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");

        // 生成有 5 个线程的线程池
        let thread_pool = ThreadPool::new(5);

        fn handle_request(mut stream: net::TcpStream) {
            let buf_reader = BufReader::new(&stream);
            let http_request: Vec<_> = buf_reader
                .lines()
                .map(|line| line.unwrap())
                .take_while(|line| !line.is_empty())
                .collect();

            let (status_line, html) = if &http_request[0] == "GET / HTTP/1.1" {
                (
                    "HTTP/1.1 200 OK",
                    fs::read_to_string(r"public/http-response-index.html").unwrap(),
                )
            } else {
                (
                    "HTTP/1.1 404 NOT FOUND",
                    fs::read_to_string(r"public/http-response-404.html").unwrap(),
                )
            };

            let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
            let response_body = html;
            let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");

            thread::sleep(Duration::from_secs(5));
            stream.write_all(http_response.as_bytes());
        }

        for stream in listener.incoming() {
            let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
            println!("Connection established!");

            thread_pool.execute(|| handle_request(stream))
        }
    }
}
