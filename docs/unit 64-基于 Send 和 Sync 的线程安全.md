## 基于 Send 和 Sync 的线程安全

为什么 Arc 可以在多线程中安全使用，而 Rc、RefCell 和裸指针不可以在多线程间使用呢，这归功于 Send 和 Sync 两个特征。

### Send 和 Sync

Send 和 Sync 是 Rust 安全并发的重中之重，但是实际上它们只是标记特征(marker trait，该特征未定义任何行为，只用于标记)：

- 实现 Send 特征的类型可以在线程间安全的传递其所有权，即**数据能够在不同线程之间转移**
- 实现 Sync 特征的类型可以在线程间安全的共享(通过引用)，即**数据能够在不同线程之间共享**

这里有一个潜在的依赖：一个类型要在线程间安全的共享的前提是，指向它的引用必须能在线程间传递。
因为如果引用都不能被传递，我们就无法在多个线程间使用引用去访问同一个数据了。
这意味着：如果 T 为 Sync 则 &T 为 Send，并且这个关系反过来在大部分情况下都是正确的，即如果 &T 为 Send 则 T 为 Sync。

观察 Rc 和 Arc 的源码片段：

```rust
// Rc源码片段
impl<T: ?Sized> !marker::Send for Rc<T> {}
impl<T: ?Sized> !marker::Sync for Rc<T> {}

// Arc源码片段
unsafe impl<T: ?Sized + Sync + Send> Send for Arc<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Arc<T> {}
```

`!` 代表移除特征的相应实现，上面代码中 `Rc<T>` 的 Send 和 Sync 特征被特地移除了实现，而 `Arc<T>` 则相反，实现了 Sync + Send。

再看一下 Mutex，Mutex 是没有 Sync 特征限制的，这意味着 Mutex 不能直接用于线程共享：

```rust
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
```

### 实现 Send 和 Sync 的类型

在 Rust 中，几乎所有类型都默认实现了 Send 和 Sync，而且由于这两个特征都是可自动派生的特征(通过 derive 派生)。

一个复合类型(例如结构体), 只要它内部的所有成员都实现了 Send 或者 Sync，那么它就自动实现了 Send 或 Sync。

正是因为以上规则，Rust 中绝大多数类型都实现了 Send 和 Sync，常见的未实现有以下几个:

- 裸指针两者都没实现，它本身没有任何安全保证
- UnsafeCell 没有实现 Sync，因此 Cell 和 RefCell 也不是
- Rc 两者都没实现(因为内部的引用计数器不是线程安全的)

当然，如果是自定义的复合类型，那没实现这两个特征就比较常见：只要复合类型中有一个成员不是 Send 或 Sync，那么该复合类型也就不是 Send 或 Sync。

手动实现 Send 和 Sync 是不安全的，通常并不需要手动实现 Send 和 Sync trait，实现者需要使用 unsafe 小心维护并发安全保证。

### 为裸指针实现 Send 和 Sync

无法直接为裸指针实现 Send 和 Sync 特征，因此需要借助 newtype 为裸指针实现这两个特征。
但有一点需要注意：Send 和 Sync 是 unsafe 特征，实现时需要用 unsafe 代码块包裹。

```rust
#[derive(Debug)]
struct MyBox(*mut u8);
// 为裸指针实现 Send 特征，支持数据在不同线程中转移
unsafe impl Send for MyBox {}

// Send 特征支持数据在不同线程中转移
let mut b = MyBox(5 as *mut u8);
let h = thread::spawn(move || {
    println!("{:?}", b);
});
h.join().unwrap();
```

Sync 特征支持数据在不同的线程中共享，但在多线程中共享数据涉及到 rust 的单一所有权问题，此时需要搭配 Arc 才能在多线程中共享：

```rust
#[derive(Debug)]
struct MyBox(*mut u8);
// 为裸指针实现 Send 特征，支持数据在不同线程中转移
unsafe impl Send for MyBox {}
// 为裸指针实现 Sync 特征，支持数据在不同线程中共享
unsafe impl Sync for MyBox {}

// Send 特征支持数据在不同线程中转移
let mut b = MyBox(5 as *mut u8);
let h = thread::spawn(move || {
    println!("{:?}", b);
});
h.join().unwrap();

// Sync 特征支持数据在不同的线程中共享，此时涉及到 rust 的所有权问题，需要搭配 Arc 才能在多线程中共享
let mut b = MyBox(5 as *mut u8);
let arc_b = Arc::new(b);
let _arc_b = Arc::clone(&arc_b);
let h = thread::spawn(move || {
    println!("{:?}", _arc_b);
});
h.join().unwrap();
```

### 总结

- 实现 Send 特征的类型可以在线程间安全的传递其所有权，即数据支持在线程中转移
- 实现 Sync 特征的类型可以在线程间安全的共享(通过引用)，即数据支持在线程中共享
- 绝大部分类型都实现了 Send 和 Sync 特征，常见的未实现的有：裸指针、Cell、RefCell、Rc 等
- 可以为自定义类型实现 Send 和 Sync，但是需要 unsafe 代码块
- 可以为部分 Rust 中的类型实现 Send、Sync，但是需要使用 newtype，例如裸指针

### Code

```rust
fn main() {
    #[derive(Debug)]
    struct MyBox(*mut u8);
    // 为裸指针实现 Send 特征，支持数据在不同线程中转移
    unsafe impl Send for MyBox {}
    // 为裸指针实现 Sync 特征，支持数据在不同线程中共享
    unsafe impl Sync for MyBox {}

    // Send 特征支持数据在不同线程中转移
    let mut b = MyBox(5 as *mut u8);
    let h = thread::spawn(move || {
        println!("{:?}", b);
    });
    h.join().unwrap();

    // Sync 特征支持数据在不同的线程中共享，此时涉及到 rust 的所有权问题，需要搭配 Arc 才能在多线程中共享
    let mut b = MyBox(5 as *mut u8);
    let arc_b = Arc::new(b);
    let _arc_b = Arc::clone(&arc_b);
    let h = thread::spawn(move || {
        println!("{:?}", _arc_b);
    });
    h.join().unwrap();
}
```
