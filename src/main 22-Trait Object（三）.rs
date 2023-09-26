fn main() {
    /*
     * ## Trait Object 对象安全
     * 在 Rust 中，有两个self，一个指代当前的实例对象，一个指代特征或者方法类型的别名：
     *
     * ```rs
     * trait Draw {
     *     fn draw(&self) -> Self;
     * }
     *
     * #[derive(Clone)]
     * struct Button;
     * impl Draw for Button {
     *     fn draw(&self) -> Self {
     *         return self.clone()
     *     }
     * }
     *
     * fn main() {
     *     let button = Button;
     *     let newb = button.draw();
     * }
     * ```
     *
     * self 指代的就是当前的实例对象，也就是 button.draw() 中的 button 实例，Self 则指代的是 Button 类型。
     *
     * ### 特征对象 Trait Object 的限制
     *
     * 回顾Trait Object：Trait Object可以来自多种数据类型，它是一种DST（动态大小）数据类型，包括具体类型的实例对象和记录Trait功能的vtable，常以&dyn Trait的形式即胖指针（指向具体实例对象和指向vtable）出现。
     * 并且最重要的一点：将类型的实体转为Trait Object后，只能调用实现于Trait T的方法，而不能调用类型本身实现的方法和类型实现于其他Trait的方法。也就是说vtable只记录了当前Trait的方法。
     *
     * 虽然Trait Object是Trait的实例，但不是所有Trait都能拥有**特征对象**，只有对象安全的特征才行。当一个特征的所有方法都有如下属性时，它的对象才是安全的：
     * - 方法的返回类型不能是 Self
     * - 方法没有任何泛型参数
     *
     * 对象安全对于Trait Object是必须的，因为一旦类型实例转换为Trait Object，就不再知道实现该特征的具体类型是什么了。
     * 如果特征方法返回了 Self 类型，但是特征对象已经忘记了其真正的类型，那这个 Self 就非常尴尬，因为没人知道它是谁了。因此Trait Object不能与返回Self类型的方法共存。
     *
     * 对于泛型类型参数来说，当使用特征时其会放入具体的类型参数：此具体类型变成了实现该特征的类型的一部分。
     * 而当使用特征对象时其具体类型被抹去了，故而无从得知放入泛型参数类型到底是什么。因此Trait Object不能与泛型参数共存。
     * > 如果还未了解过泛型，可以先看
     * > - https://rust-book.junmajinlong.com/ch12/00.html
     * > - https://course.rs/basic/trait/generic.html
     */

    trait Draw {
        fn draw(&self) {
            println!("draw")
        }
    }

    struct Button;

    impl Draw for Button {
        fn draw(&self) {
            println!("Button is drawing");
        }
    }

    Button {}.draw()
}
