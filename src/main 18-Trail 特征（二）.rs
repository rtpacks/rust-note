use std::fmt::Display;
use std::io::Write;

fn main() {
    /*
     * ## Trait /treɪt/ 和 Trait Object 特征
     *
     * ### Trait的基本用法
     * ```rs
     * trait Playable {
     *     fn play(&self);
     *     // Trait 可以提供默认方法体的方法，即实现默认方法
     *     fn pause(&self) {
     *         println!("pause");
     *     }
     *     fn get_duration(&self) -> f32;
     * }
     * ```
     * 实现 Trait 时，Trait中的所有没有提供默认方法体的方法都需要实现。
     * 对于提供了默认方法体的方法，可实现可不实现，如果实现了则覆盖默认方法体，如果没有实现，则使用默认方法体。
     * trait 可以提供给任何一个需要该特征的对象使用，并实现（implement）/组合（composite）这个 trait 独特的属性
     *
     * **对象调用方法时，对于对象未实现的方法，会从其所实现的 Trait 中寻找。即查找顺序 对象 -> trait**
     *
     * ### 理解 Trait
     *
     * 某类型实现某Trait时，需要定义该Trait中指定的所有方法。
     * 定义之后，该类型也会拥有这些方法，似乎看上去和直接为各类型定义这些方法没什么区别。
     * 但是Trait是对多种类型之间的共性进行的抽象，
     * 它只规定实现它的类型要定义哪些方法以及这些方法的签名，不关心具体的**方法体的逻辑**。
     *
     * **Trait描述了一种通用功能，这种通用功能要求具有某些行为，
     * 这种通用功能可以被很多种类型实现，每个实现了这种通用功能的类型，都可以被称之为是【具有该功能的类型】。**
     *
     * 例如，Clone Trait是一种通用功能，描述可克隆的行为，i32类型、i64类型、Vec类型都实现了Clone Trait，
     * 那么就可以说i32类型、i64类型、Vec类型具有Clone的功能，可以调用clone()方法
     *
     * 甚至，数值类型(包括i32、u32、f32等等)的加减乘除功能，也都是通过实现各种对应的Trait而来的。
     * 比如，为了支持加法操作+，这些数值类型都实现了std::ops::Add这个Trait。
     * 可以这样理解，std::ops::Add Trait是一种通用功能，只要某个类型(包括自定义类型)实现了std::ops::Add这个Trait，
     * 这个类型的实例对象就可以使用加法操作。同理，对减法、除法、乘法、取模等等操作，也都如此。
     *
     * 一个类型可以实现很多种Trait，使得这个类型具有很多种功能，可以调用这些Trait的方法。
     * 比如，原始数据类型、Vec类型、HashMap类型等等已经定义好可直接使用的类型，
     * 都已经实现好了各种各样的Trait(具体实现了哪些Trait需查各自的文档)，可以调用这些Trait中的方法。
     *
     * 类型的大多数功能是组合(composite)其他各种Trait而来的(组合优于继承的组合)。
     * **因此，Rust是一门支持组合的语言：通过实现Trait而具备相应的功能，是组合而非继承。**
     *
     * 相比较继承，用Trait来组合实现类型的多种属性/功能是更灵活和易读的。
     *
     * ### derive Traits
     *
     * 常见的特征可以通过 `#[derive(trait)]` 形式让类型快速实现Trait（特征），
     * Rust会自动为Struct类型和Enum类型定义好这些Trait所要求实现的方法。
     *
     * 例如，为下面的Struct类型、Enum类型实现Copy Trait、Clone Trait。
     * > 复习Copy Clone https://rust-book.junmajinlong.com/ch6/06_ref_copy_clone.html
     * > 具有Copy Trait的一定具有Clone Trait，常见的基础数据类型、引用数据类型等都是可Copy的，也就是可Clone的。
     * > 使用引用数据类型会产生一个问题，引用类型可Copy是指Clone引用本身，而不是引用指向的实际数据，
     * > 如果需要复制引用指向的数据，需要给引用指向的数据类型实现Clone Trait，
     * > 这样由于Rust自动解引用的存在，会优先查找到真实数据的Clone Trait，并使用真实数据的clone方法，
     * > 而不是引用数据类型自己的Clone Trait。
     * > 优先级：引用指向的数据 -> 引用，如Person的引用，优先查找Person类型（Person）的方法，未找到再去查找Person引用（&Person）自身的方法
     * > ```rs
     * > struct Person {
     * >     age: u8,
     * > }
     * > let p = &Person { age: 12 };
     * > p.clone(); // 复制的是引用类型自己，因为Person类型没有实现Clone Trait
     * > ```
     * ```rs
     * #[derive(Copy, Clone)]
     * enum Direction {
     *   Up,
     *   Down,
     *   Left,
     *   Right,
     * }
     * ```
     *
     * ### trait作用域（孤儿规则）
     * 这部分有许多博客解释，我认为下面的解释是最清晰易懂的，我从其中取出正确的部分
     * - https://rustwiki.org/zh-CN/book/ch10-02-traits.html#%E4%B8%BA%E7%B1%BB%E5%9E%8B%E5%AE%9E%E7%8E%B0-trait
     * - https://course.rs/basic/trait/trait.html#%E7%89%B9%E5%BE%81%E5%AE%9A%E4%B9%89%E4%B8%8E%E5%AE%9E%E7%8E%B0%E7%9A%84%E4%BD%8D%E7%BD%AE%E5%AD%A4%E5%84%BF%E8%A7%84%E5%88%99
     * - https://rust-book.junmajinlong.com/ch11/02_more_about_trait.html#trait%E4%BD%9C%E7%94%A8%E5%9F%9F
     *
     *
     * 关于特征实现与定义的位置，有一条非常重要的原则（孤儿规则）：
     * 如果你想要为类型 A 实现 Trait T，那么 A 或者 T 至少有一个是在**当前作用域中定义**的！也就是不能为外部类型实现外部 trait。
     *
     * 在实现时Trait时有孤儿规则，在使用时，也需要注意以下说明，这是根本原因：
     * 即使类型 A 已经实现了 Trait T，如果想要通过类型 A 的实例对象来调用来自于 Trait T 的方法，要求 Trait T 必须在当前作用域内，否则报错。
     *
     * 解释上面两句话
     * 由于类型可以实现多个 Trait，而 Trait 可能存在相同的方法名，因此类型实体调用 Trait 方法时，必须明确方法来自哪个 Trait。
     * Rsut 采用就近原则来确定方法来自哪个 Trait：
     * 首先在当前作用域的Trait定义查找，如果没有找到，则在导入的 Trait 中查找。
     * 因此，在使用 Trait 方法时，当前作用域必须存在类型已经实现，并且包含该方法的 Trait。
     *
     * 简述：Rust通过类型调用来自Trait的方法，需要明确这个方法具体来自哪个Trait，所以就只有三种情况：
     * - 导入Trait，为定义在当前作用域的Strcut C使用 `implement A for C {}`
     * - 导入类型，为定义在当前作用域的Trait A使用 `implement A for C {}`
     * - 导入Trait和类型，使用 `implement A for C{}`
     *
     * Rust 规定了不能为外部类型实现外部 trait 的规则，也就是第三种情况不生效，可以确保其它人编写的代码不会破坏你的代码，也确保了你不会莫名其妙就破坏了风马牛不相及的代码。
     *
     * ### 总结
     * Rust是一门支持组合的语言：通过实现Trait而具备相应的功能，是组合而非继承。
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

    struct Video {
        name: String,
        duration: f32,
    }

    impl Playable for Video {
        fn play(&self) {
            println!("watching video: {}", self.name);
        }
        fn pause(&self) {
            println!("video paused");
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

    // 对于对象没有实现的方法，对象会从实现的trait中寻找该方法
    Audio {
        name: String::from("Hello"),
        duration: 12.0,
    }
    .pause();

    // 实现Copy Clone
    #[allow(unused)]
    struct Person {
        age: u8,
    }
    let p = &Person { age: 12 };
    p.clone();

    #[allow(unused)]
    #[derive(Clone, Copy)]
    enum Direction {
        Up,
        Down,
        Left,
        Right,
    }

    let mut buf: Vec<u8> = vec![];

    trait W {
        fn write_all(&self) {
            println!("write all")
        }
    }

    impl W for Vec<u8> {}

    buf.write_all(); // 报错：未找到write_all方法
}
