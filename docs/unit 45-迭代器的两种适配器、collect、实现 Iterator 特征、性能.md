## 迭代器

rust 中，**迭代器的方法**可以细分为消费者适配器（consuming adaptors）和迭代器适配器（iterator adaptors），两者的区别在于是否消费迭代器，即是否调用迭代器的 next 方法。

### 消费者适配器

消费者适配器（consuming adaptors）是迭代器上的方法，它会消费掉迭代器和迭代器中的元素，然后返回其类型的值，因此被称为消费。
这些消费者（方法）都有一个共同的特点：在它们的定义中，都依赖 next 方法来消费元素。这也是为什么迭代器要实现 Iterator 特征时必须要实现 next 方法的原因。

只要迭代器上的某个方法 A 在其内部调用了 next 方法，那么 A 就可以被称为消费性适配器。这是因为 next 方法会消耗掉迭代器上的元素，所以方法 A 的调用也会消耗掉迭代器上的元素。

其中一个例子是 sum 方法，它会拿走迭代器的所有权，然后通过不断调用 next 方法对里面的元素进行求和：

```rust
let v = vec![1, 2, 3];
let iter = v.iter();
let total: i32 = iter.sum(); // 消费者适配器需要标注数据类型
// println!("{:#?}", iter); 不能再访问iter，因为sum消费了迭代器和迭代器中的元素
println!("{total}");
```

可以看到 sum 函数的定义 `fn sum(self) {}`，拿走了迭代器的所有权：

```rust
fn sum<S>(self) -> S
where
    Self: Sized,
    S: Sum<Self::Item>,
{
    Sum::sum(self)
}
```

### 迭代器适配器

迭代器适配器（iterator adapters）即迭代器方法会返回一个新的迭代器，这是实现链式方法调用的关键：`v.iter().map().filter()...`。
与消费者适配器不同，迭代器适配器是惰性的，意味着需要一个**消费者适配器来收尾**，最终将迭代器转换成一个具体的值：

```rust
let v: Vec<i32> = vec![1, 2, 3];
// v.iter().map(|x| x + 1); 仅有迭代器适配器是不行的，需要消费者适配器收尾
let newV: Vec<_> = v.iter().map(|x| x + 1).collect(); // 正常
```

> 为什么要区分消费者适配器和迭代器适配器两种方法呢？
>
> Rust 语言在设计上非常注重内存安全和效率，这种设计哲学体现在它对迭代器模式的处理上。Rust 区分消费性适配器（consuming adaptors）和迭代器适配器（iterator adaptors）主要是为了提供更细粒度的控制以及更明确的语义。
>
> 消费性适配器（Consuming Adaptors）
>
> 消费性适配器是那些会消耗迭代器的方法，它们会遍历迭代器并返回一个最终的结果。这意味着一旦调用了消费性适配器，原来的迭代器就不能再使用了。在 Rust 中，collect()就是一个消费性适配器的例子，它可以将迭代器中的元素收集到一个集合类型中，比如 Vec、HashMap 等。
>
> 迭代器适配器（Iterator Adaptors）
>
> 迭代器适配器则是对迭代器进行转换，但不会立即进行任何遍历操作。它们返回的是一个新的迭代器，这个新迭代器会在每次遍历时应用某种操作。在 Rust 中，map()就是一个迭代器适配器的例子，它会创建一个新的迭代器，这个迭代器会在每次访问时应用一个函数到原迭代器的每个元素上。
>
> 1. 性能优化：Rust 的迭代器设计允许编译器在编译时进行更多的优化，比如通过迭代器链的懒惰求值来减少中间集合的创建，这可以显著提高程序的性能。
> 2. 内存管理：Rust 通过所有权系统来保证内存安全，区分消费性适配器和迭代器适配器有助于明确所有权和借用的规则，避免悬垂指针和数据竞争等问题。
> 3. 明确的语义：在 Rust 中，当你使用 collect()时，你明确地表达了你想要从迭代器中消费所有元素并生成一个集合。这种明确性有助于代码的可读性和维护性。

### collect 方法

在上面的案例中使用了一个非常强大的 collect 方法，该方法就是一个消费者适配器，它可以将一个迭代器中的元素**收集到指定类型中**。

如为收集变量标注 `Vec<_>` 类型，是为了告诉 collect：把迭代器中的元素消费掉，然后把值收集成 `Vec<_>` 类型，至于为何使用 \_，因为编译器会帮我们自动推导。

collect 在消费时必须显式的指定想要收集成的集合类型，是因为该方法可以收集成多种不同的集合类型，如 Vec 和 HashMap。

```rust
use std::collections::HashMap;

// 收集成Vec

let v = vec![1, 2, 3];
let newV: Vec<_> = v.iter().map(|x| x + 1).collect();
println!("{:?}", newV);

// 收集成HashMap
let names = ["sunface", "sunfei"];
let ages = [18, 18];
let map: HashMap<_, _> = names.into_iter().zip(ages.into_iter()).collect();
println!("{:?}", map);
```

