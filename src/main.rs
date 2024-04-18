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
     * > [Trait 特征](/docs/unit 18-Trait 特征（二）.md)
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

    Meters(2).0.pow(2);
}
