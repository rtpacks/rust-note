## 实战：多线程 Web 服务器（代码优化和资源清理）

lib.rs：

```rust
// lib.rs
use std::{
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub type Job = Box<dyn FnOnce() + Send + 'static>;
pub struct ThreadPool {
    threads: Vec<JoinHandle<()>>,
    sender: Sender<Job>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// ## Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let mut threads = Vec::with_capacity(size);
        let (mut sender, mut receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        for i in 0..size {
            let _receiver = Arc::clone(&receiver);
            threads.push(thread::spawn(move || loop {
                for job in _receiver.lock().unwrap().recv() {
                    println!("index: {i} got a job; executing.");
                    job();
                }
            }))
        }

        ThreadPool { threads, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        // 泛型参数形式
        // 泛型参数：编译时确定闭包类型，性能更好，无需动态分发。
        // 特征对象：运行时确定闭包类型，灵活但有额外开销。
        F: FnOnce() + Send + 'static,
    {
        // 传递特征对象，因为函要求定长类型，特征属于非定长的类型
        let box_f = Box::new(f);
        self.sender.send(box_f);
    }
}
```

main.rs

```rust
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
```

### Worker 与 Thread

在 ThreadPool::new 函数中直接生成线程并 push 到线程池是比较难拓展的，代码耦合程度较高。
比如未来添加统计信息、日志记录、错误处理等功能，就需要遍历整个线程池的生成逻辑。
这里可以考虑创建一个 Worker 结构体，作为 ThreadPool 和任务线程联系的桥梁，提供更好的抽象和管理线程池的工作线程。来自 GPT 的优化：

1.  更好的职责分离
    将工作线程封装在 Worker 结构体中可以清晰地定义每个组件的职责： - ThreadPool：负责管理线程池的生命周期，包括启动和停止工作线程，分发任务等 - Worker：负责从任务队列中获取任务并执行
    这种职责分离使得代码更清晰，更易于维护和扩展。

2.  便于扩展和修改
    将线程的逻辑封装在 Worker 中，使得将来在需要修改工作线程的行为时，可以集中在 Worker 类型中进行修改，而不需要遍历整个代码库。
    例如，可以在 Worker 中添加统计信息、日志记录、错误处理等功能，而这些修改不会影响 ThreadPool 的代码。

3.  更好的错误处理和资源管理
    Worker 可以更容易地管理线程的生命周期，包括启动、停止和错误处理。例如可以在 Worker 中实现一些高级功能，例如线程重启、任务超时等。

4.  更好的封装和复用
    Worker 类型可以复用在其他类似的场景中，不仅限于当前的线程池实现。例如可以有不同类型的 Worker，它们可以有不同的任务获取和执行策略。

可以看到只要合理的抽象，代码的可维护性就会极大的提升，现在开始构建 Worker。

将 ThreadPool 存储的线程管理改为 workers：

```rust
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
}
```

Worker 结构体需要存储线程：

```rust
struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}
```

在生成线程池时，需要生成线程，但是线程现在转移到 worker 中管理，所以线程的生成也由 worker 提供：

```rust
// Worker::new
fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
    // 注意 Mutex 释放锁的问题
    //
    // Mutex 没有提供显式的 unlock 方法，它依赖于作用域的结束去释放锁。`while let, for in` 他们形成的是作用域快，在当前用例中只有 job 结束之后才会释放锁。
    //
    // 这样导致的即使已经有新任务到达，但是因为 Mutex 锁住了 receiver，导致其他线程无法使用 receiver，无法接收运行任务，
    // 只有等当前线程结束后，离开作用域自动释放 Mutex，其他线程才有机会使用 receiver，才能运行任务。
    // 所以使用 `while let, for in` 这种方式还是类似单线程，同时运行的只有一个线程，因为接收者的锁没有正确的及时释放。

    let thread = thread::spawn(move || loop {
        let job = receiver.lock().unwrap().recv();
        println!("thread {id} got a job; executing.");
        job.unwrap()();
    });
    Worker { id, thread }
}

// ThreadPool::new
pub fn new(size: usize) -> Self {
    assert!(size > 0);

    let mut workers = Vec::with_capacity(size);
    let (mut sender, mut receiver) = mpsc::channel::<Job>();
    let receiver = Arc::new(Mutex::new(receiver));

    for i in 0..size {
        let _receiver = Arc::clone(&receiver);
        workers.push(Worker::new(i, _receiver));
    }

    ThreadPool { workers, sender }
}
```