zip 是一个迭代器适配器，它的作用就是将两个迭代器的内容压缩到一起，形成 `Iterator<Item=(ValueFromA, ValueFromB)>` 这样的新的迭代器，在此处就是形如 `[(name1, age1), (name2, age2)]` 的迭代器，可以类比 JavaScript 中的 Entries 类型。

然后再通过 collect 将新迭代器中 `(K, V)` 形式的值收集成 `HashMap<K, V>`，同样的，这里必须显式声明类型，然后 HashMap 内部的 KV 类型可以交给编译器去推导，最终编译器会推导出 `HashMap<&str, i32>`。

### 闭包作为适配器参数

之前的 map 方法中，使用闭包来作为迭代器适配器的参数，它最大的好处不仅在于可以就地实现迭代器中元素的处理，还在于可以**捕获环境值**

```rust
let mut index = 0;
let v: Vec<_> = vec![0; 10]
    .into_iter()
    .map(|x| {
        index += 1;
        index
    })
    .collect();

let level = 8;
let standards: Vec<_> = v.into_iter().filter(|x| *x >= level).collect(); // 捕获环境变量
println!("{:?}", standards)
```

filter 是迭代器适配器，用于对迭代器中的每个值进行审计，符合条件则保留，反之则剔除。最后通过 collect 收集为 `Vec<i32>` 类型。

### 实现 Iterator 特征

Iterator 特征不仅仅局限于 `vec` 等内置数据类型，还可以为自定义类型实现 Iterator 特征（要求实现 next 方法），使自定义类型变为迭代器。注意，是将自定义类型变成迭代器（Iterator），而不是可迭代对象（IntoIterator）。

创建一个 counter struct：

```rust
struct Counter { count: i32 }
impl Counter {
    fn new() -> Self {
        Counter { count: 0 }
    }
}
```

为 Counter 实现 Iterator 特征：

```rust
impl Iterator for Counter {
     type Item: i32;
     fn next(&mut self) -> Option<Self::Item> {
         if self.count < 5 {
             self.count += 1;
             Some(self.count)
         } else {
             None
         }
     }
}
```

测试 Counter 迭代器：

```rust
let total: i32 = Counter::new()
    .zip(Counter::new().skip(1))
    .map(|(x, y)| x + y)
    .sum();
println!("{:?}", total);

let vector: Vec<_> = Counter::new().zip(Counter::new().skip(1)).filter(|x| *x > 0).collect();
println!("{:?}", vector);

let vector: Vec<_> = Counter::new().skip(3).collect();
println!("{:?}", vector);

// turbo fish 语法
let vector = Counter::new().skip(3).collect::<Vec<i32>>()();
println!("{:?}", vector);

let total = Counter::new().skip(3).sum::<i32>();
println!("{:?}", total);
```

- skip 是一个迭代器适配器，它的作用是跳过迭代器中的前 n 个元素，然后返回一个新的迭代器。
- zip 是一个迭代器适配器，它的作用就是将两个迭代器的内容压缩到一起，形成 `Iterator<Item=(ValueFromA, ValueFromB)>` 也就是形如 `[(name1, age1), (name2, age2)]` 的迭代器，可以类比 JavaScript 中的 Entries 类型。两者迭代器长度不一样时，以最短长度为结束条件。

> turbo fish 语法参考
> turbofish 语法可以允许不在变量上标注类型，在调用函数时传递目标类型以完成类型指定。
>
> - https://www.cnblogs.com/rotk2022/p/16449651.html
> - https://rustwiki.org/zh-CN/edition-guide/rust-2018/trait-system/impl-trait-for-returning-complex-types-with-ease.html#%E5%8F%82%E6%95%B0%E4%BD%8D%E7%BD%AE
> - https://rust-lang.github.io/impl-trait-initiative/explainer/apit_turbofish.html

#### 实现 Iterator 特征的其它方法

**其他迭代器方法都具有基于 next 方法的默认实现**，所以无需像 next 这样手动去实现。如上面案例使用到的 `zip, map, filter, sum` 等方法。

##### enumerate

enumerate 方法（迭代器适配器）是常用的迭代器方法，它能生成带有索引的迭代器，返回的结构为 `Iterator<Item=(ValueA, ValueB)>` 的迭代器，即 `(index, value)` 索引在前，值在后的结构。

```rust
let v = vec![1,2,3];
let index: Vec<_> = v.into_iter().enumerate().collect();
println!("{:?}", index);
```

### 性能

迭代器是 Rust 的 零成本抽象（zero-cost abstractions）之一，意味着抽象并不会引入运行时开销，这与 Bjarne Stroustrup（C++ 的设计和实现者）在 Foundations of C++（2012） 中所定义的 零开销（zero-overhead）如出一辙。

> In general, C++ implementations obey the zero-overhead principle: What you don’t use, you don’t pay for. And further: What you do use, you couldn’t hand code any better.
> 一般来说，C++的实现遵循零开销原则：没有使用时，你不必为其买单。 更进一步说，需要使用时，你也无法写出更优的代码了。
>
> 阅读：https://course.rs/advance/functional-programing/iterator.html#%E8%BF%AD%E4%BB%A3%E5%99%A8%E7%9A%84%E6%80%A7%E8%83%BD

### 更多迭代器方法

阅读：https://course.rs/std/iterator
