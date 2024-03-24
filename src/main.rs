use ilearn::{run, Config};
use std::{env, error::Error, fs, process};

fn main() {
    /*
     * ## 闭包 Closure
     *
     * 闭包是一种匿名函数，它可以赋值给变量也可以作为参数传递给其它函数，不同于函数的是，它允许捕获调用者作用域中的值。
     *
     * Rust 闭包在形式上借鉴了 Smalltalk 和 Ruby 语言，与函数最大的不同就是它的参数是通过 |parm1| 的形式进行声明，如果是多个参数就 |param1, param2,...|，闭包的形式定义：
     * ```rust
     * |param1, param2,...| {
     *     语句1;
     *     语句2;
     *     返回表达式
     * }
     * ```
     *
     * ### 三种 Fn 特征
     * 闭包捕获变量有三种途径，恰好对应函数参数的三种传入方式：转移所有权、可变借用、不可变借用，因此相应的 Fn 特征也有三种：FnOnce、FnMut、Fn。
     *
     * ### FnOnce
     * FnOnce，该类型的闭包会拿走**被捕获变量的所有权**，因此该闭包只能运行一次，这也是Once的来源。
     * ```rust
     * fn fn_once<F>(func: F)
     * where
     *     F: FnOnce(usize) -> bool,
     * {
     *     println!("{}", func(3));
     *     println!("{}", func(4));
     * }
     *
     * let x = vec![1, 2, 3];
     * fn_once(|z| { z == x.len() })
     * ```
     * 仅实现 FnOnce 特征的闭包在调用时会转移被捕获变量的所有权，因此不能对闭包进行二次调用（内部被捕获的变量失去所有权，调用会出错）：
     * ```rust
     * println!("{}", func(3));
     * println!("{}", func(4)); // 调用报错，在调用func(3)后，x变量已经失去所有权，再次使用x变量导致出错
     * ```
     * 如何解决这个问题呢？只需要给传入的闭包加上Copy特征，闭包就能够对被捕获的变量自动Copy，这样就不存在所有权的问题了。
     * ```rust
     * fn fn_once<F>(func: F)
     * where
     *     F: FnOnce(usize) -> bool + Copy // 增加Copy Trait，闭包能够对被捕获的变量自动Copy，就不存在所有权的问题了。
     * {}
     * ```
     *
     * 另外：如果想强制闭包取得捕获变量的所有权，可以在参数列表前添加 move 关键字，这种用法通常用于闭包的生命周期大于捕获变量的生命周期时，例如将闭包返回或移入其他线程。
     * ```rust
     * let x = vec![1, 2, 3];
     * fn_once(move |z| { z == x.len() }); // 强制闭包取得捕获变量的所有权
     * ```
     * ### FnMut
     * FnMut，它以可变借用的方式捕获了环境中的值，因此可以修改该值
     * ```rust
     * let mut s = String::new();
     * let update_string =  |str| s.push_str(str);
     * update_string("hello");
     * ```
     * 在闭包中，我们调用 s.push_str 去改变外部 s 的字符串值，`push_str(&mut self)` 需要变量的可变借用，因此这里闭包捕获了它的可变借用的**使用**操作。
     * 执行后报错了，想要在闭包内部捕获**可变借用\<操作\>**，需要把该闭包变量声明为可变类型，也就是 update_string 要修改为 mut update_string：
     * ```rust
     * let mut s = String::new();
     * let mut update_string = |str| s.push_str(str);
     * update_string("Hello");
     * println!("{s}");
     * ```
     *
     * 闭包捕获变量的可变借用的**使用**操作，闭包就会变为FnMut类型，对应的变量也需要设置为可变才能够调用闭包。注意：FnMut是类型。
     * 这种写法有点反直觉，但如果把闭包变量仅仅当成一个普通变量，那么这种声明就比较合理了（可变需要来自可变）。
     *
     * ### Fn
     * 仅需要不可变地访问其上下文的函数属于Fn trait，并且只要上下文在作用域中存在，就可以在任意位置调用。
     * ```rust
     * // 闭包类型只与闭包怎么**使用**被捕获变量的操作有关系，与变量自己的类型、捕获变量的方式没有直接关系
     * let mut s = String::new();
     * let mut update_string = |str| s.push_str(str); // FnMut
     * let mut update_string = |str| println!("{}", s.len()); // Fn
     * update_string("Hello");
     * println!("{s}");
     * ```
     * 为什么是不可变引用的使用操作？从 len 函数的第一个参数 Self 中可以看到 `&self` 是一个不可变引用。
     *
     * ### 闭包是所有权状态的描述
     * 闭包其实就是所有权各种状态的描述：拥有所有权、所有权的可变引用、所有权的独不可变引用、没有所有权，对应到闭包的类型就为FnOnce、FnMut、Fn、fn。
     * 所以闭包的类型与被捕获的变量类型没有关系，而是与闭包怎么**使用**被捕获变量有关系，捕获操作简单来说是怎么使用变量。
     *
     * 比如上述（FnMut）的例子中，闭包捕获到变量进行了可变引用的使用操作这个动作，那么闭包就成为FnMut类型，这意味着闭包被调用时会修改被捕获的变量。如果改成以下示例：
     * ```rust
     * // 闭包类型只与闭包怎么**使用**被捕获变量的操作有关系，与变量自己的类型、捕获变量的方式没有直接关系
     * let mut s = String::new();
     * let mut update_string = |str| s.push_str(str); // FnMut
     * let mut update_string = |str| println!("{}", s.len()); // Fn
     * update_string("Hello");
     * println!("{s}");
     * ```
     * - 变量的可变引用可以进行可变引用操作 `s.push_str()`，因为 `push_str` 的 `Self` 为 `&mut self`，被闭包捕获可变引用的使用操作，那么闭包就为FnMut；
     * - 变量的可变引用也可以进行不可变引用操作 `s.len()`，因为 `len` 的 `Self` 为 `&self`，被闭包捕获不可变引用的使用操作，那么闭包就为Fn。
     *
     * 又或者以下例子，闭包捕获不可变引用的使用操作：
     * ```rust
     * let s = String::from("Hello World");
     * let compare_len_with_s = |str: &str| println!("{}", str.len() == s.len());
     * compare_len_with_s("Hello");
     * println!("{s}");
     * ```
     * 为什么是不可变引用的使用操作？从 len 函数的第一个参数 Self 中可以看到 `&self` 是一个不可变引用。
     *
     * 也就是理解闭包使用被捕获变量的操作：看变量怎么用（函数的`Self`是什么），Self是什么类型，它所代表的使用操作被闭包捕获，闭包就是什么类型。
     * 比如 `s.len()` len 函数的 `&self` 意味着闭包捕获的是一个不可变引用的使用操作，闭包就是Fn，对应的闭包变量可以不mut。
     * 又比如 `s.push_str()` 的 `&mut self` 意味着闭包捕获的是一个可变引用的使用操作，闭包就是FnMut，对应的闭包变量需要mut。
     *
     * https://zhuanlan.zhihu.com/p/288626364 这张图属于从闭包范围的角度来解释闭包的关系。
     * fn>Fn>FnMut>FnOnce，fn extends Fn extends FnMut extends FnOnece，如能实现FnMut的一定能实现Fn。
     *
     * 既然 fn extends Fn extends FnMut extends FnOnce，那么从继承和多态的角度上解释闭包的关系：https://zhuanlan.zhihu.com/p/341815515。
     * FnOnce被FnMut继承，那么FnMut类型就可以赋值给FnOnce类型（多态），同样，Fn能够赋值给FnMut、FnOnce类型。
     *
     * > 为什么需要设计成 Fn extends FnMut extends FnOnce呢？
     * >
     * > 来自GPT的回答：这种继承关系的设计允许Rust在编译时进行更精确的借用检查，确保内存安全。它反映了一个从“可能完全消耗捕获的变量（FnOnce）”到“可能改变捕获的变量（FnMut）”再到“不改变捕获的变量（Fn）”的权限层次。这样的设计使得Rust的闭包既灵活又安全，能够根据不同的需求选择合适的闭包类型。
     *
     * ### move 和 Fn
     * move 常与 FnOnce 搭配使用，但实际上使用了 move 的闭包依然可能实现了 Fn 或 FnMut 特征。
     *
     * 因为，一个闭包实现了哪种 Fn 特征取决于该闭包如何**使用**被捕获的变量，而不是取决于闭包如何捕获它们。move 本身强调的就是后者，闭包如何捕获变量：
     *
     * 符合直觉的move和FnOnce示例，move强制转移变量的所有权，FnOnce需要消耗变量的所有权：
     * ```rust
     * fn exec<F: FnOnce()>(f: F)  {
     *     f()
     * }
     * let s = String::new();
     * let update_string =  move || println!("{}",s);
     * exec(update_string);
     * ```
     *
     * 但是如果保留move，将FnOnce改成Fn，编译也是可以正常的：
     *
     * ```diff
     * - fn exec<F: FnOnce()>(f: F)  {}
     * + fn exec<F: Fn()>(f: F)  {}
     * ```
     *
     * ```rust
     * fn exec<F: Fn()>(f: F)  {
     *     f()
     * }
     * let s = String::new();
     * let update_string =  move || println!("{}",s);
     * exec(update_string);
     * ```
     *
     * 为什么可以正常运行？明确 move 是闭包捕获变量的方式，不是闭包使用变量的方式。
     *
     * 闭包的类型取决于闭包如何使用变量（闭包使用变量的方式），即变量操作时函数的 `Self` 的类型。
     * ```rust
     * let s = String::from("Hello World");
     * let closure = || println!("{s}"); // Fn
     * let closure = move || println!("{s}"); // Fn
     * let closure = || println!("{}", s.len()); // Fn
     * let closure = move || println!("{}", s.len()); // Fn，因为 `len(&self)` 只需要不可变引用，所以属于Fn
     * ```
     *
     * 闭包捕获变量的方式是什么意思呢，看以下代码，闭包的三种捕获方式，捕获变量，捕获可变引用，捕获不可变引用。
     * 虽然捕获方式不同，但是由于使用方式 `len(&self)` 中 `&self`，所以三者都是Fn闭包类型
     * ```rust
     * let mut s = String::from("Hello World");
     * let closure = || println!("{}", s.len()); // Fn
     * let closure = || println!("{}", (&s).len()); // Fn
     * let closure = move || println!("{}", s.len()); // Fn
     * ```
     * 上面的示例再一次验证：一个闭包实现了哪种 Fn 特征取决于该闭包如何**使用**被捕获的变量，而不是取决于闭包如何捕获它们。
     * 
     * > 注意，`(&mut s).len()` 使用方式是先创建可变引用，也就是FnMut和Fn都存在，所以闭包类型是FnMut。
     * > ```rust
     * > let mut s = String::from("Hello World");
     * > let mut closure = || println!("{}", (&mut s).len()); // FnMut
     * > ```
     *
     * ### 三种Fn的关系
     *
     *
     *
     *
     *
     * ### 总结
     * 闭包（closure）是函数指针（function pointer）和上下文（context）的组合。
     * 没有上下文的闭包就是一个函数指针。
     * 带有不可变上下文（immutable context）的闭包属于Fn
     * 带有可变上下文（mutable context）的闭包属于FnMut
     * 拥有其上下文的闭包属于FnOnce
     *
     */

    fn fn_once<F>(func: F)
    where
        F: FnOnce(usize) -> bool + Copy,
    {
        println!("{}", func(3));
        println!("{}", func(4));
    }

    let x = vec![1, 2, 3];
    fn_once(|z| z == x.len());
    println!("{:?}", x);

    let mut s = String::new();
    // let update_string = |str| s.push_str(str); 闭包捕获可变借用，需要闭包变量也设置为可变
    let mut update_string = |str| s.push_str(str);
    update_string("Hello");
    println!("{s}");

    // 闭包类型只与闭包怎么捕获变量的操作有关系，与变量自己的类型没有直接关系。
    let mut s = String::new();
    let mut update_string = |str| s.push_str(str); // FnMut
    let mut update_string = |str| println!("{}", s.len()); // Fn
    update_string("Hello");
    println!("{s}");

    let s = String::from("Hello World");
    let compare_len_with_s = |str: &str| println!("{}", str.len() == s.len());
    compare_len_with_s("Hello");
    println!("{s}");

    let mut s = String::from("Hello World");
    let mut closure = || {
        println!("{}", &mut s);
        s
    };
    closure();

    let mut s = String::from("Hello World");
    let mut closure = move || {
        println!("{}", &mut s);
    };
    closure();

    let mut s = String::from("Hello World");
    let ss = &s;
    let mut closure = || {
        println!("{}", s.len());
        println!("{}", (&s).len());
        // println!("{}", (&mut s).len());
        // println!("{}", &mut 11);
        println!("{}", ss.len());
    };
    closure();
}
