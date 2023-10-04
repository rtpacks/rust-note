use std::fmt::Debug;

fn main() {
    /*
     * ## 泛型的使用
     * 泛型的存在让抽象程度更高，因此在rust中泛型用处很多，如结构体、枚举、方法中都可以使用泛型。
     *
     * 注意：**使用的泛型都需要提前声明。**
     *
     * ### 结构体中使用泛型
     *
     * ```rs
     * // 声明泛型T，x，y是同一类型
     * struct Point<T> {
     *     x: T,
     *     y: T,
     * }
     *
     * fn main() {
     *     let integer = Point { x: 5, y: 10 };
     *     let float = Point { x: 1.0, y: 4.0 };
     * }
     * ```
     *
     * ### 枚举中使用泛型
     *
     * 提到枚举类型，Option 永远是第一个应该被想起来的，Option<T> 是一个拥有泛型 T 的枚举类型，它第一个成员是 Some(T)，存放了一个类型为 T 的值。
     * 得益于泛型的引入，我们可以在任何一个需要返回值的函数中，使用 Option<T> 枚举类型来做为返回值，用于返回一个任意类型的值 Some(T)，或者没有值 None。
     *
     * 另外，Result也是常见的枚举：如果函数正常运行，则最后返回一个 Ok(T)，T 是函数具体的返回值类型，如果函数异常运行，则返回一个 Err(E)，E 是错误类型。
     *
     * ```rs
     * enum Option<T> {
     *     Some(T),
     *     None,
     * }
     *
     * enum Result<T, E> {
     *     Ok(T),
     *     Err(E),
     * }
     * ```
     *
     * ### 方法中使用泛型
     *
     * 方法的定义一般是存在结构体中 `impl Struct { fn function() {} }`，泛型又是代表数据类型的变量（变量自身也可以用作类型），所以在方法中使用泛型一般是如下形式：
     *
     *
     * ```rs
     * struct Point<T> {
     *     x: T,
     *     y: T,
     * }
     *
     * impl<T> Point<T> {
     *     fn x(&self) -> &T {
     *         &self.x
     *     }
     * }
     *
     * fn main() {
     *     let p = Point { x: 5, y: 10 };
     *
     *     println!("p.x = {}", p.x());
     * }
     * ```
     *
     * 提前声明泛型 `T`，只有提前声明了，我们才能在Point<T>中使用它，这样 Rust 就知道 Point 的尖括号中的类型是泛型而不是具体类型。
     *
     * 需要注意的是，这里的 Point<T> 不再是泛型声明，而是一个完整的结构体类型，因为我们定义的结构体就是 Point<T> 而不再是 Point。
     * 即当我们声明了 `let p = Point{x: 5, y: 10}` 后，`p`的类型就不再是 `Point` 或 `Point<T>` 了，而是具体的 `Point<i32>` 类型。
     *
     * 此外，还可以定义多个泛型
     *
     * ```rs
     * struct Point<T, U> {
     *     x: T,
     *     y: U,
     * }
     *
     * impl<T, U> Point<T, U> {
     *     fn mixup<V, W>(self, other: Point<V, W>) -> Point<T, W> {
     *         Point {
     *             x: self.x,
     *             y: other.y,
     *         }
     *     }
     * }
     * ```
     *
     * ### 为具体的泛型类型实现方法
     *
     * 对于 Point<T> 类型，你不仅能定义基于 T 的方法，还能针对特定的具体类型进行方法定义，这些方法只在对应的数据类型生效。
     * 如以下方法指挥在数据类型为f64时生效，这样我们就**能针对特定的泛型类型实现某个特定的方法，对于其它泛型类型则没有定义该方法**。
     *
     * ```rs
     * struct Point<T> {
     *     x: T,
     *     y: T,
     * }
     *
     * impl Point<f64> {
     *     fn distance_from_origin(&self) -> f64 {
     *         (self.x.powi(2) + self.y.powi(2)).sqrt()
     *     }
     * }
     * ```
     *
     * ### 泛型的引用类型
     * 泛型的引用类型常出现在实现相同Trait但不同类型的数据类型上，如字符数组、i32数组、i64数组等。
     *
     * 如果参数是一个引用，且需要泛型，就可以使用泛型的引用 `&T或&mut T`，&T是不可变泛型引用，&mut T是可变泛型引用。
     *
     * 如打印不同类型的数组，实现也不难，唯一要注意的是需要对 T 加一个限制 std::fmt::Debug，该限制表明 T 可以用在 println!("{:?}", arr) 中，因为 {:?} 形式的格式化输出需要 arr 实现该特征。
     * ```rs
     * fn display_arr<T: std::fmt::Debug>(arr: &[T]) {
     *     println!("{:#?}", arr);
     * }
     *
     * let arr: [i32, 4] = [1, 2, 3, 4];
     * display_arr(&arr);
     *
     * let arr: [char, 4] = ['1', '2', '3', '4'];
     * display_arr(&arr);
     * ```
     *
     * ### const 泛型（Rust 1.51 版本引入的重要特性），字面量类型
     *
     * 通过引用可以很轻松的解决处理任何类型数组的问题，但是如果在某些场景下引用不适宜用或者干脆不能用呢？
     * 比如限制类型的某一个属性在某个范围或固定值：
     * - 任何数组 到 `限制长度小于4的任何类型数组`，长度的值小于 4，此时数组的引用不适用这种情况
     * - 任何年龄的Person 到 `age 大于等于 18 的 Person`，age 的值大于等于 18，此时Person的引用不能表达这种情况。
     *
     * ```rs
     * fn display_arr<T: std::fmt::Debug>(arr: &[T]) {
     *     println!("{:#?}", arr);
     * }
     * let a: [i32, 2] = [1, 2];
     * let b: [i32, 3] = [1, 2, 3];
     *
     *
     * struct Person {
     *      age: i32
     * }
     * fn display_p<T: std::fmt::Debug>(p: &Person) {
     *      println!("{:#?}", p);
     * }
     * let p = Person { age: 17 };
     * ```
     *
     * 当某些场景下引用不适宜用或者干脆不能用，这就需要 const 泛型，也就是**针对值的泛型**（用常量值而不是类型作为泛型的参数，即字面量类型），正好可以处理类似问题，它相当于增加了限制（缩小了泛型的范围），可以作为值直接使用。
     *
     * ```rs
     * fn display_arr<T: std::fmt::Debug, const N: usize>(arr: [T; N]) {
     *     println!("{:?}", arr);
     * }
     * let arr: [i32; 3] = [1, 2, 3];
     * display_arr(arr);
     *
     * let arr: [i32; 2] = [1, 2];
     * display_arr(arr);
     * ```
     *
     * 在调用 `display_arr` 时，可传入 `N` 的**实参**为
     * - 一个单独的 const 泛型参数，如 `M`，这种方式通常是由 `祖` 级传入限制
     * - 一个字面量 (i.e. 整数，布尔值或字符)，如 `2` 表示固定值
     * - 一个具体的 const 表达式（双大括号）， 并且表达式中泛型参数不参与任何计算，如 `{ 1 + 1 }` 表示动态计算
     *
     * ```rs
     * display_arr::<i32, M>(); // ok: 符合第一种，但注意需要传递M泛型。
     * display_arr::<i32, 2021>(); // ok: 符合第二种
     * display_arr::<i32, {20 * 100 + 20 * 10 + 1}>(); // ok: 符合第三种
     *
     * display_arr::<i32, { M + 1 }>(); // error: 违背第三种，表达式中泛型参数不参与任何计算
     * display_arr::<i32, { std::mem::size_of::<T>() }>(); // error: 违背第三种，表达式中泛型参数不参与任何计算
     * ```
     *
     * 除函数可以使用const泛型参数外，变量类型也可以使用const泛型参数。
     *
     * 更多const的使用，可以查看：
     * - https://rustcc.cn/article?id=d1d98ea9-8460-416d-9280-e22dc8d47b6b
     * - https://learnku.com/docs/practice/const-fan-xing/13837
     * - https://course.rs/basic/trait/generic.html#const-%E6%B3%9B%E5%9E%8Brust-151-%E7%89%88%E6%9C%AC%E5%BC%95%E5%85%A5%E7%9A%84%E9%87%8D%E8%A6%81%E7%89%B9%E6%80%A7
     */

    struct Point<T> {
        x: T,
        y: T,
    }
    impl<T> Point<T> {
        fn x(&self) -> &T {
            &self.x
        }
    }
    impl Point<f64> {
        fn f64(&self) -> &f64 {
            &self.x
        }
    }

    let p = Point { x: 32, y: 32 };
    println!("{}, {}", p.x, p.x());
    let p = Point { x: 12.0, y: 12.0 };
    println!("{:?}, {:?}, {:?}", p.x, p.x(), p.f64());
    fn display_arr1<T: std::fmt::Debug>(arr: &[T]) {
        println!("{:#?}", arr);
    }
    let arr = [1, 2, 3, 4];
    display_arr1(&arr);
    let arr = ['1', '2', '3', '4'];
    display_arr1(&arr);

    // const 泛型参数
    fn display_arr2<T: std::fmt::Debug, const N: usize>(arr: [T; N]) {
        println!("{:#?}", arr);
        println!("{:#?}", N + 1);
    }
    fn display_arr3<T: std::fmt::Debug, const N: usize>(arr: [T; N]) {
        println!("{:#?}", arr);

        let _arr = [1, 2, 3, 4];

        // display_arr2::<T, N>(_arr);
    }
    let arr: [i32; 3] = [1, 2, 3];
    display_arr2::<i32, { 1 + 2 }>(arr);
    let arr: [i32; 2] = [1, 2];
    const k: usize = 2;
    display_arr3::<i32, { k }>(arr);
}