到此，Worker 的抽象就已经完成了，后续需要对线程做日志或其他拓展都不需要遍历线程池，理清楚 Worker 结构体即可。

### 线程池与 Drop

在并发请求中，如果某一个任务正在执行或者还未执行但已经进入消息队列，此时关闭程序，就会造成请求中失败的明显错误。
例如将线程池的数量设置为 2，然后尝试三个请求并发，接着终止程序，可能就会看到请求其实已经建立连接，但是由于程序终止，导致错误。

为了处理已经建立好连接的请求，需要对线程池的 Drop 进行一定改造，为线程池实现 Drop 特征，
利用 Drop 特征和 join 方法，在停止时要求线程池中的线程等待完成后才允许 drop：

```rust
impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in self.workers {
            worker.thread.join().unwrap();
        }
    }
}
```

但是这里有一个问题，self 的 workers 是一个存储线程的列表，它要拥有线程的所有权，而 join 方法需要拿走线程的所有权。
在 SafeRust 中，根据所有权规则，一个值最多只有一个所有者，所以 workers 与 join 不能同时拥有线程所有权。

```rust
// join 方法拿走线程的所有权
pub fn join(self) -> Result<T> {
    self.0.join()
}
```

所有权冲突有多种解决办法，其中一个常用的办法是利用 Option 包裹结构体，因为 Option 可以表达 Some 和 None 两种状态：

```rust
pub struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

// Worker::new
Worker {
    id,
    thread: Some(thread),
}
```

以上两处简单的修改并不涉及到线程池的代码，这就是抽象 Worker 的优势。

Option 的 take 方法，可以拿走线程的所有权：

```rust
impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
```

但是以上代码还是会有类型错误，因为使用 `for in` 拿走的是 worker 的所有权，所以会报错。

因为要拿走 worker 内线程的所有权，拿走线程所有权后 worker.thread 将会变为 None。根据借用规则，应该使用 workers 的可变引用，然后再取走 worker 存储的线程：

```rust
impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
```

验证 drop 是否生效，可以在 incoming 函数中使用 take，只获取前两个请求，然后自动结束：

```rust
// main.rs
for stream in listener.incoming().take(2) {
    let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
    println!("Connection established!");

    thread_pool.execute(|| handle_request(stream))
}
```

运行后会发现有些线程可以关闭，有些线程无法关闭，甚至有些请求无法返回响应，这是因为 join 在等待线程内部消息通道的关闭。

### 停止消息通道

```rust
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

for stream in listener.incoming().take(2) {
    let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
    println!("Connection established!");

    thread_pool.execute(|| handle_request(stream))
}
println!("The server has stopped running.");
```

分析并解决无法关闭线程与请求无应答：

