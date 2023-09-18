fn main() {
    /*
     * ## 模式匹配与匹配模式
     *
     * 模式有两种形式：refutable（可反驳模式）和irrefutable（不可反驳模式）：
     * 1. 模式匹配必须匹配成功，匹配失败就报错，主要是变量赋值型的(let/for/函数传参)模式匹配
     * 2. 模式匹配可以匹配失败，匹配失败时不执行相关代码，主要是分支（表达式/语句）类型的模式匹配
     *
     * Rust中为这两种匹配模式定义了专门的称呼
     * - 不可反驳模式(irrefutable)：一定会匹配成功，否则编译错误
     * - 可反驳模式(refutable)：可以匹配成功，也可以匹配失败，匹配失败的结果是不执行对应分支的代码
     *
     * 每一个 `模式匹配` 拥有自己的 `匹配模式`，当给出的 `匹配模式` 不符合 `模式匹配` 时，编译器会给出警告或直接报错。
     * - let变量赋值、for迭代、函数传参这三种模式匹配只接受 **不可反驳模式**
     * - if let 和 while let 只接受 **可反驳模式**。
     * - match的分支支持两种模式
     *      - 当明确给出分支的Pattern时，必须是可反驳模式，这些模式允许匹配失败，使用 `_` 作为最后一个分支时，是不可反驳模式，它一定会匹配成功
     *      - 如果只有一个Pattern分支，则可以是不可反驳模式，也可以是可反驳模式
     *
     * ## (完整的模式语法)[https://rust-book.junmajinlong.com/ch10/02_pattern_details.html#%E5%AE%8C%E6%95%B4%E7%9A%84%E6%A8%A1%E5%BC%8F%E8%AF%AD%E6%B3%95]
     * 1. 字面量模式
     *
     * 模式部分可以是字面量
     * ```rs
     * let x = 1;
     * match x {
     *   1 => println!("one"),
     *   2 => println!("two"),
     *   _ => println!("anything"),
     * }
     * ```
     * 2. 模式带有变量名（元组，数组，对象等）
     *
     * 分支的变量名是一个新变量
     * ```rs
     * fn main() {
     *  let x = (11, 22);
     *  let y = 10;
     *  match x {
     *    (22, y) => println!("Got: (22, {})", y),
     *    (11, y) => println!("y = {}", y),    // 匹配成功，输出22
     *    _ => println!("Default case, x = {:?}", x),
     *  }
     *  println!("y = {}", y);   // y = 10
     * }
     * ```
     * 上面的match会匹配第二个分支，同时为找到的变量y进行赋值，即y=22。这个y只在第二个分支对应的代码部分有效，跳出作用域后，y恢复为y=10。
     *
     * 3. 多选一模式
     *
     * 使用 `|` 可组合多个模式，表示逻辑或(or)的意思。
     * ```rs
     * let x = 1;
     * match x {
     *   1 | 2 => println!("one or two"),
     *   3 => println!("three"),
     *   _ => println!("anything"),
     * }
     * ```
     *
     * 4. 范围匹配模式
     *
     * 首先了解Rust支持数值和字符的范围，有如下几种范围表达式：
     * |  Production  | Syntax | Type | Range |
     * | :----: | :----: | :----: | :----: |
     * | RangeExpr |	start..end	 | std::ops::Range  |	start ≤ x < end |
     * | RangeFromExpr |	start..	 | std::ops::RangeFrom |	start ≤ x |
     * | RangeToExpr |	..end | 	std::ops::RangeTo	 |x < end |
     * | RangeFullExpr |	..	 | std::ops::RangeFull |	- |
     * | RangeInclusiveExpr |	start..=end	 | std::ops::RangeInclusive |	start ≤ x ≤ end |
     * | RangeToInclusiveExpr |	..=end | 	std::ops::RangeToInclusive |	x ≤ end |
     *
     * 但范围作为模式匹配的Pattern时，只允许使用全闭合的..=范围语法，其他类型的范围类型都会报错。
     * 这是因为Rust的设计目标之一是强调安全性和可预测性，而使用闭区间可以更容易地确保这些目标。
     *
     * ## 总结
     * - 匹配模式有两种，irrefutable（不可反驳模式）和refutable（可反驳模式），区别在于不可反驳模式一定会匹配成功，否则编译报错
     * - 变量赋值匹配、if let、while let等模式匹配拥有自己的指定的匹配模式，如果给出的匹配模式不属于当前的模式匹配，编译器会给出警告或直接报错
     * - 匹配有很多语法，其中注意复杂数据的模式匹配、or、范围匹配
     */

    // irrefutable 不可反驳模式
    let a = 5;

    // irrefutable 不可反驳模式
    fn print_int(v: i64) {
        println!("{}", v);
    }

    // irrefutable 不可反驳模式，一定会匹配成功，否则编译错误 print_int("a");
    print_int(a);

    // refutable 可反驳模式，可以匹配成功，也可以匹配失败，匹配失败结果不执行对应分支代码
    if let 5 = a {
        println!("a is {a}");
    }

    // match分支具有irrefutable（不可反驳模式）和refutable（可反驳模式）两种模式
    match a {
        5 => println!("{a}"),
        6 => println!("{}", 6),
        _ => {
            println!("{a}");
        } // match匹配必须要穷举，哪怕 `_` 的含义就是不定
    }

    // let变量赋值时使用可反驳的模式(允许匹配失败)，编译失败。把2赋值给Some(x)即用2匹配给Some(x)
    // let Some(x) = 2;

    // if let 和 while let 只接受 `可反驳模式`，即可以匹配成功，也可以匹配失败的模式，如果给出一个一定匹配成功的模式，编译器会给出警告
    // 只接受 refutable 可反驳模式，但给出的模式是一定匹配成功的irrefutable 不可反驳模式
    if let c = a {
        println!("{}", c);
    };

    // 分支中的 `|` 表示 或(or)，并非数字的或计算关系
    let x = 1;
    match x {
        1 | 2 => println!("one or two"),
        3 => println!("three"),
        _ => println!("anything"),
    }

    // 范围
    match x {
        1..=2 => println!("{x} is one or two"),
        _ => println!("anything"),
    }

    // 范围
    if let 1..=2 = a {
        println!("{a} is one or two")
    }
}
