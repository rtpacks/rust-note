use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

fn main() {
    /*
     *
     * ## 基于 Send 和 Sync 的线程安全
     * 为什么 Arc 可以在多线程中安全使用，而 Rc、RefCell 和裸指针不可以在多线程间使用呢，这归功于 Send 和 Sync 两个特征。
     *
     * ### Send 和 Sync
     * Send 和 Sync 是 Rust 安全并发的重中之重，但是实际上它们只是标记特征(marker trait，该特征未定义任何行为，只用于标记)：
     * - 实现 Send 特征的类型可以在线程间安全的传递其所有权
     * - 实现 Sync 特征的类型可以在线程间安全的共享(通过引用)
     *
     * 这里有一个潜在的依赖：一个类型要在线程间安全的共享的前提是，指向它的引用必须能在线程间传递。
     * 因为如果引用都不能被传递，我们就无法在多个线程间使用引用去访问同一个数据了。
     * 这意味着：如果 T 为 Sync 则 &T 为 Send，并且这个关系反过来在大部分情况下都是正确的，即如果 &T 为 Send 则 T 为 Sync。
     *
     * 观察 Rc 和 Arc 的源码片段：
     * ```rust
     * // Rc源码片段
     * impl<T: ?Sized> !marker::Send for Rc<T> {}
     * impl<T: ?Sized> !marker::Sync for Rc<T> {}
     *
     * // Arc源码片段
     * unsafe impl<T: ?Sized + Sync + Send> Send for Arc<T> {}
     * unsafe impl<T: ?Sized + Sync + Send> Sync for Arc<T> {}
     * ```
     * `!` 代表移除特征的相应实现，上面代码中 `Rc<T>` 的 Send 和 Sync 特征被特地移除了实现，而 `Arc<T>` 则相反，实现了 Sync + Send。
     *
     * 再看一下 Mutex，Mutex 是没有 Sync 特征限制的：
     * ```rust
     * unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
     * ```
     * 
     */
}
