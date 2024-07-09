use futures::{future, join, pin_mut, select, try_join, TryFutureExt};

fn main() {
    /*
     *
     * ## async 异步编程：join! 和 select!
     *
     * 在 JavaScript 中，如果希望同时运行多个 async 并在对应时机处理 async 结果，一般会使用 `Promise.all` 或 `Promise.allSettled` 方法。
     * 在 Rust 中，也有同时运行多个 async 的方法，例如 `join!` 和 `select!`。
     *
     * ### join!
     * `join!` 类似 `Promise.allSettled` 方法，它会等待所有的 Future 执行结束（成功、失败），然后再返回结果。
     * 当然 `Promise.allSettled` 也需要经过 `await Promise.allSettled` 才能在同步的逻辑中编写异步的代码：
     * ```rust
     * async fn join1() {
     *     println!("join1");
     * }
     * async fn join2() {
     *     println!("join2");
     * }
     * async fn join3() {
     *     // join 需要在 async 函数中使用
     *     join!(join1(), join2());
     *     println!("join3");
     * }
     * futures::executor::block_on(join3());
     * ```
     *
     * 特别注意，以下写法是串行的：
     * ```rust
     * async fn join4() {
     *     // 注意，这种写法是串行的
     *     (join1().await, join2().await);
     * }
     * ```
     *
     * 如果希望某一个 Future 报错后就立即停止所有 Future 的执行，可以使用 try_join!，功能上类似 `Promise.all` 函数。
     *
     * 注意，因为 try_join! 在某一个 Future 报错后就会停止所有的 Future 并返回 Error，所以 try_join! 并发的 Future 需要返回同一个 Error 类型。
     * 这样无论哪一个 Future 发生错误，错误类型也是匹配的。如果错误类型不同，可以考虑使用来自 futures::future::TryFutureExt 模块的 map_err 和 err_into 方法将错误进行转换。
     *
     * 使用方法与 join! 类似，也需要在 async 函数中使用 try_join!。
     * ```rust
     * async fn try_join1() -> Result<(), String> {
     *     println!("try_join1");
     *     Ok(())
     * }
     * async fn try_join2() -> Result<(), i32> {
     *     println!("try_join2");
     *     Ok(())
     * }
     * async fn try_join3() {
     *     // try_join 需要在 async 函数中使用，并且需要统一错误类型
     *     // 如果错误类型不同，可以考虑使用来自 futures::future::TryFutureExt 模块的 map_err 和 err_info 方法将错误进行转换。
     *     let fut_1 = try_join1();
     *     let fut_2 = try_join2().map_err(|num| num.to_string());
     *     try_join!(fut_1, fut_2);
     *     println!("try_join3");
     * }
     * futures::executor::block_on(try_join3());
     * ```
     *
     * 如果希望以数组的形式传递多个 Future，可以使用 `futures::future::join_all` 方法。
     *
     * ### select!
     * select! 与 `Promise.race` 的功能相似，它总是取第一个完成的（成功或失败） Future：
     *
     * ```rust
     * async fn select1() {
     *     println!("select1");
     * }
     * async fn select2() {
     *     println!("select2");
     * }
     * async fn select3() {
     *     use futures::FutureExt;
     *     let fut1 = select1().fuse();
     *     let fut2 = select2().fuse();
     *
     *     pin_mut!(fut1, fut2);
     *     select! {
     *         () = fut1 => {},
     *         () = fut2 => {}
     *     }
     *     println!("select3");
     * }
     * futures::executor::block_on(select3());
     * ```
     *
     * 在使用 select! 时，还需要为 Future 添加两道保障：`fuse()` 和 `pin_mut!`。
     *
     * #### FusedFuture
     * 原生的 Future trait 只提供了 Poll:Ready 和 Poll::Pending 两种状态，即使 Future 已经处于 Poll::Ready 的状态，外部也是可以再次 poll 这个 Future 的。
     *
     * 这会导致以下问题：
     * - 效率低下：已经完成的 Future 被反复轮询没有意义，浪费资源
     * - 逻辑错误：如果一个已经完成的 Future 被再次轮询并返回 Poll::Ready，可能会导致程序逻辑错误，比如返回 Ready 的逻辑只能执行一次，反复轮询可能使 Ready 逻辑多次执行，导致错误数据的产生
     *
     * 因为原生 Future trait 并不直接提供一种机制来判断 Future 是否已经**终止**（终止代表不允许再次 Poll 该 Future）。
     * 所以需要额外的机制来保证已经完成（成功或失败）的 Future 不再被 poll，futures 库提供了 FusedFuture trait。
     * FusedFuture 特征通过增加 is_terminated 方法，提供了一种明确的方式来检查 Future 是否已经完成。
     *
     * // TODO 完成 select! 为什么需要 FusedFuture 和 Pin
     *
     *
     * 使用 FusedFuture 特征的优势：
     * - 已经完成的 Future 不再被轮询，可以提高异步操作的效率，避免无意义的计算。
     *
     *
     *
     * #### default 和 complete
     * default 分支是 select! 执行时，按照给定的分支顺序
     *
     *
     * ### unknown
     * Promise 有一个非常好用的属性 thenable，它支持在 promise 结束后又返回一个类似 promise 的数据，当前的 promise 后续的 then 方法就会以返回的 promise 状态为准。
     * rust 的 Future 也有类似的逻辑，
     */

    async fn join1() {
        println!("join1");
    }
    async fn join2() {
        println!("join2");
    }
    async fn join3() {
        // join 需要在 async 函数中使用
        join!(join1(), join2());
        println!("join3");
    }
    futures::executor::block_on(join3());

    async fn join4() {
        // 注意，这种写法是串行的
        (join1().await, join2().await);
    }

    async fn try_join1() -> Result<(), String> {
        println!("try_join1");
        Ok(())
    }
    async fn try_join2() -> Result<(), i32> {
        println!("try_join2");
        Ok(())
    }
    async fn try_join3() {
        // try_join 需要在 async 函数中使用，并且需要统一错误类型
        // 如果错误类型不同，可以考虑使用来自 futures::future::TryFutureExt 模块的 map_err 和 err_into 方法将错误进行转换。
        let fut_1 = try_join1();
        let fut_2 = try_join2().map_err(|num| num.to_string());
        try_join!(fut_1, fut_2);
        println!("try_join3");
    }
    futures::executor::block_on(try_join3());

    async fn select1() {
        println!("select1");
    }
    async fn select2() {
        println!("select2");
    }
    async fn select3() {
        use futures::FutureExt;
        let fut1 = select1().fuse();
        let fut2 = select2().fuse();

        pin_mut!(fut1, fut2);
        select! {
            () = fut1 => {},
            () = fut2 => {}
        }
        println!("select3");
    }
    futures::executor::block_on(select3());

    async fn select_loop() {
        let mut a_fut = future::ready(4);
        let mut b_fut = future::ready(6);
        let mut total = 0;

        loop {
            select! {
                a = a_fut => {
                    println!("a_fut, total = {}, total + a = {}", total, total + a);
                    total += a;
                },
                b = b_fut => {
                    println!("b_fut, total = {}, total + b = {}", total, total +b);
                    total += b;
                },
                complete => break,
                default => panic!(), // 该分支永远不会运行，因为 `Future` 会先运行，然后是 `complete`
            };
        }
    }
    futures::executor::block_on(select_loop());
}