1.  第一个请求到达，`thread_pool.execute` 分配线程 x 执行，任务执行完成返回响应
2.  第二个请求到达，`thread_pool.execute` 分配线程 y 执行，此时由于 `incoming().take(2)` 只取两个请求，所以主线程往下运行，输出 The server has stopped running.
3.  主线程运行到最后，开始清理作用域内的资源，rust 通过 Drop 特征清理各种类型的资源，线程无法关闭以及请求无法再响应的问题就出现在这。
    rust 通过 Drop 特征清理资源，Drop 特征就是在一个作用域结束时，rust 会自动调用清理的方法。这个特征功能是编译器在编译时期自动插入的，是一个零开销的实现。
    因此，rust 通过 Drop 清理 ThreadPool 时，会调用 ThreadPool 的 drop 方法：

    ```rust
     impl Drop for ThreadPool {
         fn drop(&mut self) {
             for worker in &mut self.workers {
                 if let Some(thread) = worker.thread.take() {
                     println!("Shutting down worker {}", worker.id);
                     thread.join().unwrap();
                 }
             }
         }
     }
    ```

    在 `for in` 迭代中，每一个线程都有 `join` 等待线程完成的时刻，for 循环是顺序执行的，这意味着是一个接一个地检查每个 worker thread，然后等待（join）每个 worker 线程结束。
    又因为 worker thread 是通过消息通道获取可执行的闭包：

    ```rust
    // Worker::new
    let thread = thread::spawn(move || loop {
        let job = receiver.lock().unwrap().recv();
        println!("thread {id} got a job; executing.");
        job.unwrap()();
    });
    ```

    此时，worker 的线程会一直阻塞在获取可执行的闭包 `recv` 这个函数中，当前阻塞的线程就是 worker 线程响应第一个请求的线程。
    最后就会陷入死锁状态，主线程想关闭清理资源，并且是希望等待其他线程运行结束后（join 方法决定）再关闭，但是其他线程一直在等待可执行的闭包，最后导致死锁。
    这个问题很好解决，在第一次使用消息通道时就已经碰见消息通道没有关闭，主线程无法结束的这个问题。只需要释放发送者，接受者就会受到通道关闭的错误信息。
    在哪关闭？当程序主动尝试释放终止时，这个时间点是最合理的，这个时间点就是调用 drop 方法的时候。

    ```rust
    impl Drop for ThreadPool {
        fn drop(&mut self) {
            drop(self.sender);
            for worker in &mut self.workers {
                if let Some(thread) = worker.thread.take() {
                    println!("Shutting down worker {}", worker.id);
                    thread.join().unwrap();
                    println!("Shut down worker {}", worker.id);
                }
            }
        }
    }
    ```

    drop 函数需要拿走 sender 的所有权，但是线程池需要拥有 sender，和 worker 与 thread 的问题类似，使用 Option 解决该问题：

    ```rust
    pub struct ThreadPool {
        workers: Vec<Worker>,
        sender: Option<Sender<Job>>,
    }

    // ThreadPool::execute
    pub fn execute<F>(&self, f: F)
    where
        // 泛型参数形式
        // 泛型参数：编译时确定闭包类型，性能更好，无需动态分发。
        // 特征对象：运行时确定闭包类型，灵活但有额外开销。
        F: FnOnce() + Send + 'static,
    {
        // 传递特征对象，因为函要求定长类型，特征属于非定长的类型
        let box_f = Box::new(f);
        self.sender.as_ref().unwrap().send(box_f);
    }
    ```

    execute 方法中，使用 `as_ref` 获取发送者的不可变引用。改造 drop 逻辑，使用 take 从 ThreadPool 中取出 sender 所有权并释放 sender：

    ```rust
    impl Drop for ThreadPool {
        fn drop(&mut self) {
            drop(self.sender.take());
            for worker in &mut self.workers {
                if let Some(thread) = worker.thread.take() {
                    println!("Shutting down worker {}", worker.id);
                    thread.join().unwrap();
                    println!("Shut down worker {}", worker.id);
                }
            }
        }
    }
    ```

    最后，当释放发送者关闭消息通道时，接收者接受的可能是一个错误，需要处理：

    ```rust
    let thread = thread::spawn(move || loop {
        let message = receiver.lock().unwrap().recv();
        match message {
            Ok(job) => {
                println!("thread {id} got a job; executing.");
                job();
            }
            Err(_) => {
                // 当消息通道关闭时，退出循环获取的逻辑
                println!("thread {id} disconnected; shutting down.");
                break;
            }
        }
    });
    ```

经过以上流程，线程池、程序都正常关闭，再次学到如何定位并解决多线程编程中死锁的问题。

### 更多

其他功能：

- https://course.rs/advance-practice1/graceful-shutdown.html#可以做的更多

重点关注：

- https://course.rs/advance-practice1/graceful-shutdown.html#上一章节的遗留问题
- 如何实现线程池，主线程如何发送任务给工作线程，为什么需要使用消息通道，线程池如何管理工作线程，抽象 Worker 的好处
- Mutex 与 while let, for in 形成的锁问题导致同一时间只有一个线程在执行，关闭消息通道碰见的死锁问题
- 使用 mpsc 多发送者单接收者的好处是不会存在多个接收者竞争一个执行闭包，消息通道中任务就是闭包的代名词。
