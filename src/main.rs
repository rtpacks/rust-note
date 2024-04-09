use ilearn::{run, Config};
use std::{array::IntoIter, env, error::Error, fmt::Display, fs, process};

fn main() {
    /*
     * ## 迭代器
     *
     * rust中，**迭代器的方法**可以细分为消费者适配器（consuming adaptors）和迭代器适配器（iterator adaptors），两者的区别在于是否消费迭代器，即是否调用迭代器的 next 方法。
     *
     * ### 消费者适配器
     * 消费者适配器（consuming adaptors）是迭代器上的方法，它会消费掉迭代器和迭代器中的元素，然后返回其类型的值，因此被称为消费。
     * 这些消费者（方法）都有一个共同的特点：在它们的定义中，都依赖 next 方法来消费元素。这也是为什么迭代器要实现 Iterator 特征时必须要实现 next 方法的原因。
     *
     * 只要迭代器上的某个方法 A 在其内部调用了 next 方法，那么 A 就可以被称为消费性适配器。这是因为 next 方法会消耗掉迭代器上的元素，所以方法 A 的调用也会消耗掉迭代器上的元素。
     *
     * 其中一个例子是 sum 方法，它会拿走迭代器的所有权，然后通过不断调用 next 方法对里面的元素进行求和：
     * ```rust
     * let v = vec![1, 2, 3];
     * let iter = v.iter();
     * let total: i32 = iter.sum(); // 消费者适配器需要标注数据类型
     * // println!("{:#?}", iter); 不能再访问iter，因为sum消费了迭代器和迭代器中的元素
     * println!("{total}");
     * ```
     *
     * 可以看到sum函数的定义 `fn sum(self) {}`，拿走了迭代器的所有权：
     * ```rust
     * fn sum<S>(self) -> S
     * where
     *     Self: Sized,
     *     S: Sum<Self::Item>,
     * {
     *     Sum::sum(self)
     * }
     * ```
     *
     * ### 迭代器适配器
     * 迭代器适配器（iterator adapters）即迭代器方法会返回一个新的迭代器，这是实现链式方法调用的关键：`v.iter().map().filter()...`。
     *
     */

    let v = vec![1, 2, 3];
    let mut iter = v.into_iter();
    let total: i32 = iter.sum();
    // println!("{:#?}", iter); 不能再访问iter，因为sum消费了迭代器和迭代器中的元素
    println!("{total}");

    let v = vec![1, 2, 3];
    let mut iter = v.into_iter();

    let total: i32 = iter.map(|x| x + 1).sum();
    println!("{total}");
}
