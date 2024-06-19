fn main() {
    /*
     *
     * ## 转换和边界异常处理
     *
     * Result 和 Option 在业务程序中很常见，并且通常还需要自定义错误类型以便快速定位问题。
     *
     * ### 组合器
     *
     * 因为 Result 和 Option 类型很常见，但使用真实值时需要取出再判断，或者 Result 和 Option 互相转换，这些操作都显得比较琐碎，所以 rust 提供了一些组合器简化这些操作。
     * 这些操作的功能与 JavaScript 的与或非 `|, ||, &, &&, !` 的功能类似，可以返回某个值。
     *
     *
     * 组合器不同于组合模式，组合器更多的是用于对返回结果的类型进行变换：例如使用 ok_or 将一个 Option 类型转换成 Result 类型。
     * > 组合模式：将对象组合成树形结构以表示“部分整体”的层次结构。组合模式使得用户对单个对象和组合对象的使用具有一致性。–GoF \<\<设计模式\>\>
     *
     * ```rust
     * let id: Option<i32> = Some(1);
     * let id: Result<i32, &str> = id.ok_or("没有数据的错误信息");
     * println!("{}", id.unwrap());
     * ```
     *
     * #### or 和 and
     * 这两个方法会对两个 Option 表达式或两个 Result 表达式做逻辑组合，最终返回 Option 或 Result。
     *
     * - or()，表达式按照顺序求值，若任何一个表达式的结果是 Some 或 Ok，则该值会立刻返回
     * - and()，若两个表达式的结果都是 Some 或 Ok，则第二个表达式中的值被返回。若任何一个的结果是 None 或 Err ，则立刻返回。
     *
     * ```rust
     * // or 和 and
     * let id1: Option<i32> = Some(1);
     * let id2: Option<i32> = None;
     * println!(
     *     "id1 or id2 = {:?}, id1 and id2 = {:?}, id2 or id1 = {:?}, id2 and id1 = {:?}",
     *     id1.or(id2),
     *     id1.and(id2),
     *     id2.or(id1),
     *     id2.and(id1)
     * )
     * ```
     * 除了 or 和 and 之外，Rust 还提供了异或 xor ，但是它只能应用在 Option 上，不能应用在 Result 上，因为不能对一个值和错误进行异或操。
     *
     * #### or_else() 和 and_then()
     * 它们跟 or() 和 and() 类似，唯一的区别在于，它们的第二个表达式是一个返回 Option 或 Result 的闭包。
     * ```rust
     * // or_else 或 and_then
     * let id1: Option<i32> = Some(1);
     * let id2 = || None;
     * let id3 = |_| Some(1);
     *
     * // 注意 impl 不能作为直接作为普通变量的类型
     * // let id3: impl Fn() -> Option<i32> = || None; error `impl Trait` is only allowed in arguments and return types
     * println!(
     *     "id1.or_else(id2) = {:?}, id1.and_then(id3) = {:?}",
     *     id1.or_else(id2),
     *     id1.and_then(id3)
     * );
     * ```
     *
     * #### filter
     * filter 用于对 Option 进行条件过滤：
     * ```rust
     * // filter 用于对 Option 进行条件过滤
     * let id1 = Some(1);
     * let is_even = |x: &i32| x % 2 == 0;
     * println!("id1.filter(is_event) = {:?}", id1.filter(is_even));
     * ```
     *
     * #### map, map_err, map_or, map_or_else
     * map 可以将 Some 中的值映射为另一个 Some，Ok 类似：
     *
     * ```rust
     * let id1 = Some(1);
     * let mapFn = |x: i32| -> i32 {
     *     if x > 2 {
     *         2
     *     } else {
     *         1
     *     }
     * };
     * println!("id1.map(mapFn) = {:?}", id1.map(mapFn))
     * ```
     *
     * 如果需要对 Result 的 Err 的信息进行修改，就需要使用 map_err。
     *
     * map_or 和 map_or_else 在 map 的基础上添加了一个默认值，区别是 map_or 给定指定类型的默认值，map_or_else 通过闭包提供默认值。
     *
     * #### ok_or ok_or_else
     * ok_or 和 ok_or_else 都是将 Option 转换为 Result 的组合器，两者都接受一个 Err 默认参数，ok_or 直接给定类型参数，ok_or_else 通过闭包给定默认参数。
     * 
     * ```rust
     * let id: Option<i32> = Some(1);
     * let id: Result<i32, &str> = id.ok_or("没有数据的错误信息");
     * println!("{}", id.unwrap());
     * ```
     * 
     *
     */

    //  Option 转换成 Result
    let id: Option<i32> = Some(1);
    let id: Result<i32, &str> = id.ok_or("没有数据的错误信息");
    println!("{}", id.unwrap());

    // or 和 and
    let id1: Option<i32> = Some(1);
    let id2: Option<i32> = None;
    println!(
        "id1.or(id2) = {:?}, id1.and(id2) = {:?}, id2.or(id1) = {:?}, id2.and(id1) = {:?}",
        id1.or(id2),
        id1.and(id2),
        id2.or(id1),
        id2.and(id1)
    );

    // or_else 或 and_then
    let id1: Option<i32> = Some(1);
    let id2 = || None;
    let id3 = |_| Some(1);

    // 注意 impl 不能作为直接作为普通变量的类型
    // let id3: impl Fn() -> Option<i32> = || None; error `impl Trait` is only allowed in arguments and return types
    println!(
        "id1.or_else(id2) = {:?}, id1.and_then(id3) = {:?}",
        id1.or_else(id2),
        id1.and_then(id3)
    );

    // filter 用于对 Option 进行条件过滤
    let id1 = Some(1);
    let is_even = |x: &i32| x % 2 == 0;
    println!("id1.filter(is_event) = {:?}", id1.filter(is_even));

    // map 可以将 Some 中的值映射为另一个 Some，Ok 类似
    let id1 = Some(1);
    let mapFn = |x: i32| -> i32 {
        if x > 2 {
            2
        } else {
            1
        }
    };
    println!("id1.map(mapFn) = {:?}", id1.map(mapFn))
}
