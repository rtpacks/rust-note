use core::fmt;
use ilearn::{run, Config};
use std::{
    fmt::{Debug, Display},
    ops::{Add, Index},
};

fn main() {
    /*
     * ## newtype 和类型别名 TypeAlias
     * 学习如何创建自定义类型，以及了解何为动态大小的类型
     *
     * ### newtype
     * > https://course.rs/basic/compound-type/struct.html#%E5%85%83%E7%BB%84%E7%BB%93%E6%9E%84%E4%BD%93tuple-struct
     *
     * 什么是 newtype？简单来说，就是使用**元组结构体**将已有的类型包裹起来，形成 `struct Meters(u32)` 的结构，此处 `Meters` 就是一个 newtype。
     *
     * newtype 的设计主要是为了增强类型安全并提供更明确的**语义区分**。这种设计允许开发者从现有的类型**派生出新的类型**，而这些新类型在逻辑上虽然与原始类型相似，但在类型系统中被视为完全不同的类型，这有助于避免类型间的错误混用。
     *
     * 例如 `struct Millimeters(u32)` 和 `struct Meters(u32)` 在逻辑形式上是与 u32 相同的，但是它们在类型系统是完全不一样的类型。即使两个 newtype 底层都是使用 u32，它们也不能互相替换，除非进行显式的类型转换。
     *
     * 从三个方面来解释：
     * - 自定义类型可以让我们给出更有意义和可读性的类型名，例如与其使用 u32 作为距离的单位类型，我们可以使用 Meters，它的可读性要好得多
     * - 对于某些场景，只有 newtype 可以很好地解决
     * - 隐藏内部类型的细节
     *
     * #### 为外部类型实现外部特征
     * > https://rustwiki.org/zh-CN/book/ch10-02-traits.html#%E4%B8%BA%E7%B1%BB%E5%9E%8B%E5%AE%9E%E7%8E%B0-trait
     * > 
     * > https://github.com/rtpacks/rust-note/blob/main/docs/unit%2018-Trait%20%E7%89%B9%E5%BE%81%EF%BC%88%E4%BA%8C%EF%BC%89.md#trait-%E4%BD%9C%E7%94%A8%E5%9F%9F%E5%AD%A4%E5%84%BF%E8%A7%84%E5%88%99
     *
     * 在为类型实现trait中，提到过一个孤儿原则：如果你想要为类型 A 实现 Trait T，那么 A 或者 T 至少有一个是在**当前作用域中定义**的！也就是不能为外部类型实现外部 trait。
     *
     * 这是因为由于类型可以实现多个 Trait，而不同的 Trait 可能存在相同的方法名，因此类型实体调用 Trait 方法时，必须明确方法来自哪个 Trait，所以需要孤儿原则来保证调用明确的方法。
     *
     * 例如，如果想使用 `println!("{}", v)` 的方式去格式化输出一个动态数组 `Vec`，以期给用户提供更加清晰可读的内容，那么就需要为 Vec 实现 Display 特征。
     * 但是这里有一个问题： `Vec` 类型定义在标准库中，`Display` 亦然，根据孤儿院则不能给外部类型实现外部特征，不能直接为 Vec 实现 Display 特征。
     *
     * 现在可以通过 newtype **定义新类型**来解决这个问题，定义一个元组结构体，通过 `.0` 访问原始类型数据：
     * ```rust
     * struct Wrapper(Vec<String>);
     *
     * impl fmt::Display for Wrapper {
     *     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
     *         write!(f, "[{}]", self.0.join(", ")) // 访问元组中的元素，即原始数据
     *     }
     * }
     *
     * let w = Wrapper(vec![String::from("hello"), String::from("world")]);
     * println!("w = {}", w);
     * ```
     *
     * 通过 newtype 形式定义新类型 `struct Wrapper(Vec<String>)` 后，就满足孤儿原则的当前作用域必须存在类型或特征要求。
     *
     * #### 更好的可读性及类型异化
     * **更好的可读性不等于更少的代码**，但可读性的提升降低维护代码的难度。例如 `struct Millimeters(u32)` 和 `struct Meters(u32)` 在逻辑形式上是与 u32 相同的，但是它们在类型系统是完全不一样的，两个类型不允许直接相加。
     *
     * 如果需要两个类型实现相加操作，约定返回Millimeters，可以为其实现Add特征：
     * ```rust
     * // newtype实现可读性的提升
     * struct Meters(u32);
     * struct Millimeters(u32);
     *
     * // 解除Add默认只能使用相同类型的限制
     * impl Add<Millimeters> for Meters {
     *     type Output = Millimeters;
     *     fn add(self, rhs: Millimeters) -> Millimeters {
     *         Millimeters(self.0 * 1000 + rhs.0)
     *     }
     * }
     *
     * impl fmt::Display for Millimeters {
     *     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
     *         write!(f, "{}mm", self.0)
     *     }
     * }
     *
     * let diff = Meters(3) + Millimeters(3000);
     *
     * println!("{}", diff); // 6000
     * ```
     *
     * #### 隐藏内部类型的细节
     * Rust 的类型有很多自定义的方法，假如把某个类型传给了用户，又不想用户调用类型方法，就可以使用 newtype：
     * ```rust
     * struct Meters(u32);
     * let i: u32 = 2;
     * assert_eq!(i.pow(2), 4); // u32 具有 pow 方法
     *
     * let n = Meters(i);
     * // assert_eq!(n.pow(2), 4); 错误，Meters(u32) 没有 pow 方法
     * ```
     *
     * 虽然 newtype 能够隐藏方法，但是用户可以通过 `n.0.pow(2)` 的方式来绕过限制，并调用内部类型的方法：
     * ```rust
     * assert_eq!(i.pow(2), 4);
     * ```
     *
     * ### 类型别名 TypeAlias
     * 使用 newtype 可以创建新类型，也可以使用一个更传统的方式，用类型别名来创建**新的类型名称**：
     * ```rust
     * // 使用TypeAlias创建新的类型名称，与原有类型相等
     * type MetersType = u32;
     * type MillimetersType = u32;
     * let diff1: MetersType = 3;
     * let diff2: MillimetersType = 3000;
     * println!("{}", diff1 * 1000 + diff2);
     * ```
     *
     * **类型别名并不是一个独立的全新的类型，而是某一个类型的别名**，因此编译器依然会把类型别名视为原有类型。
     *
     * 与 newtype 的区别：
     * - 类型别名只是别名，是为了让可读性更好，并不是全新的类型，而 newtype 是一个全新的类型
     * - 类型别名无法实现为外部类型实现外部特征等功能，因为类型别名还是等于原有类型，而 newtype 可以
     *
     * 类型别名除了让类型可读性更好，还能**减少模版代码的使用**，在一些引用交叉类型的代码上：
     * ```rust
     * let f: Box<dyn Fn() + Send + 'static> = Box::new(|| println!("hi"));
     *
     * fn takes_long_type(f: Box<dyn Fn() + Send + 'static>) {}
     * fn returns_long_type() -> Box<dyn Fn() + Send + 'static> {}
     * ```
     *
     * f 是一个令人眼花缭乱的类型 `Box<dyn Fn() + Send + 'static>`，使用时标注非常的麻烦，此时就可以用类型别名来解决：
     * ```rust
     * type Thunk = Box<dyn Fn() + Send + 'static>;
     *
     * let f: Thunk = Box::new(|| println!("hi"));
     * fn takes_long_type(f: Thunk) {}
     * fn returns_long_type() -> Thunk {}
     * ```
     *
     * 常用的 `std::io` 的 Result 也是经过类型别名简化的，它实际上是 `std::result::Result<T, std::io::Error>` 的别名
     * ```rust
     * type Result<T> = std::result::Result<T, std::io::Error>;
     * ```
     *
     * 如果为了区分 `std::io` 和 `std::fmt` 的 Result 和 Error类型，同时简化模板代码，那可以使用类型别名来简化代码中 `std::io` 和 `std::fmt` 两者的 Result 和 Error：
     * ```rust
     * type IOError = std::io::Error;
     * type FmtError = std::fmt::Error;
     * type IOResult<T> = std::result::Result<T, IOError>;
     * type FmtResult<T> = std::result::Result<T, FmtError>;
     * ```
     * **类型别名并不是一个独立的全新的类型，而是某一个类型的别名**，编译器依然会把类型别名视为原有类型，因此可以用类型别名来调用真实类型的所有方法。
     *
     * ### !永不返回类型
     * 在 TypeScript 中有一个 `never` 类型，表示永不返回类型，永不返回类型可能发生在**函数运行异常**和**程序死循环**这两点上。
     *
     * rust 用 `!` 表示永不返回类型，它除了在函数运行异常和程序死循环外，还能用在 match 匹配中。
     * 以下是一段错误代码，要赋值给 v，就必须保证 match 的各个分支返回的值是同一个类型，第一个分支返回数值、另一个分支返回元类型 ()，所以会出错。
     * ```rust
     * let i = 2;
     * let v = match i {
     *    0..=3 => i,
     *    _ => println!("不合规定的值:{}", i)
     * };
     * ```
     *
     * 可以用 `!` 永不返回类型解决这个问题，panic 的返回值是 !，代表它决不会返回任何值，既然没有任何返回值，那自然不会存在分支类型不匹配的情况。
     * ```rust
     * let i = 2;
     * let v = match i {
     *    0..=3 => i,
     *    _ => panic!("不合规定的值:{}", i)
     * };
     * ```
     *
     */

    //  newtype 使用元组结构体快速构建新类型，解决孤儿原则的限制
    struct Wrapper(Vec<String>);

    impl Display for Wrapper {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{}]", self.0.join(","))
        }
    }

    let w = Wrapper(vec![String::from("Hello"), String::from("World")]);
    println!("{w}");

    // newtype实现可读性的提升
    struct Meters(u32);
    struct Millimeters(u32);

    // 解除Add默认只能使用相同类型的限制
    impl Add<Millimeters> for Meters {
        type Output = Millimeters;
        fn add(self, rhs: Millimeters) -> Millimeters {
            Millimeters(self.0 * 1000 + rhs.0)
        }
    }

    impl fmt::Display for Millimeters {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}mm", self.0)
        }
    }

    let diff = Meters(3) + Millimeters(3000);

    println!("{}", diff); // 6000

    // 可以绕过类型限制，调用原有数据类型的方法
    Meters(2).0.pow(2);

    // 使用TypeAlias创建新的类型名称，与原有类型相等
    type MetersType = u32;
    type MillimetersType = u32;
    let diff1: MetersType = 3;
    let diff2: MillimetersType = 3000;
    println!("{}", diff1 * 1000 + diff2);

    // 类型别名提升可读性和减少冗长的类型模板代码
    // 类型模板代码 std::result::Result<T, std::io::Error>
    type IOError = std::io::Error;
    type FmtError = std::fmt::Error;
    type IOResult<T> = std::result::Result<T, IOError>;
    type FmtResult<T> = std::result::Result<T, FmtError>;

    // 永不返回的类型 !
    let i = 2;
    let x = 1..2;
    let x = 1..=2;
    let v = match i {
        0..=2 => i,
        _ => panic!("不符合规定的值 {i}"),
    };
}
