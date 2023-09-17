fn main() {

    /*
     * ## 模式匹配
     * Rust中经常使用到的一个强大功能是模式匹配(pattern match)，例如let变量赋值本质上就是在进行模式匹配。得益于Rust模式匹配功能的强大，使用模式匹配比不使用模式匹配，往往会减少很多代码。
     *
     * 可在如下几种情况下使用模式匹配：
     * - let变量赋值
     * - 函数参数传值时的模式匹配
     * - match分支
     * - if let
     * - while let
     * - for迭代的模式匹配
     *
     * 在涉及到赋值时就会存在模式匹配，如定义变量/更新变量值/函数调用传递参数等。
     *
     * 常见的使用模式匹配的方式有:
     * 1. match 
     * match是最为强大的模式匹配方式，它的形式如下
     * ```rs
     * match VALUE {
     *   PATTERN1 => EXPRESSION1,
     *   PATTERN2 => EXPRESSION2,
     *   PATTERN3 => EXPRESSION3,
     * }
     * ```
     *
     * =>左边的是各分支的模式，VALUE将与这些分支逐一进行匹配，=>右边的是各分支匹配成功后执行的代码。每个分支后使用逗号分隔各分支，最后一个分支的结尾逗号可以省略(但建议加上)。
     *
     * **match会从前先后匹配各分支，一旦匹配成功则不再继续向下匹配。**
     * ```rs
     * let x = (11, 22);
     *  match x {
     *    (22, a) => println!("(22, {})", a),   // 匹配失败
     *    (a, b) => println!("({}, {})", a, b), // 匹配成功，停止匹配
     *    (a, 11) => println!("({}, 11)", a),   // 匹配失败
     *  }
     * ```
     *
     * 如果某分支对应的要执行的代码只有一行，则直接编写该行代码，如果要执行的代码有多行，则需加上大括号包围这些代码。无论加不加大括号，每个分支都是一个独立的作用域。
     * match结构自身也是表达式，它有返回值，且可以赋值给变量。match的返回值由每个分支最后执行的那行代码决定。Rust要求match的每个分支返回值类型必须相同，且如果是一个单独的match表达式而不是赋值给变量时，每个分支必须返回()类型。
     *
     * 如 `println!("{}", a)`（没有分号）就表示返回唯一类型（`()`），而加上分号后 `println!("{}", a);` 就表示返回空（void）。
     * 
     * tip: match 匹配是穷尽的，也就是 match 中需要处理所有的 case。当然可以使用通配模式和 _ 占位符，如使用一个变量来匹配所有的值，使用 `_`，忽略剩余的情况。
     *
     * 2. if let
     *
     * 如果只关心某一种情况，即某一种分支，可以选择if let。if let是match的一种特殊情况的语法糖：当只关心一个match分支，其余情况全部由_负责匹配时，可以将其改写为更精简if let语法。if let可以结合else if、else if let和else一起使用。
     *
     * 匹配成功，因此执行大括号内的代码，if let是独立作用域，变量a b只在大括号中有效
     * ```rs
     * let x = (11, 22);
     * if let (a, b) = x {
     *   println!("{},{}", a, b);
     * }
     * ```
     * 等价于如下代码
     * ```rs
     *   let x = (11, 22);
     *  match x {
     *    (a, b) => println!("{},{}", a, b),
     *    _ => (),
     *  }
     * ```
     *
     * 3. while let
     *
     * 只要while let的模式匹配成功，就会一直执行while循环内的代码。
     *
     * 当stack.pop成功时，将匹配Some(top)成功，并将pop返回的值赋值给top，当没有元素可pop时，返回None，匹配失败，于是while循环退出。
     * ```rs
     *  let mut stack = Vec::new();
     *  stack.push(1);
     *  stack.push(2);
     *  stack.push(3);
     *  while let Some(top) = stack.pop() {
     *    println!("{}", top);
     *  }
     * ```
     *
     * 4. for迭代
     *
     * for迭代也有模式匹配的过程：为控制变量赋值。例如：
     * ```rs
     * let v = vec!['a','b','c'];
     *  for (idx, value) in v.iter().enumerate(){
     *      println!("{}: {}", idx, value);
     *  }
     * ```
     * 在JavaScript/TypeScript中，这种情况被称为解构。
     * 
     * ### 阅读
     * - https://rustwiki.org/zh-CN/book/ch06-02-match.html
     */
}
