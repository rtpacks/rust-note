pub struct ThreadPool {}

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

        ThreadPool {}
    }

    pub fn execute<F>(&self, f: F)
    where
        // 泛型参数形式
        // 泛型参数：编译时确定闭包类型，性能更好，无需动态分发。
        // 特征对象：运行时确定闭包类型，灵活但有额外开销。
        F: FnOnce() + Send + 'static,
    {
    }
}
