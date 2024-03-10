use ilearn::{run, Config};
use std::{env, error::Error, fs, process};

fn main() {
    /*
     *
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
     * 如果闭包只有一个返回表达式，可以简化定义：
     *
     * ```rust
     * |param1| 返回表达式
     * ```
     *
     * 特别注意：闭包中最后一行表达式返回的值，就是闭包执行后的返回值。
     *
     * ```rust
     * fn main() {
     *    let x = 1;
     *    let sum = |y| x + y;
     *
     *    assert_eq!(3, sum(2));
     * }
     * ```
     * 代码中闭包 sum，它拥有一个入参 y，同时捕获了作用域中的 x 的值，因此调用 sum(2) 意味着将 2（参数 y）跟 1（x）进行相加，最终返回它们的和：3。
     *
     * 可以看到 sum 非常符合闭包的定义：可以赋值给变量，允许捕获调用者作用域中的值。
     *
     * ### 闭包类型推导
     * Rust 是静态语言，因此所有的变量都具有类型，但是得益于编译器的强大类型推导能力，在很多时候我们并不需要显式地去声明类型。
     *
     * 但显然函数并不在此列，因为函数往往会作为 API 提供给你的用户，你的用户必须在使用时知道传入参数的类型和返回值类型，**因此必须手动为函数的所有参数和返回值指定类型**。
     *
     * 与函数相反，闭包并不会作为 API 对外提供，因此它可以享受编译器的类型推导能力，**无需标注参数和返回值的类型**。
     *
     * 为了增加代码可读性，有时候我们会显式地给类型进行标注，出于同样的目的，也可以给闭包标注类型，在下面sum函数中，定义两个参数 `x y` 和返回值为 i32 类型。
     * ```rust
     * let sum = |x: i32, y: i32| -> i32 {
     *     x + y
     * }
     * ```
     * 与之相比，不标注类型的闭包声明会更简洁些：let sum = |x, y| x + y，需要注意的是，针对 sum 闭包，如果你只进行了声明，但是没有使用，编译器会提示你为 x, y 添加类型标注，因为它缺乏必要的上下文：
     *
     * 这是因为虽然类型推导很好用，但是它不是泛型，当编译器推导出一种类型后，它就会一直使用该类型：
     *
     * ```rust
     * let example_closure = |x| x;
     * let s = example_closure(String::from("hello"));
     * let n = example_closure(5);
     * ```
     *
     * 首先，在 s 中，编译器为 x 推导出类型 String，但是紧接着 n 试图用 5 这个整型去调用闭包，跟编译器之前推导的 String 类型不符，因此报错。
     *
     * ### 结构体中的闭包
     *
     * 假设我们要实现一个简易缓存，功能是获取一个值，然后将其缓存起来，那么可以这样设计：
     * - 一个闭包用于获取值
     * - 一个变量，用于存储该值
     *
     * 可以使用结构体来代表缓存对象，最终设计如下：
     * ```rust
     * struct Cacher<T>
     * where
     *     T: Fn(u32) -> u32,
     * {
     *     query: T,
     *     value: Option<u32>,
     * }
     * ```
     * T是一个泛型，`Fn(u32) -> u32` 这一长串是 T 的特征约束，即 Fn(u32) -> u32 是一个特征，用来表示 T 是一个**闭包类型**。
     * 特征 `Fn(u32) -> u32` 从表面来看，就对闭包形式进行了显而易见的限制：该闭包拥有一个u32类型的参数，同时返回一个u32类型的值。
     * 需要注意的是，其实 Fn 特征不仅仅适用于闭包，还适用于函数，因此上面的 query 字段除了使用闭包作为值外，还能使用一个具名的函数来作为它的值。
     * > 熟悉JavaScript的同学知道，在JavaScript中闭包的声明和函数是一样的，闭包只是比函数访问了外部的变量。
     *
     * 接着为结构体实现方法：
     * ```rust
     * impl<T> Cacher<T>
     * where
     *     T: Fn(u32) -> u32,
     * {
     *     fn new(query: T) -> Cacher<T> {
     *         Cacher { query, value: None }
     *     }
     *
     *     fn value(&mut self, arg: u32) -> u32 {
     *         match self.value {
     *             Some(v) => v,
     *             None => {
     *                 let v = (self.query)(arg);
     *                 // 闭包是实现了 Fn trait 的类型，而不是直接的函数指针。因此不能直接使用self.query形式，需要用(self.query)标识
     *                 // let v = self.query(arg);
     *                 self.value = Some(v);
     *                 v
     *             }
     *         }
     *     }
     * }
     * ```
     *
     * > 可以修改u32类型为泛型，让结构体通用，` E: Copy `
     *
     * 调用Cacher，验证缓存：
     * ```rust
     * fn call_with_different_values() {
     *     let mut c = Cacher::new(|a| a);
     *
     *     let v1 = c.value(1);
     *     let v2 = c.value(2);
     *
     *     assert_eq!(v2, 1);
     * }
     *
     * call_with_different_values();
     * ```
     *
     * ### 捕获作用域中的值
     * 前面提到过，闭包相比较函数多出来的一个功能就是能够访问作用域中的值：
     *
     * x 并不是闭包 equal_to_x 的参数，但是它依然可以去使用 x，因为 equal_to_x 在 x 的作用域范围内。
     * 而对于函数来说，就算你把函数定义在 main 函数体中，它也不能访问 x。
     * ```rust
     * fn main() {
     *     let x = 4;
     *     let equal_to_x = |z| z == x;
     *     let y = 4;
     *     assert!(equal_to_x(y));
     *
     * // fn equal_to_x(z: i32) -> bool {
     * //    z == x
     * // }
     * // assert!(equal_to_x(y));
     * }
     * ```
     *
     * > 函数的表示很简单，它是一段编译之后就不会变化的代码段，我们可以用一个简单的地址就能代表它。
     * > 而闭包则不一样，存放了什么数据、存放的数据类型是什么、数据存放在哪里、存放的数据是否可以修改、其生命周期如何，这些都是需要考虑的事情。
     * > 所以在区分闭包和函数这点上，动态语言JavaScript可以做到统一函数和闭包，因为只有在执行时才确定数据。
     * > https://www.zhihu.com/question/628166233
     *
     * ### 闭包对内存的影响
     * 当闭包从环境中捕获一个值时，会分配内存去存储这些值。对于有些场景来说，这种额外的内存分配会成为一种负担。
     * 与之相比，函数就不会去捕获这些环境值，因此定义和使用函数不会拥有这种内存负担。
     *
     * ### 三种 Fn 特征
     * 闭包捕获变量有三种途径，恰好对应函数参数的三种传入方式：转移所有权、可变借用、不可变借用，因此相应的 Fn 特征也有三种：
     * 1. FnOnce，该类型的闭包会拿走**被捕获变量的所有权**，因此该闭包只能运行一次，这也是Once的来源。
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
     *
     * 2. FnMut，它以可变借用的方式捕获了环境中的值，因此可以修改该值
     * 
     *
     */

    let closure = |x| x;
    println!("{}", closure(1));
    // println!("{}", closure(String::from("x"))); 如果再次使用closure则会报错，因为closure的类型已经被确定了

    // 增加缓存结构体
    struct Cacher<T, E>
    where
        T: Fn(E) -> E,
        E: Copy,
    {
        query: T,
        value: Option<E>,
    }

    impl<T, E> Cacher<T, E>
    where
        T: Fn(E) -> E,
        E: Copy,
    {
        fn new(query: T) -> Cacher<T, E> {
            Cacher { query, value: None }
        }

        fn value(&mut self, arg: E) -> E {
            match self.value {
                Some(v) => v,
                None => {
                    let v = (self.query)(arg);
                    // 闭包是实现了 Fn trait 的类型，而不是直接的函数指针。因此不能直接使用self.query形式，需要用(self.query)标识
                    // let v = self.query(arg);
                    self.value = Some(v);
                    v
                }
            }
        }
    }

    fn call_with_different_values() {
        let mut c = Cacher::new(|a| a);

        let v1 = c.value(1);
        let v2 = c.value(2);

        assert_eq!(v2, 1);
    }
    call_with_different_values();

    let x = 4;
    let closure_to = |z| z == x; // 访问作用域中的变量
    let y = 4;
    assert!(closure_to(y));

    fn fn_once<F>(func: F)
    where
        F: FnOnce(usize) -> bool + Copy,
    {
        println!("{}", func(3));
        println!("{}", func(4));
    }

    let x = vec![1, 2, 3];
    fn_once(|z| z == x.len())
}
