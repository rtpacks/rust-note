## 理解 Trait Object 和 vtable

大部分 Trait 教程把 Trait 和 Trait Object 的关系介绍的很复杂，其实从 Trait 的定义和类型的实现上去理解会比较简单。

举个例子，如果某个位置需要使用 Drivable 功能，那么它的要求就是类型需要实现了 Trait Drivable。
Car、Bus 等类型实现 Trait Drivable 后，实例对象能够调用来自 Trait Drivable 的方法。
因此该位置就可以直接使用实现了 Trait Drivable 的 Car、Bus 类型。
这些实例对象都可以充当 Trait Object。

用一句话来描述 Trait Object：**Trait Object 强调的是类型实现了 Trait，是具有这个 Trait 功能的对象，它不关注是哪一个类型**。

按照上面的说法，当 B、C、D 类型实现了 Trait A 后，就可以将类型 B、C、D 当作 Trait A 来使用。那么 trait 为什么不能直接生成实例呢？

**第一个原因：**
因为 Trait 是一种通用接口，事实上 Trait Object 与类似的 Slice<T> 才是真正的数据类型，并且还是 DST（动态尺寸类型）类型，即它的大小是不固定的。
这意味着 Trait Object 作为类型生成的实例的大小是不固定的，rust 编译器不允许直接使用大小不确定的数据类型。
因此，**Rust 中不能将 Trait 当作类型使用， Trait Object 类型的实例总是以引用的形式出现 &dyn**。
这也是 trait 跟许多语言中的 “interface” 的一个区别。

**第二个原因：**
Trait 在 [Trait | rust 官方文档](https://rustwiki.org/zh-CN/reference/items/traits.html) 的定义：trait 描述类型可以实现的抽象接口。
Trait 作为一种接口，本身是不生成实例的，而是通过其他类型实现接口，让其他实现它的类型的实体，以多态的形式转换为接口的实例。
因为 Trait 可以被多个类型实现，这意味着 Trait Object 类型可以来自多种类型。这种在运行时确定的多态也被称为“**动态分派**”。

```rs
trait A {}
trait B {}
struct C {}

impl A for C {}
impl B for C {}

let c: A = C {}; // 这是错误的，因为Trait是接口，Trait Object和Slice<T>才是真正的DST类型，Size不固定不能作为数据类型
```

Trait 与 Slice 的对比

- 对于类型 T，写法[T]表示类型 T 的 Slice 类型，由于 Slice 的大小不固定，因此几乎总是使用 Slice 的引用方式&[T]，Slice 的引用保存在栈中，包含两份数据：Slice 所指向数据的起始指针和 Slice 的长度。
- 对于 Trait A，写法 dyn A 表示 Trait A 的 Trait Object 类型，由于 Trait Object 的大小不固定。因此几乎总是使用 Trait Object 的引用方式&dyn A，Trait Object 的引用保存在栈中，包含两份数据：Trait Object 所指向数据的指针和指向一个虚表 vtable 的指针。

Trait Object，还有几点需要解释：

- Trait Object 大小不固定：这是因为，对于特征 T，类型 A 可以实现特征 T，类型 B 也可以实现特征 T，因此特征对象不确定来源也就没有固定大小
- 几乎总是使用 Trait Object 的引用方式，如 &dyn T、Box<dyn T>
- 虽然特征对象没有固定大小，但它的引用类型的大小是固定的，它由两个指针组成（ptr 和 vptr），因此占用两个指针大小
  - **一个指针 ptr 指向实现了特征 T 的具体类型的实例**，也就是当作特征 T 来用的类型的实例，比如类型 A 的实例、类型 B 的实例
  - \*\*另一个指针 vptr 指向一个虚表 vtable，vtable 中保存了类型 A 或类型 B 的实例对于可以调用的实现于特征 T 的方法。当调用方法时，直接从 vtable 中找到方法并调用。之所以要使用一个 vtable 来保存各实例的方法，是因为实现了特征 T 的类型有多种，这些类型拥有的方法各不相同，当将这些类型的实例都当作特征 T 来使用时(此时，它们全都看作是特征 T 类型的实例)，有必要区分这些实例各自有哪些方法可调用

简而言之，当类型 A 实现了 Trait T 时，类型 A 的实例对象 a 可以当作特征 T 的特征对象类型（Trait Object）来使用。
a 中保存了作为特征对象的数据指针（指向类型 A 的实例数据）和行为指针（指向 vtable）。

一定要注意，此时的 a 是 T 的特征对象的实例，而不再是具体类型 A 的实例，而且 a 的 vtable 只包含了实现自特征 T 的那些方法。
因此 a 只能调用实现于 Trait T 的方法，而不能调用类型 A 本身实现的方法和 A 实现于其他 Trait 的方法。
也就是说，a 是哪个特征对象的实例，它的 vtable 中就包含了该特征的方法。

为什么还需要 vtable？
因为 Trait Object 是一个数据类型，它的实例虽然是由其他的类型生成的，但是转成 Trait Object 后，实例丢失了信息。需要 vtable 记录实例可以调用 Trait T 中的哪些方法。

### 参考

- https://rust-book.junmajinlong.com/ch11/04_trait_object.html
- https://course.rs/basic/trait/trait-object.html
- https://zhuanlan.zhihu.com/p/23791817

### code

```rs
fn main {
    trait A {}

    trait B {}

    struct C {}

    impl A for C {}

    impl B for C {}

    // let c: A = C {}; // 错误的
}
```
