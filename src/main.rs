fn main() {
    /*
     *
     * ## async 异步编程：Stream 流处理
     * async/.await 是 Rust 语法的一部分，它在遇到阻塞操作时( 例如 IO )会让出当前线程的控制权而不是阻塞当前线程，这样就允许当前线程继续去执行其它代码，最终实现并发。
     *
     * 有两种方式可以使用 async： `async fn() {}` 用于声明函数，`async { ... }` 用于声明语句块，它们会返回一个实现 Future 特征的值:
     *
     * ```rust
     * async fn foo() -> i32 {
     *     5
     * }
     * let bar = async {
     *     let value = foo().await;
     *     println!("{}", value + 3);
     *     value + 3
     * };
     * futures::executor::block_on(bar);
     * ```
     * 
     * async 是懒惰的，直到被执行器 poll 或者 .await 后才会开始运行，其中后者是最常用的运行 Future 的方法。
     * 当 .await 被调用时，它会尝试运行 Future 直到完成，但是若该 Future 进入阻塞，那就会让出当前线程的控制权。
     * 当 Future 后面准备再一次被运行时(例如从 socket 中读取到了数据)，执行器会得到通知，并再次运行该 Future ，如此循环，直到完成。
     * 
     * 以上过程只是一个简述，详细内容在底层探秘中已经被深入讲解过，因此这里不再赘述。
     *
     * 
     * 
     */

     // 目前处于稳定版本的 async fn 和 async {}
    async fn foo() -> i32 {
        5
    }
    let bar = async {
        let value = foo().await;
        println!("{}", value + 3);
        value + 3
    };
    futures::executor::block_on(bar);
}
