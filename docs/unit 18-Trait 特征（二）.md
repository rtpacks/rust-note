## Trait /treɪt/ 和 Trait Object 特征

### Trait 的基本用法

```rs
trait Playable {
    fn play(&self);
    // Trait 可以提供默认方法体的方法，即实现默认方法
    fn pause(&self) {
        println!("pause");
    }
    fn get_duration(&self) -> f32;
}
```

实现 Trait 时，Trait 中的所有没有提供默认方法体的方法都需要实现。
对于提供了默认方法体的方法，可实现可不实现，如果实现了则覆盖默认方法体，如果没有实现，则使用默认方法体。
trait 可以提供给任何一个需要该特征的对象使用，并实现（implement）/组合（composite）这个 trait 独特的属性

**对象调用方法时，对于对象未实现的方法，会从其所实现的 Trait 中寻找。即查找顺序 对象 -> trait**

### 理解 Trait

某类型实现某 Trait 时，需要定义该 Trait 中指定的所有方法。
定义之后，该类型也会拥有这些方法，似乎看上去和直接为各类型定义这些方法没什么区别。
但是 Trait 是对多种类型之间的共性进行的抽象，
它只规定实现它的类型要定义哪些方法以及这些方法的签名，不关心具体的**方法体的逻辑**。

**Trait 描述了一种通用功能，这种通用功能要求具有某些行为，
这种通用功能可以被很多种类型实现，每个实现了这种通用功能的类型，都可以被称之为是【具有该功能的类型】。**

例如，Clone Trait 是一种通用功能，描述可克隆的行为，i32 类型、i64 类型、Vec 类型都实现了 Clone Trait，
那么就可以说 i32 类型、i64 类型、Vec 类型具有 Clone 的功能，可以调用 clone()方法

甚至，数值类型(包括 i32、u32、f32 等等)的加减乘除功能，也都是通过实现各种对应的 Trait 而来的。
比如，为了支持加法操作+，这些数值类型都实现了 std::ops::Add 这个 Trait。
可以这样理解，std::ops::Add Trait 是一种通用功能，只要某个类型(包括自定义类型)实现了 std::ops::Add 这个 Trait，
这个类型的实例对象就可以使用加法操作。同理，对减法、除法、乘法、取模等等操作，也都如此。

一个类型可以实现很多种 Trait，使得这个类型具有很多种功能，可以调用这些 Trait 的方法。
比如，原始数据类型、Vec 类型、HashMap 类型等等已经定义好可直接使用的类型，
都已经实现好了各种各样的 Trait(具体实现了哪些 Trait 需查各自的文档)，可以调用这些 Trait 中的方法。

类型的大多数功能是组合(composite)其他各种 Trait 而来的(组合优于继承的组合)。
**因此，Rust 是一门支持组合的语言：通过实现 Trait 而具备相应的功能，是组合而非继承。**

相比较继承，用 Trait 来组合实现类型的多种属性/功能是更灵活和易读的。

### derive Traits

常见的特征可以通过 `#[derive(trait)]` 形式让类型快速实现 Trait（特征），
Rust 会自动为 Struct 类型和 Enum 类型定义好这些 Trait 所要求实现的方法。

例如，为下面的 Struct 类型、Enum 类型实现 Copy Trait、Clone Trait。

> 复习 Copy Clone https://rust-book.junmajinlong.com/ch6/06_ref_copy_clone.html
> 具有 Copy Trait 的一定具有 Clone Trait，常见的基础数据类型、引用数据类型等都是可 Copy 的，也就是可 Clone 的。
> 使用引用数据类型会产生一个问题，引用类型可 Copy 是指 Clone 引用本身，而不是引用指向的实际数据，
> 如果需要复制引用指向的数据，需要给引用指向的数据类型实现 Clone Trait，
> 这样由于 Rust 自动解引用的存在，会优先查找到真实数据的 Clone Trait，并使用真实数据的 clone 方法，
> 而不是引用数据类型自己的 Clone Trait。
> 优先级：引用指向的数据 -> 引用，如 Person 的引用，优先查找 Person 类型（Person）的方法，未找到再去查找 Person 引用（&Person）自身的方法
>
> ```rs
> struct Person {
>     age: u8,
> }
> let p = &Person { age: 12 };
> p.clone(); // 复制的是引用类型自己，因为Person类型没有实现Clone Trait
> ```

```rs
#[derive(Copy, Clone)]
enum Direction {
  Up,
  Down,
  Left,
  Right,
}
```

### trait 作用域（孤儿规则）

这部分有许多博客解释，我认为下面的解释是最清晰易懂的，我从其中取出正确的部分

- https://rustwiki.org/zh-CN/book/ch10-02-traits.html#%E4%B8%BA%E7%B1%BB%E5%9E%8B%E5%AE%9E%E7%8E%B0-trait
- https://course.rs/basic/trait/traiml#%E7%89%B9%E5%BE%81%E5%AE%9A%E4%B9%89%E4%B8%8E%E5%AE%9E%E7%8E%B0%E7%9A%84%E4%BD%8D%E7%BD%AE%E5%AD%A4%E5%84%BF%E8%A7%84%E5%88%99
- https://rust-book.junmajinlong.com/ch11/02_more_about_trait.html#trait%E4%BD%9C%E7%94%A8%E5%9F%9F

关于特征实现与定义的位置，有一条非常重要的原则（孤儿规则）：
如果你想要为类型 A 实现 Trait T，那么 A 或者 T 至少有一个是在**当前作用域中定义**的！也就是不能为外部类型实现外部 trait。

在实现时 Trait 时有孤儿规则，在使用时，也需要注意以下说明，这是根本原因：
即使类型 A 已经实现了 Trait T，如果想要通过类型 A 的实例对象来调用来自于 Trait T 的方法，要求 Trait T 必须在当前作用域内，否则报错。

解释上面两句话
由于类型可以实现多个 Trait，而 Trait 可能存在相同的方法名，因此类型实体调用 Trait 方法时，必须明确方法来自哪个 Trait。
Rsut 采用就近原则来确定方法来自哪个 Trait：
首先在当前作用域中查找，如果没有找到，则在导入的 Trait 中查找。如果仍未找到，则报错。
因此，在使用 Trait 方法时，当前作用域必须存在类型已经实现，并且包含该方法的 Trait。
此外，Rust 还规定了不能为外部类型实现外部 trait 的规则。

### 总结

Rust 是一门支持组合的语言：通过实现 Trait 而具备相应的功能，是组合而非继承。
