use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

fn main() {
    /*
     *
     * ## 全局变量
     * 在一些场景，我们可能需要全局变量来简化状态共享的代码，包括全局 ID，全局数据存储等等。
     *
     * 全局变量是一种特殊的变量，在 rust 中相对复杂，但有一点可以肯定，全局变量的生命周期肯定是'static，但是不代表它需要用static来声明。
     *
     * 具体来说，全局变量分为编译期初始化和运行期初始化两种。
     *
     * **常量与普通变量的区别**
     * - 关键字是const而不是let
     * - 定义常量必须指明类型（如 i32），不能省略
     * - 定义常量时变量的命名规则一般是全部大写
     * - 对于变量出现重复的定义(绑定)会发生变量遮盖，后面定义的变量会遮住前面定义的变量，常量则不允许出现重复的定义
     * - 常量可以在任意作用域进行定义，其生命周期贯穿整个程序的生命周期。编译时编译器会尽可能将其内联到代码中，所以在不同地方对同一常量的引用并不能保证引用到相同的内存地址
     * - 编译期初始化常量的赋值只能是常量表达式/数学表达式，也就是说必须是在编译期就能计算出的值，如果需要在运行时才能得出结果的值比如函数，则不能赋值给常量表达式。即常量的赋值不能在程序运行时通过配置实现。
     *
     * ### 编译期初始化
     * 大多数使用的全局变量都只需要在编译期初始化，例如静态配置、计数器、状态值等等。
     *
     * #### 静态常量
     *
     * 全局常量可以在程序任何一部分使用，如果它是定义在某个模块中，则需要引入对应的模块才能使用。全局常量很适合用作静态配置：
     * ```rust
     * const MAX_ID: usize =  usize::MAX / 2;
     * println!("最大的用户 ID = {}", MAX_ID);
     * ```
     *
     * 常量可以在任意作用域中定义，编译时编译器会尽可能将其内联到代码中，所以在不同地方对同一常量的引用并不能保证引用到相同的内存地址。
     *
     *
     * #### 静态变量
     *
     * 静态变量允许声明一个全局的变量，常用于全局数据统计，例如统计总请求数：
     * ```rust
     * // 静态变量
     * static mut REQUEST_COUNT: usize = 0;
     * unsafe {
     *     // 操作 static 类型的变量需要 unsafe 模块
     *     // 因为这种使用方式往往并不安全，当在多线程中同时去修改时，会不可避免的遇到脏数据
     *     REQUEST_COUNT = 2;
     * }
     * unsafe {
     *     println!("REQUEST_COUNT = {}", REQUEST_COUNT);
     * }
     * ```
     * 操作 static 类型的变量需要 unsafe 作用域，因为这种使用方式往往并不安全，当在多线程中同时去修改时，会不可避免的遇到脏数据。
     *
     * 和常量相同，定义静态变量的时候必须赋值为在编译期就可以计算出的值(常量表达式/数学表达式)，不能是运行时才能计算出的值(如函数)，即不能通过程序运行时再配置定义静态变量。
     *
     *
     * #### 静态变量和常量的区别
     *
     * - 静态变量不会被内联，在整个程序中，静态变量只有一个实例，所有的引用都会指向同一个地址
     * - 为了能在多线程中正常使用，存储在静态变量中的值必须要实现 Sync trait
     *
     * #### 原子类型
     * 原子类型是多线程共享数据的线程安全的最好方式之一：
     * ```rust
     * // 原子类型是共享状态最好的一种方式
     * static REQUEST_RECV: AtomicUsize = AtomicUsize::new(0);
     * let mut handles: Vec<JoinHandle<()>> = Vec::new();
     * for i in 0..100 {
     *     handles.push(thread::spawn(move || {
     *         REQUEST_RECV.fetch_add(i, Ordering::SeqCst);
     *     }));
     * }
     * for h in handles {
     *     h.join().unwrap();
     * }
     * println!("REQUEST_RECV = {}", REQUEST_RECV.load(Ordering::SeqCst));
     * ```
     *
     */

    //  静态常量
    const MAX_ID: usize = usize::MAX / 2;
    println!("最大的用户 ID = {}", MAX_ID);

    // 静态变量
    static mut REQUEST_COUNT: usize = 0;
    unsafe {
        // 操作 static 类型的变量需要 unsafe 作用域
        // 因为这种使用方式往往并不安全，当在多线程中同时去修改时，会不可避免的遇到脏数据
        REQUEST_COUNT = 2;
    }
    unsafe {
        println!("REQUEST_COUNT = {}", REQUEST_COUNT);
    }

    // 原子类型是共享状态最好的一种方式
    static REQUEST_RECV: AtomicUsize = AtomicUsize::new(0);
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    for i in 0..100 {
        handles.push(thread::spawn(move || {
            REQUEST_RECV.fetch_add(i, Ordering::SeqCst);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!("REQUEST_RECV = {}", REQUEST_RECV.load(Ordering::SeqCst));
}
