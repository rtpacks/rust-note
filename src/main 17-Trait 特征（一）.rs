fn main() {
    /*
     * ## Trait 和 Trait Object 特征
     *
     * Trait /treɪt/ 定义了一组可以被共享的行为，只要实现了特征，你就能使用这组行为
     *
     * ### Trait 和 interface
     *
     * 从多种数据类型中抽取出这些类型之间可通用的方法或属性，
     * 并将它们放进另一个相对更抽象的类型中，是一种很好的代码复用方式，也是多态的一种体现方式。
     *
     * 在面向对象语言中，这种功能一般通过接口(interface)实现。
     * 在Rust中，这种功能通过Trait实现。Trait类似于其他语言中接口的概念。
     * 例如，Trait可以被其他具体的类型实现(implement)，也可以在Trait中定义一些方法，实现该Trait的类型都必须实现这些方法。
     *
     * 严格来说，Rust中Trait的作用主要体现在两方面：
     * - Trait类型：用于定义抽象行为，抽取那些共性的属性，主要表现是作为泛型的数据类型(对泛型进行限制)，主要用在为某一个对象 composite（组合）该 Trait 的方法/属性。
     * - Trait对象：即Trait Object，能用于多态。主要用在表示Trait与Trait间继承关系。
     *
     * 组合是灵活且易读的，当继承链非常长时，难以判断属性是继承于谁。此外，rust 的 Trait 类似其他语言的interface，是指可以实现**默认方法**的 interface！
     *
     * 总之，Trait很重要，说是Rust的基石也不为过，它贯穿于整个Rust。
     */

    trait Playable {
        fn play(&self);
        // 可以实现默认方法
        fn pause(&self) {
            println!("pause");
        }
        fn get_duration(&self) -> f32;
    }

    struct Audio {
        name: String,
        duration: f32,
    }

    // 为 Audio 实现/组合 Playable Trait
    impl Playable for Audio {
        fn play(&self) {
            println!("playing");
        }
        fn get_duration(&self) -> f32 {
            self.duration
        }
    }

    // 组合是灵活且易读的，当继承链非常长时，难以判断属性是继承于谁
    println!(
        "{}",
        Audio {
            name: String::from("Hello"),
            duration: 12.0
        }
        .get_duration()
    );
}
