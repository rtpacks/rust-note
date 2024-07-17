use async_std::prelude::*;
use futures::{future, StreamExt};
use std::time::Duration;

fn main() {
    /*
     *
     * ## 实战：async Web 服务器
     * 在实现单线程 Web 服务器和传统的多线程 Web 服务器后，了解并熟悉了相关的概念，现在开始实现更现代的 async/await 服务器。
     *
     * 多线程 Web 服务器的实现，lib.rs：
     * ```rust
     * pub type Job = Box<dyn FnOnce() + Send + 'static>;
     *
     * pub struct Worker {
     *     id: usize,
     *     thread: Option<JoinHandle<()>>,
     * }
     * impl Worker {
     *     fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
     *         // Mutex 没有提供显式的 unlock 方法，它依赖于作用域的结束去释放锁。`while let, for in` 他们形成的是作用域快，在当前用例中只有 job 结束之后才会释放锁。
     *         //
     *         // 这样导致的即使已经有新任务到达，但是因为 Mutex 锁住了 receiver，导致其他线程无法使用 receiver，无法接收运行任务，
     *         // 只有等当前线程结束后，离开作用域自动释放 Mutex，其他线程才有机会使用 receiver，才能运行任务。
     *         // 所以使用 `while let, for in` 这种方式还是类似单线程，同时运行的只有一个线程，因为接收者的锁没有正确的及时释放。
     *
     *         let thread = thread::spawn(move || loop {
     *             let message = receiver.lock().unwrap().recv();
     *             match message {
     *                 Ok(job) => {
     *                     println!("thread {id} got a job; executing.");
     *                     job();
     *                 }
     *                 Err(_) => {
     *                     println!("thread {id} disconnected; shutting down.");
     *                     break;
     *                 }
     *             }
     *         });
     *         Worker {
     *             id,
     *             thread: Some(thread),
     *         }
     *     }
     * }
     *
     * pub struct ThreadPool {
     *     workers: Vec<Worker>,
     *     sender: Option<Sender<Job>>,
     * }
     *
     * impl ThreadPool {
     *     pub fn new(size: usize) -> Self {
     *         assert!(size > 0);
     *
     *         let mut workers = Vec::with_capacity(size);
     *         let (mut sender, mut receiver) = mpsc::channel::<Job>();
     *         let receiver = Arc::new(Mutex::new(receiver));
     *
     *         for i in 0..size {
     *             let _receiver = Arc::clone(&receiver);
     *             workers.push(Worker::new(i, _receiver));
     *         }
     *
     *         ThreadPool {
     *             workers,
     *             sender: Some(sender),
     *         }
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
     *         self.sender.as_ref().unwrap().send(box_f);
     *     }
     * }
     *
     * impl Drop for ThreadPool {
     *     fn drop(&mut self) {
     *         drop(self.sender.take());
     *         for worker in &mut self.workers {
     *             if let Some(thread) = worker.thread.take() {
     *                 println!("Shutting down worker {}", worker.id);
     *                 thread.join().unwrap();
     *                 println!("Shut down worker {}", worker.id);
     *             }
     *         }
     *     }
     * }
     * ```
     * 多线程 Web 服务器的实现，main.rs：
     * ```rust
     * let listener =
     *     net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");
     *
     * // 生成有 5 个线程的线程池
     * let thread_pool = ThreadPool::new(2);
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
     * for stream in listener.incoming().take(2) {
     *     let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
     *     println!("Connection established!");
     *
     *     thread_pool.execute(|| handle_request(stream))
     * }
     * println!("The server has stopped running.");
     * ```
     *
     * 对于高并发的网络/IO，线程还是太重了，使用 async 实现 Web 服务器才是最适合的。
     *
     * ### async-std
     * 回顾之前所学的 async/await，async/await 作为 Future 的语法糖，最终会被编译器编译为 Future 状态机。
     * rust 为了减少打包体积的大小，只提供了 async/await 相关的标准，并没有提供默认的运行时。
     * 之前的代码尝试实现了简单的执行器来进行 .await 或 poll，但是在实际项目中，这还远远不够，需要选择一个比较完备的第三方 async 运行时来实现相关的功能。
     *
     * > 回顾重点：
     * > - await 不会阻塞当前线程，而是让出当前线程的执行权，在 async 函数结束前会一直异步等待 await 结束
     * > - `.await` 是非阻塞线程的，当 `future.await` 时，它暂停的是当前 future 所在的上层 async 函数，并让出当前线程的执行权，当前线程可以执行与 `future.await` 上层同级的其它异步 Future
     * > - Future 是一个能**产出值的异步计算**(值可能为空，例如 `()`)。它是异步函数的返回值和被执行的关键，异步函数则是异步编程的核心，所以 Future 特征是 Rust 异步编程的核心。
     * > - Future 是惰性的，需要在 poll 函数调用后才会真正执行，同时 poll 只会获取异步任务执行的状态，对异步任务执行流程和结果没有任何影响。
     * > - **Future 一定要有一个能表达任务状态的数据**，这样执行器在 poll Future 时才知道对 Future 的操作是等待 `Poll::Pedning` 还是结束 `Poll::Ready`。
     * > - 避免 Future 完成后被再次执行，原生的 Future trait 只提供了 Poll:Ready 和 Poll::Pending 两种状态，即使 Future 已经处于 Poll::Ready 的状态，外部也是可以再次 poll 这个 Future 的。
     * > - FusedFuture 特征通过增加 is_terminated 方法，提供了一种明确的方式来检查 Future 是否已经完成。
     * > - select! 和 join!
     * > - async 语句块和 async fn 最大的区别就是 async 语句块无法显式的声明返回值，当配合 `?`（错误传播）一起使用时就会有类型错误。
     * > - 由于 rust async/await 的调度，Future 可能运行在不同的线程上，由于多线程需要保证数据的所有权和引用的正确性。所以当处于多线程时 Future 需要关注 **.await 运行过程中**，传递给 Future 作用域的变量类型是否是 Send。
     *
     * 现在先选择 async-std ，该包的最大优点就是跟标准库的 API 类似，相对来说更简单易用。
     *
     * 首先先将 handle_request 改成 async 函数：
     * ```rust
     * async fn handle_request(mut stream: net::TcpStream) {
     *     let buf_reader = BufReader::new(&stream);
     *     let http_request: Vec<_> = buf_reader
     *         .lines()
     *         .map(|line| line.unwrap())
     *         .take_while(|line| !line.is_empty())
     *         .collect();
     *
     *     let (status_line, path) = if &http_request[0] == "GET / HTTP/1.1" {
     *         ("HTTP/1.1 200 OK", r"public/http-response-index.html")
     *     } else {
     *         ("HTTP/1.1 404 NOT FOUND", r"public/http-response-404.html")
     *     };
     *
     *     let html = fs::read_to_string(path).unwrap();
     *     let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
     *     let response_body = html;
     *     let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");
     *
     *     thread::sleep(Duration::from_secs(5));
     *     stream.write_all(http_response.as_bytes());
     * }
     * ```
     *
     * 然后可以选择将 main 函数改造成 async，或者选择在 main 函数中使用 block_on：
     * ```rust
     * // 将 main 函数改造成 async
     * #[async_std::main]
     * async fn main() {
     *     let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
     *     for stream in listener.incoming() {
     *         let stream = stream.unwrap();
     *         // 警告，这里无法并发
     *         handle_connection(stream).await;
     *     }
     * }
     *
     * // 或者在 main 函数中使用 block_on，将 async 转移到 start 函数
     * async fn start() {
     *     let listener =
     *         net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");
     *
     *     for stream in listener.incoming() {
     *         let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
     *         println!("Connection established!");
     *         // 警告，这里无法并发
     *         handle_request(stream).await
     *     }
     * }
     *
     * async_std::task::block_on(start());
     * ```
     *
     * 以上代码改造后，测试运行会发现效果与单线程 Web 服务器一样，需要等待前一个请求完成后，后一个请求才会开始运行。
     * 原因在于：handle_reuqest 函数中使用线程休眠（线程阻塞）模拟了耗时任务，线程休眠后异步运行时无法调度任务。
     *
     * 为什么后续的请求不会安排到其他线程执行？分析以上代码：
     * 当前的 handle_request 与 async_std 的执行器属于同一个线程，handle_request 阻塞当前线程后，当前线程无法处理任务，下一个请求只能等待。
     *
     * > 理解：
     * >
     * > async 异步运行时是基于非阻塞任务的假设设计的，在大部分时候都是基于**单线程异步并发**的认识去使用 async/await，这与 JavaScript 的单线程事件循环（event loop）是类似的。
     * > async 异步运行时假设任务会在遇到 I/O 操作或其他需要等待的操作时会让出控制权（例如通过 await，由 Waker 触发执行器 poll 获取 Future 状态后决定是否需要继续运行），如果任务执行了阻塞操作，那么在线程解除阻塞前任务不会让出线程的控制权，调度器无法将正在执行的任务迁移到其他线程。
     * >
     * > 异步任务调度器依赖于任务的非阻塞特性来高效地调度任务。除了单线程异步并发外，异步运行时（如 async-std 和 tokio）通常还会使用一个线程池来**并行**执行任务。
     * > 线程池中的每个线程都会执行一个事件循环（event loop）来处理任务队列中的任务，所以如果一个任务阻塞了当前线程，那么这个线程上的其他任务都会受到影响。
     *
     * 将线程休眠改成异步的休眠，避免影响其他的任务执行：
     * ```diff
     * - thread::sleep(Duration::from_secs(5));
     * + async_std::task::sleep(Duration::from_secs(5)).await;
     * ```
     *
     * 运行后，发现还是单线程效果，这是因为标准库中的 incoming() 是一个阻塞的迭代器，需要将其改成不阻塞的 stream，参考 【unit 74-async 异步编程：Stream 流处理】中 for_each_concurrent。
     * 标准库中的 net::TcpListener 的生成默认是阻塞的，应该改成异步的 async_std::net::TcpListener
     * ```rust
     * async fn start() {
     *     let listener = async_std::net::TcpListener::bind("127.0.0.1:7878")
     *         .await
     *         .expect("TcpListener started with an error");
     *
     *     listener
     *         .incoming()
     *         .enumerate()
     *         .for_each_concurrent(None, |(i, stream)| async move {
     *             let stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
     *             println!("{i} Connection established!");
     *             handle_request(stream).await
     *         })
     *         .await;
     * }
     * ```
     *
     * > rust 的迭代器并不是一个迭代器处理完集合中的所有数据后再传递给下一个迭代器处理，它的设计更像是中间件，即不同方法的组合。这与 JavaScript 不相同。
     * >
     * > 对于一条数据，当前迭代器的逻辑处理完成后，就会给到下一个迭代器处理。并不是等收集所有数据，在一个迭代器中处理完成这些数据后再给到下一个迭代器。
     *
     * 为了异步应该要将所有的同步耗时操作都改成异步的，比如当前服务器中 html 文件的同步（阻塞）读写，响应同步（阻塞）返回：
     * ```rust
     * async fn handle_request(mut stream: async_std::net::TcpStream) {
     *     let buf_reader = async_std::io::BufReader::new(&stream);
     *
     *     let http_request: Vec<_> = buf_reader
     *         .lines()
     *         .map(|line| line.unwrap())
     *         .take_while(|line| future::ready(!line.is_empty()))
     *         .collect()
     *         .await;
     *
     *     let (status_line, path) = if &http_request[0] == "GET / HTTP/1.1" {
     *         ("HTTP/1.1 200 OK", r"public/http-response-index.html")
     *     } else {
     *         ("HTTP/1.1 404 NOT FOUND", r"public/http-response-404.html")
     *     };
     *
     *     let html = async_std::fs::read_to_string(path).await.unwrap();
     *     let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
     *     let response_body = html;
     *     let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");
     *
     *     async_std::task::sleep(Duration::from_secs(5)).await;
     *     // use async_std::io::WriteExt;
     *     stream.write_all(http_response.as_bytes()).await;
     * }
     * ```
     *
     * ### 并行
     * async 异步运行时是基于非阻塞任务的假设设计的，在大部分时候都是基于**单线程异步并发**的认识去使用 async/await，这与 JavaScript 的单线程事件循环（event loop）是类似的。
     * 异步任务调度器依赖于任务的非阻塞特性来高效地调度任务。除了单线程异步并发外，异步运行时（如 async-std 和 tokio）通常还会使用一个线程池来**并行**执行任务。
     *
     * 可以手动指定任务到另外的线程中执行，async_std::task::spawn 是一种高效的异步执行机制，它利用协程和线程池来避免过度创建线程，从而降低资源消耗并提高性能：
     * ```rust
     * async fn start() {
     *     let listener = async_std::net::TcpListener::bind("127.0.0.1:7878")
     *         .await
     *         .expect("TcpListener started with an error");
     *
     *     listener
     *         .incoming()
     *         .enumerate()
     *         .for_each_concurrent(None, |(i, stream)| async move {
     *             let stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
     *             println!("{i} Connection established!");
     *             // handle_request(stream).await
     *             async_std::task::spawn(handle_request(stream));
     *         })
     *         .await;
     * }
     * ```
     *
     * ### 测试
     * 可以选择 public/async-bench.js 测试，也可以参考 course.rs 中的测试用例：https://course.rs/advance/async/web-server.html#测试-handle_connection-函数
     * 
     * ### 总结
     * rust 的 async 与 JavaScript 类似，在大部分时候都是以单线程异步并发的形式出现。
     * 但 rust async 又比 JavaScript async 强大，并且考虑的东西更多。
     * 
     * 选择 async 意味着相关的同步耗时任务需要考虑改造成 Future，并且尽量不出现线程阻塞的情况，否则会出现线程不能执行其他任务的问题。
     */

    async fn handle_request(mut stream: async_std::net::TcpStream) {
        let buf_reader = async_std::io::BufReader::new(&stream);

        let http_request: Vec<_> = buf_reader
            .lines()
            .map(|line| line.unwrap())
            .take_while(|line| future::ready(!line.is_empty()))
            .collect()
            .await;

        let (status_line, path) = if &http_request[0] == "GET / HTTP/1.1" {
            ("HTTP/1.1 200 OK", r"public/http-response-index.html")
        } else {
            ("HTTP/1.1 404 NOT FOUND", r"public/http-response-404.html")
        };

        let html = async_std::fs::read_to_string(path).await.unwrap();
        let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
        let response_body = html;
        let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");

        async_std::task::sleep(Duration::from_secs(5)).await;
        // use async_std::io::WriteExt;
        stream.write_all(http_response.as_bytes()).await;
    }

    async fn start() {
        let listener = async_std::net::TcpListener::bind("127.0.0.1:7878")
            .await
            .expect("TcpListener started with an error");

        listener
            .incoming()
            .enumerate()
            .for_each_concurrent(None, |(i, stream)| async move {
                let stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
                println!("{i} Connection established!");
                // handle_request(stream).await;
                async_std::task::spawn(handle_request(stream));
            })
            .await;
    }

    async_std::task::block_on(start());

    println!("The server has stopped running.");
}
