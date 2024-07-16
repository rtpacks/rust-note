use futures::stream;
use ilearn::threadpool::ThreadPool;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net, thread,
    time::Duration,
};

fn main() {
    /*
     *
     * ## 实战：多线程 Web 服务器（代码优化和资源清理）
     * lib.rs：
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
     * ### Worker 与 Thread
     * 在 ThreadPool::new 函数中直接生成线程并 push 到线程池是比较难拓展的，代码耦合程度较高。
     * 比如未来添加统计信息、日志记录、错误处理等功能，就需要遍历整个线程池的生成逻辑。
     * 这里可以考虑创建一个 Worker 结构体，作为 ThreadPool 和任务线程联系的桥梁，提供更好的抽象和管理线程池的工作线程。来自 GPT 的优化：
     *
     * 1. 更好的职责分离
     * 将工作线程封装在 Worker 结构体中可以清晰地定义每个组件的职责：
     *     - ThreadPool：负责管理线程池的生命周期，包括启动和停止工作线程，分发任务等
     *     - Worker：负责从任务队列中获取任务并执行
     * 这种职责分离使得代码更清晰，更易于维护和扩展。
     *
     * 2. 便于扩展和修改
     * 将线程的逻辑封装在 Worker 中，使得将来在需要修改工作线程的行为时，可以集中在 Worker 类型中进行修改，而不需要遍历整个代码库。
     * 例如，可以在 Worker 中添加统计信息、日志记录、错误处理等功能，而这些修改不会影响 ThreadPool 的代码。
     *
     * 3. 更好的错误处理和资源管理
     * Worker 可以更容易地管理线程的生命周期，包括启动、停止和错误处理。例如可以在 Worker 中实现一些高级功能，例如线程重启、任务超时等。
     *
     * 4. 更好的封装和复用
     * Worker 类型可以复用在其他类似的场景中，不仅限于当前的线程池实现。例如可以有不同类型的 Worker，它们可以有不同的任务获取和执行策略。
     *
     * 可以看到只要合理的抽象，代码的可维护性就会极大的提升，现在开始构建 Worker。
     *
     * 将 ThreadPool 存储的线程管理改为 workers：
     * ```rust
     * pub struct ThreadPool {
     *     workers: Vec<Worker>,
     *     sender: Sender<Job>,
     * }
     * ```
     *
     * Worker 结构体需要存储线程：
     * ```rust
     * struct Worker {
     *     id: usize,
     *     thread: JoinHandle<()>,
     * }
     * ```
     *
     * 在生成线程池时，需要生成线程，但是线程现在转移到 worker 中管理，所以线程的生成也由 worker 提供：
     * ```rust
     * // Worker::new
     * fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
     *     // 注意 Mutex 释放锁的问题
     *     //
     *     // Mutex 没有提供显式的 unlock 方法，它依赖于作用域的结束去释放锁。`while let, for in` 他们形成的是作用域快，在当前用例中只有 job 结束之后才会释放锁。
     *     //
     *     // 这样导致的即使已经有新任务到达，但是因为 Mutex 锁住了 receiver，导致其他线程无法使用 receiver，无法接收运行任务，
     *     // 只有等当前线程结束后，离开作用域自动释放 Mutex，其他线程才有机会使用 receiver，才能运行任务。
     *     // 所以使用 `while let, for in` 这种方式还是类似单线程，同时运行的只有一个线程，因为接收者的锁没有正确的及时释放。
     *
     *     let thread = thread::spawn(move || loop {
     *         let job = receiver.lock().unwrap().recv();
     *         println!("thread {id} got a job; executing.");
     *         job.unwrap()();
     *     });
     *     Worker { id, thread }
     * }
     *
     * // ThreadPool::new
     * pub fn new(size: usize) -> Self {
     *     assert!(size > 0);
     *
     *     let mut workers = Vec::with_capacity(size);
     *     let (mut sender, mut receiver) = mpsc::channel::<Job>();
     *     let receiver = Arc::new(Mutex::new(receiver));
     *
     *     for i in 0..size {
     *         let _receiver = Arc::clone(&receiver);
     *         workers.push(Worker::new(i, _receiver));
     *     }
     *
     *     ThreadPool { workers, sender }
     * }
     * ```
     *
     * 到此，Worker 的抽象就已经完成了，后续需要对线程做日志或其他拓展都不需要遍历线程池，理清楚 Worker 结构体即可。
     * 
     * 
     *
     *
     * // TODO 继续生成 Worker
     *
     *
     *
     *
     */

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
