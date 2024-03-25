use ilearn::{run, Config};
use std::{env, error::Error, fs, process};

fn main() {
    /*
     * ## 闭包 Closure
     *
     * > 本章内容较长，且在本章内容尾部更新了对闭包的认识，读者应读完全章，不要取其中部分。
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
     * ### 闭包作为函数返回值
     * 在实现 `Cacher` 中我们将闭包作为参数传递给函数（方法），现在考虑如何将闭包应用在函数的返回值上，因为只有这样，才能将内部值传递出去。
     *
     * 闭包在Rust中有一个独特的特性：每个闭包都有其自己独特的匿名类型（这个类型就是类似 `i32 String` 的一种数据格式类型），这是因为**闭包类型不仅仅是由其参数和返回类型定义**的，还包括它捕获的环境。每个闭包根据其捕获的环境（变量、生命周期等）具有不同的类型。
     * 即使两个闭包有相同的签名，它们也被认为是不同的类型。这意味着直接返回闭包类型（`Fn(i32) -> i32`）是不可能的，因为闭包的具体类型是未知的，且无法直接命名。
     *
     *
     * #### 正确标注闭包作为函数返回值
     * 使用impl Trait语法允许我们返回一个实现了指定trait的类型，而不需要指定具体的类型。（impl Trait 形式来说明一个函数返回了一个类型，该类型实现了某个特征，外部使用时只能使用该特征已声明的属性）(函数返回中的impl trait)[https://course.rs/basic/trait/trait.html#%E5%87%BD%E6%95%B0%E8%BF%94%E5%9B%9E%E4%B8%AD%E7%9A%84-impl-trait]
     * 使用 `impl trait` 形式将闭包标识为实现了某个特征的类型后就可以返回。在闭包的场景中，Fn、FnMut、FnOnce它们分别对应不同的闭包类型。通过返回impl Fn(参数类型) -> 返回值类型，这将告诉Rust编译器，返回一个实现了Fn trait的类型，但不指定具体是哪个类型。
     * 简而言之，impl关键词的使用是因为闭包的类型是匿名且不可直接命名的，而impl Trait语法允许我们以一种抽象的方式返回实现了特定trait的闭包，而无需关心闭包的具体类型。这大大增加了代码的灵活性和可重用性。
     *
     * 为什么闭包作为函数的参数时不需要显式的指定 impl ？
     * 当闭包作为参数传递给函数时，**不需要**使用impl关键字，是因为在这种情况下可以直接指定闭包参数遵循的特定trait（如Fn、FnMut或FnOnce）。
     * 这是通过使用**trait界定（trait bounds）**来实现的(简单理解为自动推断和实现)，它允许函数接受任何实现了指定trait的类型。这种方式提供了足够的灵活性，同时避免了impl Trait在参数位置的使用。
     *
     * ```rust
     * fn factory() -> impl Fn(i32) -> i32 {
     *     let num = 5;
     *     move |x| x + num
     * }
     *
     * let f = factory();
     * let answer = f(1);
     * assert_eq!(6, answer);
     * ```
     *
     * 用 `impl trait` 形式实现闭包作为返回值返回，最大的问题是 `impl trait` 要求返回只能有一个具体的类型。而闭包即使签名一致也可能是不同的类型。
     * ```rust
     * 编译错误 error
     * fn factory(x: i32) -> impl Fn(i32) -> i32 {
     *     let num = 5;
     *     if x > 1 {
     *         |x| x + num
     *      } else {
     *          |x| x - num
     *      }
     * }
     * ```
     *
     * 与impl trait相对应的，动态特征对象不限制某一个具体的类型，因此可改为使用动态特征对象解决这个问题。
     * ```rust
     * fn factory(x:i32) -> Box<dyn Fn(i32) -> i32> {
     *     let num = 5;
     *     if x > 1{
     *         Box::new(move |x| x + num)
     *     } else {
     *         Box::new(move |x| x - num)
     *     }
     * }
     * ```
     * ### 阅读
     * - [泛型、特征、特征对象](https://course.rs/basic/trait/intro.html)
     *
     *
     */

    let x = 2;
    let closure = || println!("{}", x);

    fn factory() -> impl Fn(i32) -> i32 {
        let x = 2;
        let closure = move |a| x + a;
        closure
    }

    let f = factory();
    println!("{}", f(1));

    // fn factory(x: i32) -> impl Fn(i32) -> i32 {
    //     let num = 5;
    //     if x > 1 {
    //         |x| x + num
    //     } else {
    //         |x| x - num
    //     }
    // }
}
