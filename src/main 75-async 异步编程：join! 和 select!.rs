use futures::{
    future::{self, Fuse, FusedFuture},
    join, pin_mut, select,
    stream::{FusedStream, Stream},
    try_join, TryFutureExt,
};

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
     *         () = fut1 => {
     *             if fut1.is_terminated() {
     *                 println!("fut1.is_terminated")
     *             }
     *         },
     *         () = fut2 => {
     *             if fut1.is_terminated() {
     *                 println!("fut1.is_terminated")
     *             }
     *         }
     *     }
     *     println!("select3");
     * }
     * futures::executor::block_on(select3());
     * ```
     *
     * select! 的完整实现比较复杂，以下 GPT 给出一个简化的 select! 实现以帮助理解：
     * ```rust
     * macro_rules! select {
     *     (
     *         $( $name:ident = $future:expr => $code:block ),*,
     *         complete => $complete:block,
     *         default => $default:block
     *     ) => {{
     *         use std::pin::Pin;
     *         use std::task::{Context, Poll, Waker};
     *         use futures::future::FutureExt;
     *
     *         let mut futures = ($(Pin::new(&$future.fuse())),*);
     *         let waker = futures::task::noop_waker();
     *         let mut cx = Context::from_waker(&waker);
     *
     *         loop {
     *             let mut all_ready = true;
     *             $(
     *                 if futures.$name.is_terminated() {
     *                     // Future already terminated, skip it
     *                     continue;
     *                 }
     *                 if let Poll::Pending = futures.$name.poll(&mut cx) {
     *                     all_ready = false;
     *                 } else {
     *                     // Future is ready, execute the associated code block
     *                     { $code }
     *                     break;
     *                 }
     *             )*
     *
     *             if all_ready {
     *                 // All futures are ready, execute the complete block
     *                 { $complete }
     *                 break;
     *             } else {
     *                 // None of the futures are ready, execute the default block
     *                 { $default }
     *             }
     *         }
     *     }};
     * }
     * ```
     *
     * select! 通过一个循环来轮询每个 Future，如果一个 Future 返回 Poll::Ready，执行相应的代码块并退出循环。
     * 如果所有的 Future 都返回 Poll::Pending，执行默认的代码块即 default 分支。
     * 如果所有的 Future 都返回 Poll::Ready，则会执行 complete 分支。
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
     * 所以需要额外的机制来保证已经完成（成功或失败）的 Future 不再被 poll，futures 库提供了 FusedFuture trait 来完成这个功能。
     * FusedFuture 特征通过增加 is_terminated 方法，提供了一种明确的方式来检查 Future 是否已经完成。
     *
     * 使用 FusedFuture 特征的优势：
     * - 提高效率：已经完成的 Future 不再被轮询，可以提高异步操作的效率，避免无意义的计算
     * - 防止重复轮询：当一个 Future 完成后，当FusedFuture 来检查 Future 是否已经完成并防止它被再次轮询
     *
     * #### pin_mut!
     *
     * select! 需要同时轮询多个 Future，这些 Future 可能在不同的时间点完成。
     * 为了避免避免潜在的内存安全问题（如【unit 73-async 异步编程：Pin 和 Unpin】介绍的 Future 自引用问题），需要确保这些 Future 在整个异步操作过程中保持固定位置，此时用到 pin_mut! 辅助。
     *
     * ```rust
     * use futures::FutureExt;
     * let fut1 = select1().fuse();
     * let fut2 = select2().fuse();
     *
     * pin_mut!(fut1, fut2);
     * select! {
     *     () = fut1 => {
     *         if fut1.is_terminated() {
     *             println!("fut1.is_terminated")
     *         }
     *     },
     *     () = fut2 => {
     *         if fut1.is_terminated() {
     *             println!("fut1.is_terminated")
     *         }
     *     }
     * }
     * ```
     *
     * select! 使用 Pin 就是为了使 Future 在轮询过程中保持固定的位置，确保 Future 不会在内存中被移动，保证内存安全。
     * 准确的说来自变量名/路径的future要求其实现Unpin+FusedFuture，对于来自表达式的future可以放宽Unpin的限制。
     *
     * #### default 和 complete
     * 除了给定 Future 分支外，select! 还支持两个特殊的分支：default 和 complete：
     * - default 分支，若没有任何 Future 或 Stream 处于 Ready 状态， 则该分支会被立即执行
     * - complete 分支当所有的 Future 和 Stream 完成后才会被执行，它往往配合 loop 使用，loop 用于循环完成所有的 Future
     *
     * 其中 default 分支比较特殊，一个 select! 宏，default 分支可能重复执行，也可能不执行：
     * - 当 select! 执行时，按照给定的分支顺序并发执行 Future，如果给定的 Future 都不处于 Poll::Ready 状态（Poll::Pending），则 default 分支会被执行。理论上 select! 内部每次轮询时如果给定的 Future 都返回 Poll::Pending，那么 default 可以无限次执行
     * - 除了可以重复执行的特性外，default 分支在部分场景下无法被运行。因为 default 分支运行的条件是给定的所有分支都不处于 Poll::Ready 状态，只要有一个分支在运行 default 分支前处于 Poll::Ready，那么 default 分支就不能被运行
     *
     * select! 宏虽然会并发运行给定的 Future，但是最终只会调用第一个完成的 Future 分支，然后主动结束 select! 代码块。这意味着即使后续有已完成的分支也不会被调用。
     *
     * ```rust
     * async fn select_default() {
     *     let mut a_fut = future::ready(4);
     *     let mut b_fut = future::ready(6);
     *     let mut total = 0;
     *
     *     select! {
     *         default => {
     *             // 因为所有的 Future 分支初始化都是 Poll:Ready 状态，所以 default 分支不会被执行
     *             println!("default branch");
     *         },
     *         a = a_fut => {
     *             println!("a_fut, total = {}, total + a = {}", total, total + a);
     *             total += a;
     *         },
     *         b = b_fut => {
     *             println!("b_fut, total = {}, total + b = {}", total, total +b);
     *             total += b;
     *         },
     *     };
     * }
     * futures::executor::block_on(select_default());
     * ```
     *
     * complete 分支运行的条件是所有的 Future 分支都完成（返回 Poll::Ready），但 select! 在第一个 Future 完成后就主动结束 select! 代码块。
     * 因此，为了获取所有的 Future 状态，complete 常常与循环搭配使用。
     * 同时，上一次 select! 结束时的 Future 分支（返回 Poll::Ready），为避免副作用，不应该再被 poll 执行，所以 complete 需要搭配 FusedFuture，表示已经结束。
     * 最后，当 complete 分支执行时，需要注意结束外层的循环。
     *
     * ```rust
     * async fn select_complete() {
     *     let mut a_fut = future::ready(4);
     *     let mut b_fut = future::ready(6);
     *     let mut total = 0;
     *
     *     loop {
     *         select! {
     *             a = a_fut => {
     *                 println!("a_fut, total = {}, total + a = {}", total, total + a);
     *                 total += a;
     *             },
     *             b = b_fut => {
     *                 println!("b_fut, total = {}, total + b = {}", total, total +b);
     *                 total += b;
     *             },
     *             default => {
     *                 println!("default branch");
     *             },
     *             complete => {
     *                 println!("complete branch");
     *                 break; // 需要主动结束外层的循环
     *             }
     *         };
     *     }
     * }
     * futures::executor::block_on(select_complete());
     * ```
     *
     * ### select 循环并发
     *
     * Promise 有一个非常好用的属性 thenable，它支持在 promise 结束后又返回一个类似 promise 的对象，当前的 promise 后续的 then 方法就会以新返回的类似 promise 的对象状态为准。
     *
     * rust 的 Future 也有类似的逻辑，更多参考：
     * - https://course.rs/advance/async/multi-futures-simultaneous.html#%E5%9C%A8-select-%E5%BE%AA%E7%8E%AF%E4%B8%AD%E5%B9%B6%E5%8F%91
     *
     * ```rust
     * async fn get_new_num() -> i32 {
     *     5
     * }
     * async fn run_on_new_num(_: i32) {}
     * async fn run_loop(
     *     mut interval_timer: impl Stream<Item = ()> + FusedStream + Unpin,
     *     starting_num: i32,
     * ) {
     *     use futures::FutureExt;
     *     use futures::StreamExt;
     *     let run_on_new_num_fut = run_on_new_num(starting_num).fuse();
     *     let get_new_num_fut = Fuse::terminated();
     *     pin_mut!(run_on_new_num_fut, get_new_num_fut);
     *     loop {
     *         select! {
     *             () = interval_timer.select_next_some() => {
     *                 // 定时器已结束，若`get_new_num_fut`没有在运行，就创建一个新的
     *                 if get_new_num_fut.is_terminated() {
     *                     get_new_num_fut.set(get_new_num().fuse());
     *                 }
     *             },
     *             new_num = get_new_num_fut => {
     *                 // 收到新的数字 -- 创建一个新的`run_on_new_num_fut`并丢弃掉旧的
     *                 run_on_new_num_fut.set(run_on_new_num(new_num).fuse());
     *             },
     *             // 运行 `run_on_new_num_fut`
     *             () = run_on_new_num_fut => {},
     *             // 若所有任务都完成，直接 `panic`， 原因是 `interval_timer` 应该连续不断的产生值，而不是结束后，执行到 `complete` 分支
     *             complete => panic!("`interval_timer` completed unexpectedly"),
     *         }
     *     }
     * }
     * ```
     *
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
            () = fut1 => {
                if fut1.is_terminated() {
                    println!("fut1.is_terminated")
                }
            },
            () = fut2 => {
                if fut1.is_terminated() {
                    println!("fut1.is_terminated")
                }
            },
        }
        println!("select3");
    }
    futures::executor::block_on(select3());

    async fn select_default() {
        let mut a_fut = future::ready(4);
        let mut b_fut = future::ready(6);
        let mut total = 0;

        select! {
            default => {
                // 因为所有的 Future 分支初始化都是 Poll:Ready 状态，所以 default 分支不会被执行
                println!("default branch");
            },
            a = a_fut => {
                println!("a_fut, total = {}, total + a = {}", total, total + a);
                total += a;
            },
            b = b_fut => {
                println!("b_fut, total = {}, total + b = {}", total, total +b);
                total += b;
            },
        };
    }
    futures::executor::block_on(select_default());

    async fn select_complete() {
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
                default => {
                    println!("default branch");
                },
                complete => {
                    println!("complete branch");
                    break; // 需要主动结束外层的循环
                }
            };
        }
    }
    futures::executor::block_on(select_complete());

    async fn get_new_num() -> i32 {
        5
    }
    async fn run_on_new_num(_: i32) {}
    async fn run_loop(
        mut interval_timer: impl Stream<Item = ()> + FusedStream + Unpin,
        starting_num: i32,
    ) {
        use futures::FutureExt;
        use futures::StreamExt;
        let run_on_new_num_fut = run_on_new_num(starting_num).fuse();
        let get_new_num_fut = Fuse::terminated();
        pin_mut!(run_on_new_num_fut, get_new_num_fut);
        loop {
            select! {
                () = interval_timer.select_next_some() => {
                    // 定时器已结束，若`get_new_num_fut`没有在运行，就创建一个新的
                    if get_new_num_fut.is_terminated() {
                        get_new_num_fut.set(get_new_num().fuse());
                    }
                },
                new_num = get_new_num_fut => {
                    // 收到新的数字 -- 创建一个新的`run_on_new_num_fut`并丢弃掉旧的
                    run_on_new_num_fut.set(run_on_new_num(new_num).fuse());
                },
                // 运行 `run_on_new_num_fut`
                () = run_on_new_num_fut => {},
                // 若所有任务都完成，直接 `panic`， 原因是 `interval_timer` 应该连续不断的产生值，而不是结束后，执行到 `complete` 分支
                complete => panic!("`interval_timer` completed unexpectedly"),
            }
        }
    }
}
