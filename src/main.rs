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
     * ### async 的生命周期
     * async fn 函数返回的 Future 的生命周期会受到 Future 所使用变量的生命周期的限制，如使用引用类型的变量。
     *
     * 分析 async fn 的生命周期，与等价的函数进行对比：
     * ```rust
     * async fn foo(x: &u8) -> u8 { *x }
     *
     * // 上面的函数跟下面的函数是等价的
     * fn foo_expanded<'a>(x: &'a u8) -> impl Future<Output = u8> + 'a {
     *     async move { *x }
     * }
     * ```
     * 上面的生命周期表明：如果要 Future 运行正常( .await )，就必须保证 x 在 Future 运行时保持正常，即说 x 必须比 Future 活得更久。
     * 所以 Future 的生命周期受限于所使用变量的生命周期，必须保证所使用变量的声明周期包含 Future 的生命周期，Future 才能正常运行。
     *
     * 在一般情况下，在函数调用后就立即 .await 不会存在任何问题，例如foo(&x).await。
     * 但是如果 Future 被先存起来或发送到另一个任务或者线程，就可能存在问题了。
     *
     * 错误示例，以下代码会报错，因为 x 的生命周期只到 bad 函数的结尾。 但是 Future 显然会活得更久：
     * ```rust
     * async fn borrow_x(x: &u8) -> u8 { *x }
     *
     * fn bad() -> impl Future<Output = u8> {
     *     let x = 5;
     *     borrow_x(&x) // ERROR: `x` does not live long enough
     * }
     * ```
     *
     * 这种场景最常用的解决办法是返回一个新的 Future，将新的 Future 的生命周期变为 `'static`。将原有 Future 的调用与原有 Future 使用的变量用新 Future 包裹起来：
     *
     * ```rust
     * // fn bad() -> impl futures::Future<Output = u8> {
     * //     let x = 5;
     * //     borrow_x(&x) // ERROR: `x` does not live long enough
     * // }
     * fn good() -> impl futures::Future<Output = u8> {
     *     // 错误
     *     //     let x = 5;
     *     //     borrow_x(&x) // ERROR: `x` does not live long enough
     *
     *     // 正确
     *     async {
     *         let x = &5;
     *         borrow_x(x).await
     *     }
     * }
     * // async 写法
     * async fn good_async() -> u8 {
     *     let x = &5;
     *     borrow_x(x).await
     * }
     * ```
     * 
     * 以上就是使用 async 和 `fn() -> impl Future<Output = T>` 两种语法的区别，新的 Future 正好与标注返回的 Future 类型保持了一致。。
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

    async fn borrow_x(x: &u8) -> u8 {
        *x
    }

    // fn bad() -> impl futures::Future<Output = u8> {
    //     let x = 5;
    //     borrow_x(&x) // ERROR: `x` does not live long enough
    // }
    fn good() -> impl futures::Future<Output = u8> {
        // 错误
        //     let x = 5;
        //     borrow_x(&x) // ERROR: `x` does not live long enough

        // 正确
        async {
            let x = &5;
            borrow_x(x).await
        }
    }
    // async 写法
    async fn good_async() -> u8 {
        let x = &5;
        borrow_x(x).await
    }
}
