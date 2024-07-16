use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

// pub type Job = Box<dyn FnOnce() + Send + 'static>;
// pub struct ThreadPool {
//     threads: Vec<JoinHandle<()>>,
//     sender: Sender<Job>,
// }

// impl ThreadPool {
//     /// Create a new ThreadPool.
//     ///
//     /// The size is the number of threads in the pool.
//     ///
//     /// ## Panics
//     ///
//     /// The `new` function will panic if the size is zero.
//     pub fn new(size: usize) -> Self {
//         assert!(size > 0);

//         let mut threads = Vec::with_capacity(size);
//         let (mut sender, mut receiver) = mpsc::channel::<Job>();
//         let receiver = Arc::new(Mutex::new(receiver));

//         for i in 0..size {
//             let _receiver = Arc::clone(&receiver);
//             threads.push(thread::spawn(move || loop {
//                 let job = _receiver.lock().unwrap().recv();
//                 println!("index: {i} got a job; executing.");
//                 job.unwrap()();
//             }));

//             // Mutex 没有提供显式的 unlock 方法，它依赖于作用域的结束去释放锁。`while let, for in` 他们形成的是作用域快，在当前用例中只有 job 结束之后才会释放锁。
//             //
//             // 这样导致的即使已经有新任务到达，但是因为 Mutex 锁住了 receiver，导致其他线程无法使用 receiver，无法接收运行任务，
//             // 只有等当前线程结束后，离开作用域自动释放 Mutex，其他线程才有机会使用 receiver，才能运行任务。
//             // 所以使用 `while let, for in` 这种方式还是类似单线程，同时运行的只有一个线程，因为接收者的锁没有正确的及时释放。
//             //
//             // let _receiver = Arc::clone(&receiver);
//             // threads.push(thread::spawn(move || loop {
//             //     for job in _receiver.lock().unwrap().recv() {
//             //         println!("index: {i} got a job; executing.");
//             //         job();
//             //     }
//             // }))
//         }

//         ThreadPool { threads, sender }
//     }

//     pub fn execute<F>(&self, f: F)
//     where
//         // 泛型参数形式
//         // 泛型参数：编译时确定闭包类型，性能更好，无需动态分发。
//         // 特征对象：运行时确定闭包类型，灵活但有额外开销。
//         F: FnOnce() + Send + 'static,
//     {
//         // 传递特征对象，因为函要求定长类型，特征属于非定长的类型
//         let box_f = Box::new(f);
//         self.sender.send(box_f);
//     }
// }

pub type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Self {
        // Mutex 没有提供显式的 unlock 方法，它依赖于作用域的结束去释放锁。`while let, for in` 他们形成的是作用域快，在当前用例中只有 job 结束之后才会释放锁。
        //
        // 这样导致的即使已经有新任务到达，但是因为 Mutex 锁住了 receiver，导致其他线程无法使用 receiver，无法接收运行任务，
        // 只有等当前线程结束后，离开作用域自动释放 Mutex，其他线程才有机会使用 receiver，才能运行任务。
        // 所以使用 `while let, for in` 这种方式还是类似单线程，同时运行的只有一个线程，因为接收者的锁没有正确的及时释放。

        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("thread {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    println!("thread {id} disconnected; shutting down.");
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
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

        let mut workers = Vec::with_capacity(size);
        let (mut sender, mut receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));

        for i in 0..size {
            let _receiver = Arc::clone(&receiver);
            workers.push(Worker::new(i, _receiver));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
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
        self.sender.as_ref().unwrap().send(box_f);
    }
}

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
