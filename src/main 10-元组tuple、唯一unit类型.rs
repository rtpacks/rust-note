use std::vec;

fn main() {
    /*
     * ## 元组
     * Rust的tuple类型可以存放0个、1个或多个任意数据类型的数据，这些数据内容和顺序是固定的。使用tup.N的方式可以访问索引为N的元素。
     * - https://rust-book.junmajinlong.com/ch3/05_tuple_unit.html
     * - https://course.rs/basic/compound-type/tuple.html
     *
     * ```rs
     * let tup = (1, 2.3, 4);
     * let first = tup.0;
     * let (first, second, thirf) = tup;
     *
     * ```
     *
     * 注意，访问tuple元素的索引必须是编译期间就能确定的数值，而不能是变量。当tup只有一个元素时，不能省略逗号！
     *
     * ```rs
     * let tup = (1, 2, 3);
     * let tup = ("Hello", ); // 不能省略逗号，用于判断类型
     * ```
     *
     * ## unit 唯一类型
     * 不保存任何数据的tuple表示为()。在Rust中，它是特殊的，它有自己的类型：unit。unit类型的写法为()，该类型也只有一个值，写法仍然是()。
     * 
     * ```rs
     * //       类型  值
     * let unit: () = ()
     * ```
     * 
     * unit类型通常用在那些不关心返回值的函数中。在其他语言中，那些不写return语句或return不指定返回内容的的函数，一般表示不关心返回值。在Rust中可将这种需求写为return ()。
     */

    let tup = (1, 2, 3.3);
    let first = tup.0;
    let (first, second, third) = tup;
    println!("{:?}", first);
    // let single_tup = (1); error
    let single_tup = (1,); // right
    println!("{:?}, {:?}", tup, single_tup);
}
