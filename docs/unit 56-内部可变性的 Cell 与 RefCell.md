## 内部可变性的 Cell 与 RefCell

Rust 通过严格的规则来保证所有权和借用的正确性，这带来安全提升的同时，损失了灵活性，比如结构体可变必须要求结构体所有字段可变。

这是由于 Rust 的 mutable 特性，一个结构体中的字段，要么全都是 immutable，要么全部是 mutable，**不支持针对部分字段进行设置**。
比如，在一个 struct 中，可能只有个别的字段需要修改，其他字段并不需要修改，为了一个字段而将整个 struct 变为 `&mut` 是不合理的。

rust 提供实现了**内部可变性** Cell 和 RefCell 解决这类问题，通过**内部可变性**可以实现 struct 部分字段可变，而不用将整个 struct 设置为 mutable。

> 内部可变性的实现是因为 Rust 使用了 unsafe 来做到这一点，但是对于使用者来说，这些都是透明的，因为这些不安全代码都被封装到了安全的 API 中。
> 简而言之，**可以在拥有不可变引用的同时修改目标数据**。

### Cell

Cell 和 RefCell 在功能上没有区别，区别在于 `Cell<T>` 适用于 T 实现 Copy 特征的情况：

```rust
//  use std::cell::Cell;
let s_cell = Cell::new("Hello World");
let s = s_cell.get(); // 获取内部数据
s_cell.set("Hi"); // 不可变引用直接修改内部数据
println!("{s_cell:?}, {s}");
```

以上代码展示了 Cell 的基本用法，有几点值得注意：

- "Hello World" 是 `&str` 类型，它实现了 Copy 特征
- get 用来取值，set 用来设置新值

取到值保存在 s 变量后，还能同时进行修改，这个违背了 Rust 的借用规则，但是由于实现了内部可变性的结构体 Cell 的存在，可以优雅地做到用不可变引用修改目标数据。

Cell 适用于实现 Copy 的类型，如果尝试在 Cell 中存放 String，编译器会立刻报错，这是因为 `String` 没有实现 Copy 特征：

```rust
let c = Cell::new(String::from("asdf")); 错误，String没有实现Copy特征
```

如果是自定义的结构体实现，会发现 safe 代码中不能实现在拥有不可变引用的情况下修改数据。因为这与方法接收者的类型不一致，不可变引用不能调用可变引用的方法（点操作符的隐式转换）：

```rust
struct MyCell<T: Copy> {
    value: T,
}
impl<T: Copy> MyCell<T> {
    fn new(v: T) -> MyCell<T> {
        MyCell { value: v }
    }
    fn set(&mut self, v: T) {
        self.value = v;
    }
}
let my_cell = MyCell::new("Hello World");
my_cell.set("Hi"); 错误，set函数 `set(&mut self, v: T)` 要求接收者是可变引用 `self: &mut Self`，而此时的 `my_cell` 是一个不可变引用。
```

#### 简单总结

Cell 通过内部的 `get set` 方法完成数据的获取和替换，即 `get` 提供不可变引用功能（读），`get set` 提供可变引用（读写）

### RefCell

在实际开发中，程序操作的更多是一个复杂数据类型，如多字段深层结构体。Cell 适用于 实现了 Copy 特征的类型，显然当复杂类型没有实现 Copy 时就需要另外一个内部可变性的工具来代替 Cell。
rust 针对复杂数据类型（未实现 Copy）提供实现了内部可变性的 `RefCell`。

**RefCell 的功能是通过 unsafe 操作，为一个类型（变量/值）对外提供该类型的不可变引用和可变引用，无论这个类型（变量/值）是否可变**。由于是 unsafe 的实现，不受借用规则限制。

对外暴露的不可变引用和可变引用操作是**有限制**的，必须要符合借用规则。
RefCell 关注点在为一个无论是否可变的类型（变量/值），**对外提供该类型的不可变引用和可变引用**，这里是 unsafe 的实现，不受借用规则限制。
接收不可变引用和可变引用的变量不属于 RefCell 的关注点，它们依然要符合借用规则，以保证 RefCell 智能指针的正常运行。
RefCell 会在内部记录不可变引用（borrow 方法）和可变引用（borrow_mut 方法）的使用次数，通过**使用次数来判断此时是否符合借用规则**。

```rust
// **RefCell 的功能是通过 unsafe 操作，为一个类型（变量/值）对外提供该类型的不可变引用和可变引用，无论这个类型（变量/值）是否可变**。
// RefCell 会在内部记录不可变引用（borrow）和可变引用（borrow_mut）的使用次数，通过使用次数来判断此时是否符合借用规则
let s = RefCell::new(String::from("Hello World"));
let s1 = s.borrow(); // RefCell 记录一次不可变引用，不可变引用是1，可变引用是0，符合借用规则，正常运行
let s1 = s.borrow(); // RefCell 记录一次不可变引用，不可变引用是2，可变引用是0，符合借用规则，正常运行
// let s2 = s.borrow_mut(); // RefCell 记录一次可变引用，不可变引用是2，可变引用是1，此时会报错，因为不能同时存在不可变引用和可变引用

let s = RefCell::new(String::from("Hello World"));
let s1 = s.borrow_mut(); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行
// let s2 = s.borrow_mut(); // RefCell 记录一次可变引用，不可变引用是0，可变引用是2，此时会报错，因为不能同时存在多个可变引用（一个可变引用周期内存在另外一个可变引用）
println!("{s1}");

let s = RefCell::new(String::from("Hello World"));
*s.borrow_mut() = String::from("Hi"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
*s.borrow_mut() = String::from("Hello"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
println!("{s}");
```

也就是 RefCell 实际上**没有解决可变引用和引用可以共存的问题**。
它的关注点在于为一个无论是否可变的类型（变量/值），**对外提供该类型的不可变引用和可变引用**，这里是 **unsafe** 的实现，不受借用规则限制。
所以 RefCell 只是绕过了编译期的错误，将报错从编译期推迟到运行时，从编译器错误变成了 panic 异常。

#### 为什么需要 RefCell？

既然没有解决问题，为什么还需要 RefCell？这是因为复杂类型的不可变与可变性。
由于 Rust 的 mutable 特性，一个结构体中的字段，要么全都是 immutable，要么全部是 mutable，**不支持针对部分字段进行设置**。
比如，在一个 struct 中，可能只有个别的字段需要修改，其他字段并不需要修改，为了一个字段而将整个 struct 变为 `&mut` 是不合理的。

而 RefCell 通过 unsafe 操作，可以为一个无论是否可变的类型（变量/值），**对外提供该类型的不可变引用和可变引用**，只需要接收的变量遵守借用规则就不会出现运行时错误。

这意味着可以**通过 RefCell 让一个结构体既有不可变字段，也有可变字段**，例如：

```rust
// 通过 RefCell，让一个结构体既有不可变字段，也有可变字段
#[derive(Debug)]
struct Person {
    name: RefCell<String>,
    age: i32,
}
let p = Person {
    name: RefCell::new(String::from("L")),
    age: 18,
};
// p.age = 22; 错误的，如果需要age可更改，需要p是可变的。
*p.name.borrow_mut() = String::from("M"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
*p.name.borrow_mut() = String::from("N"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
println!("{p:?}");
```

对于大型的复杂程序，可以选择使用 RefCell 来让事情简化。例如在 Rust 编译器的 ctxt 结构体中有大量的 RefCell 类型的 map 字段，主要的原因是：这些 map 会被分散在各个地方的代码片段所广泛使用或修改。由于这种分散在各处的使用方式，导致了管理可变和不可变成为一件非常复杂的任务（甚至不可能），你很容易就碰到编译器抛出来的各种错误。而且 RefCell 的运行时错误在这种情况下也变得非常有用：一旦有人做了不正确的使用，代码会 panic，然后告诉我们哪些借用冲突了。

总之，当有一个复杂类型，既有可变又有不可变，又或者需要被到处使用和修改然后导致借用关系难以管理时，都可以优先考虑使用 RefCell。

#### RefCell 总结

- RefCell 适用 Copy 和非 Copy 类型，一般来说 Copy 类型可直接选择 Cell
- RefCell 只是绕过编译期的借用规则，程序运行期没有绕过
- RefCell 适用于编译期误报或者一个引用被在多处代码使用、修改以至于难于管理借用关系时
- 使用 RefCell 时，`borrow` 和 `borrow_mut` 提供不可变引用和可变引用不能违背借用规则，否则会导致运行期的 panic
- RefCell 通过 unsafe 操作，可以为一个无论是否可变的类型（变量/值），**对外提供该类型的不可变引用和可变引用**，由于是 unsafe 操作，编译时期 `borrow(不可变借用)` 和 `borrow_mut(可变借用)` 方法内部实现不受借用规则的限制，所以编译不会报错。但是两个方法的接收者变量不是 unsafe 操作，接收者会受到借用规则的限制，RefCell 智能指针在运行时会记录不可变借用和可变借用的次数，如果方法接收者变量不符合借用规则，则会 panic。

### 选择 Cell 还是 RefCell

- RefCell 适用 Copy 和非 Copy 类型，一般来说 Copy 类型可直接选择 Cell
- Cell 通过内部的 `get set` 方法完成数据的获取和替换，即 `get` 提供不可变引用功能（读），`get set` 提供可变引用（读写）
- RefCell 通过 unsafe 操作，可以为一个无论是否可变的类型（变量/值），**对外提供该类型的不可变引用和可变引用**，由于是 unsafe 操作，编译时期 `borrow(不可变借用)` 和 `borrow_mut(可变借用)` 方法内部实现不受借用规则的限制，所以编译不会报错。但是两个方法的接收者变量不是 unsafe 操作，接收者会受到借用规则的限制，RefCell 智能指针在运行时会记录不可变借用和可变借用的次数，如果方法接收者变量不符合借用规则，则会 panic。
- Cell 没有额外的性能损耗，RefCell 有一点运行期开销，原因是它包含了一个字节大小的“借用状态”指示器，该指示器在每次运行时借用时都会被修改，进而产生一点开销。

总之，当需要使用内部可变性时，首选 Cell，只有类型没有实现 Copy 特征时，再选择 RefCell。

```rust
// code snipet 1
let x = Cell::new(1);
let y = &x;
let z = &x;
x.set(2);
y.set(3);
z.set(4);
println!("{}", x.get());

// code snipet 2 编译失败，原因是不能对基础类型取引用
let mut x = 1;
let y = &mut x;
let z = &mut x;
x = 2;
*y = 3;
*z = 4;
println!("{}", x);
```

### 内部可变性

Cell 与 RefCell 具有内部可变性，何为内部可变性？简单来说，**对一个不可变的值进行可变借用**。具体到 Cell 和 RefCell：

- Cell 通过内部的 `get set` 方法完成数据的获取和替换，即 `get` 提供不可变引用功能（读），`get set` 提供可变引用（读写）
- RefCell 通过 unsafe 操作，可以为一个无论是否可变的类型（变量/值），**对外提供该类型的不可变引用和可变引用**，由于是 unsafe 操作，编译时期 `borrow(不可变借用)` 和 `borrow_mut(可变借用)` 方法内部实现不受借用规则的限制，所以编译不会报错。但是两个方法的接收者变量不是 unsafe 操作，接收者会受到借用规则的限制，RefCell 智能指针在运行时会记录不可变借用和可变借用的次数，如果方法接收者变量不符合借用规则，则会 panic。

内部可变性并不符合 Rust 的基本借用规则：**不能对一个不可变的值进行可变借用**，这会破坏 Rust 的安全性保证。
这是因为当值不可变时，可能会有多个不可变的引用指向它，此时若将其中一个修改为可变的，会造成可变引用与不可变引用共存的情况，这可能会造成未定义的行为。

相反，可以对一个可变值进行不可变借用，根据借用规则只允许一个借用存在，所以当值可变时，最多只会有一个可变引用指向它，将其修改为不可变，那么最终依然是只有一个不可变的引用指向它。

Rust 的借用规则是内存安全的保证基石，但是有些场景遵守借用规则会非常麻烦，比如由于 Rust 的 mutable 特性，一个结构体中的字段，要么全都是 immutable，要么全部是 mutable，**不支持针对部分字段进行设置**。

比如；

```rust
// 通过 RefCell，让一个结构体既有不可变字段，也有可变字段
#[derive(Debug)]
struct Person {
    name: RefCell<String>,
    age: i32,
}
let p = Person {
    name: RefCell::new(String::from("L")),
    age: 18,
};
// p.age = 22; 错误的，如果需要age可更改，需要p是可变的。
*p.name.borrow_mut() = String::from("M"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
*p.name.borrow_mut() = String::from("N"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
println!("{p:?}");
```

如果需要修改 age 则需要将整个 Person 设置为可变，这种行为不合理。

又比如为自定义结构体实现外部特征，外部特征的方法接收者为 `self: &Self` 时：

```rust
// 定义在外部库中的特征，不能直接修改
pub trait Messenger {
    fn send(&self, msg: String);
}

// 自定义的数据结构和实现（消息队列结构体）
struct MsgQueue {
    msg_cache: Vec<String>,
}

// 为自定义数据结构实现外部特征
impl Messenger for MsgQueue {
    fn send(&self, msg: String) {
        self.msg_cache.push(msg) // 报错，因为接收者 self 的类型是不可变引用，不能通过不可变引用修改值
    }
}
```

因为接收者 self 的类型是不可变引用，**不能通过不可变引用修改值**，所以上述代码编译就会报错。
并且由于实现的是**外部特征，不能直接修改方法签名**，此时就依靠 `RefCell` 的内部可变性为不可变值提供可变引用，进而修改：

```rust
// 定义在外部库中的特征，不能直接修改
pub trait Messenger {
    fn send(&self, msg: String);
}

// 自定义的数据结构和实现（消息队列结构体），用 RefCell 为一个无论是否可变的类型（变量/值），**对外提供该类型的不可变引用和可变引用**
struct MsgQueue {
    msg_cache: RefCell<Vec<String>>,
}

// 为自定义数据结构实现外部特征
impl Messenger for MsgQueue {
    fn send(&self, msg: String) {

        // 编译正常，虽然接收者 self 的类型是不可变引用，但 msg_cache 通过内部可变性提供了可变引用。
        // 此外，运行正常，RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则正常运行。
        self.msg_cache.borrow_mut().push(msg)
    }
}
```

通过 RefCell 为一个无论是否可变的类型（变量/值），**对外提供该类型的不可变引用和可变引用**，解决了 `&self` 不能通过不可变引用改变值的问题。

#### 总结

当遇到需要通过不可变引用修改数据，或者需要被到处使用和修改然后导致借用关系难以管理时，就可以考虑内部可变性的 Cell 和 RefCell。

### Rc/Arc + RefCell 的组合使用

可以将所有权、借用规则和这些智能指针做一个对比：
| Rust 规则 | 智能指针带来的额外规则 |
| --------------------------------- | ------------------------------------ |
| 一个数据只有一个所有者 | Rc/Arc 让一个数据可以拥有多个所有者 |
| 要么多个不可变借用，要么一个可变借用 | RefCell 实现编译期可变、不可变引用共存 |
| 违背规则导致编译错误 | 违背规则导致运行时 panic |

`Rc/Arc` 和 `RefCell` 合理结合，可以解决 Rust 中严苛的所有权和借用规则带来的某些场景下难使用的问题，甚至某些时候可以达到其他带 GC 的高级语言的程度。

- Rc/Arc 智能指针通过引用计数（不可变引用）在符合借用规则的情况下实现一个值可以被多个变量访问。实现原理是：**利用结构体存储底层数据的地址和引用次数**，底层数据（实际类型数据）存放在堆上，结构体（胖指针，智能指针）存储在栈上作为管理信息数据管理实际类型数据。
- RefCell 通过内部 unsafe 操作实现数据的可变性，为一个无论是否可变的类型（变量/值），**对外提供该类型的不可变引用和可变引用**。

```rust
// Rc与RefCell的结合使用，可以让rust变得像其他高级语言一样使用变量/值
let s = Rc::new(RefCell::new(String::from("Hello World")));
let s1 = s.clone();
let s2 = s.clone();
s1.borrow_mut().push_str(" ❌"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
s1.borrow_mut().push_str(" 2"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
println!("{s:?}");
*s2.borrow_mut() = String::from("Hello World"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
println!("{s:?}");
```

两者的结合流程认识 Rc<RefCell<T>>：

- RefCell 为一个无论是否可变的类型（变量/值）提供不可变引用和可变引用，让数据减少借用规则的影响，让数据更容易被改变
- Rc/Arc 为一个类型提供简化的生命周期管理（回收资源），让 rust 的变量达到传统 GC 语言指针引用的便捷
  Rc/Arc 结合 RefCell 后功能上可以看成**减少手动管理生命周期（回收资源）的步骤，并且可以随时获取不可变引用和可变引用的类型**，即能达到传统带 GC 语言变量的程度。

#### 性能损耗

功能上 Rc/Arc 与 RefCell 的结合可以极大的降低生命周期管理和借用规则的复杂性，并且在性能上，这个组合也是非常高的。
大致相当于没有线程安全版本的 C++ std::shared_ptr 指针，事实上，C++ 这个指针的主要开销也在于原子性这个**并发原语**上，毕竟线程安全在哪个语言中开销都不小。

#### 内存损耗

Rc/Arc 与 RefCell 的结合相当于以下结构体，从对内存的影响来看，仅仅多分配了三个 usize/isize，并没有其它额外的负担。

```rust
struct Wrapper<T> {
    // Rc 数据
    strong_count: usize,
    weak_count: usize,

    // Refcell 数据
    borrow_count: isize,

    // 包裹的数据
    item: T,
}
```

#### CPU 损耗

从 CPU 来看，损耗如下：

- 对 Rc<T> 解引用是免费的（编译期自动转换），但是 `*` 带来的间接取值并不免费
- 克隆 Rc<T> 需要将当前的引用计数跟 0 和 usize::Max 进行一次比较，然后将计数值加 1
- 释放（drop） Rc<T> 需要将计数值减 1， 然后跟 0 进行一次比较
- 对 RefCell 进行不可变借用，需要将 isize 类型的借用计数加 1，然后跟 0 进行比较
- 对 RefCell 的不可变借用进行释放，需要将 isize 减 1
- 对 RefCell 的可变借用大致流程跟上面差不多，但是需要先跟 0 比较，然后再减 1
- 对 RefCell 的可变借用进行释放，需要将 isize 加 1（存疑：为什么不是减 1）

https://course.rs/advance/smart-pointer/cell-refcell.html#cpu-%E6%8D%9F%E8%80%97

其实这些细节不必过于关注，只要知道 CPU 消耗也非常低，甚至编译器还会对此进行进一步优化！

#### CPU 缓存 Miss

唯一需要担心的可能就是这种组合数据结构对于 CPU 缓存是否亲和，这个我们证明，只能提出来存在这个可能性，最终的性能影响还需要在实际场景中进行测试。

总之，分析这两者组合的性能还挺复杂的，大概总结下：

- 从表面来看，它们带来的内存和 CPU 损耗都不大，但是由于 Rc 额外的引入了一次间接取值（`*`），在少数场景下可能会造成性能上的显著损失
- CPU 缓存可能也不够亲和

### 过 Cell::from_mut 解决借用冲突

使用迭代器时，如果恰巧碰上需要修改迭代器内的数据，就会遇到两种情况，这两种情况都不能通过借用规则的检查：

- 不可变引用与可变引用一起使用：`iter()` 与 修改迭代器数据
- 可变引用与可变引用一起使用：`iter_mut()` 与 修改迭代器数据

```rust
let mut nums = vec![1, 2, 3, 4];
let mut i = 0;
for num in nums.iter().filter(|x| **x > 2) {
    // nums[i] = *num; 错误的，不能同时使用可变引用与不可变引用
    // i += 1;
}
let mut i = 0;
for num in nums.iter_mut().filter(|x| **x > 2) {
    // nums[i] = *num; 错误的，不能同时使用多个可变借用
    // i += 1;
}
```

对于迭代器出现的这两个场景，多个不可变引用与不可引用和可变引用同时使用的问题，可以**通过索引解决**，即不使用迭代器就不会出现问题：

```rust
let mut nums = vec![1, 2, 3, 4];
let mut i = 0;
for j in 0..nums.len() {
    if (nums[j] > 2) {
        nums[i] = nums[j];
        i += 1;
    }
}
```

但是使用索引就违背迭代器的初衷了，毕竟迭代器会让代码更加简洁。此时可以使用 `from_mut` 方法来解决这个问题：

```rust
// 使用索引不符合迭代器的初衷，迭代器能够简化代码
// 此时可以通过 `Cell` 解决这个问题，因此 Cell 可以提供 set get 方法设置数据。
let mut nums = vec![1, 2, 3, 4];
// cell_slice 是一个 Cell 的引用类型，内部元素是切片
let nums_slice = &mut nums[..];
let cell_slice = Cell::from_mut(&mut nums[..]);

// as_slice() 方法返回的是一个不可变的切片，这意味着返回的切片不能被修改，也就是nums不能被修改。
// let cell_slice_ref = Cell::from_mut(&mut nums.as_slice());

let mut nums = vec![1, 2, 3, 4];
// 内部元素是切片引用
let cell_slice_ref = Cell::from_mut(&mut nums.as_slice());

// 将 nums 中的元素变为 Cell 类型，就能够访问和设置元素数据
// 手动声明形式
let slice_nums = vec![Cell::new(1), Cell::new(2), Cell::new(3), Cell::new(4)];

// Cell::from_mut 与 Cell::as_slice_of_cells 结合生成，两种写法
let mut nums = vec![1, 2, 3, 4];
let cell_slice = Cell::from_mut(&mut nums[..]);
let slice_cell = Cell::as_slice_of_cells(cell_slice);
let slice_cell = Cell::from_mut(&mut nums[..]).as_slice_of_cells();

let i = 0;
for num in slice_cell.iter().filter(|x| (**x).get() > 2) {
    slice_cell[i].set(num.get()); // 通过slice_cell改变nums的数据，避免直接修改nums让不可变引用和可变引用同时存在，导致借用规则检查失败
}
println!("{nums:?}");
```

### 内部可变性的 Drop 的流程认识，与 Rc 和 Arc 对比

在 Rc/Arc 中，rust 通过**引用计数 (`reference counting`)**来简化不可变引用对应值的 Drop 实现。
在 Cell/RefCell 中，rust 又是通过什么来维护 Drop 的流程？
Cell/RefCell 的 Drop 流程很简单，与 rust 普通的堆上值是一样的释放流程。

### 总结

- Cell 与 RefCell 带来了内部可变性这个重要特性，将借用规则的检查从编译期推迟到运行期，但是这个检查并不能被绕过，RefCell 在运行期的报错会造成 panic。
- RefCell 适用于编译器误报或者一个引用被在多个代码中使用、修改以至于难于管理借用关系时，还有就是需要内部可变性时。
- 从性能上看，RefCell 由于是非线程安全的，因此无需保证原子性，性能虽然有一点损耗，但是依然非常好，而 Cell 则完全不存在任何额外的性能损耗。
- Rc 跟 RefCell 结合使用可以实现多个所有者共享同一份数据，非常好用，但是潜在的性能损耗也要考虑进去，建议对于热点代码使用时，做好 benchmark。

### Code

```rust
fn main() {
    //  use std::cell::Cell;
    let s_cell = Cell::new("Hello World");
    let s = s_cell.get(); // 获取内部数据
    s_cell.set("Hi"); // 不可变引用直接修改内部数据
    println!("{s_cell:?}, {s}");

    struct MyCell<T: Copy> {
        value: T,
    }
    impl<T: Copy> MyCell<T> {
        fn new(v: T) -> MyCell<T> {
            MyCell { value: v }
        }
        fn set(&mut self, v: T) {
            self.value = v;
        }
    }
    let my_cell = MyCell::new("Hello World");
    // my_cell.set("Hi"); 错误，set函数 `set(&mut self, v: T)` 要求接收者是可变引用 `self: &mut Self`，而此时的 `my_cell` 是一个不可变引用。

    // **RefCell 的功能是通过 unsafe 操作，为一个类型（变量/值）对外提供该类型的不可变引用和可变引用，无论这个类型（变量/值）是否可变**。
    // RefCell 会在内部记录不可变引用（borrow）和可变引用（borrow_mut）的使用次数，通过使用次数来判断此时是否符合借用规则
    let s = RefCell::new(String::from("Hello World"));
    let s1 = s.borrow(); // RefCell 记录一次不可变引用，不可变引用是1，可变引用是0，符合借用规则，正常运行
    let s1 = s.borrow(); // RefCell 记录一次不可变引用，不可变引用是2，可变引用是0，符合借用规则，正常运行
                         // let s2 = s.borrow_mut(); // RefCell 记录一次可变引用，不可变引用是2，可变引用是1，此时会报错，因为不能同时存在不可变引用和可变引用

    let s = RefCell::new(String::from("Hello World"));
    let s1 = s.borrow_mut(); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行
                             // let s2 = s.borrow_mut(); // RefCell 记录一次可变引用，不可变引用是0，可变引用是2，此时会报错，因为不能同时存在多个可变引用（一个可变引用周期内存在另外一个可变引用）
    println!("{s1}");

    let s = RefCell::new(String::from("Hello World"));
    let mut s2 = s.borrow_mut(); // 给出原始数据的可变引用
    *s2 = String::from("Hi");
    println!("{:?}", &s2); // 运行成功，无论是编译器还是运行时，都是符合rust的借用规则的

    let mut s = String::from("Hello World");
    let s_ref = RefCell::new(s);
    drop(s_ref); // 释放资源

    // 通过 RefCell，让一个结构体既有不可变字段，也有可变字段
    #[derive(Debug)]
    struct Person {
        name: RefCell<String>,
        age: i32,
    }
    let p = Person {
        name: RefCell::new(String::from("L")),
        age: 18,
    };
    // p.age = 22; 错误的，如果需要age可更改，需要p是可变的。
    *p.name.borrow_mut() = String::from("M"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
    *p.name.borrow_mut() = String::from("N"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
    println!("{p:?}");

    // Rc与RefCell的结合使用，可以让rust变得像其他高级语言一样使用变量/值
    let s = Rc::new(RefCell::new(String::from("Hello World")));
    let s1 = s.clone();
    let s2 = s.clone();
    s1.borrow_mut().push_str(" ❌"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
    s1.borrow_mut().push_str(" 2"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
    println!("{s:?}");
    *s2.borrow_mut() = String::from("Hello World"); // RefCell 记录一次可变引用，不可变引用是0，可变引用是1，符合借用规则，正常运行。borrow_mut没有接收者意味着可变引用使用后被释放，可变引用计数归0
    println!("{s:?}");

    // 使用迭代器时，如果恰巧碰上需要修改迭代器内的数据，就会遇到两种情况：
    // 不可变引用与可变引用一起使用 iter() 与 修改迭代器数据
    // 可变引用与可变引用一起使用 iter_mut() 与 修改迭代器数据
    // 这两种情况都不能通过借用规则的检查
    let mut nums = vec![1, 2, 3, 4];
    let mut i = 0;
    for num in nums.iter().filter(|x| **x > 2) {
        // nums[i] = *num; 错误的，不能同时使用可变引用与不可变引用
        // i += 1;
    }
    let mut i = 0;
    for num in nums.iter_mut().filter(|x| **x > 2) {
        // nums[i] = *num; 错误的，不能同时使用多个可变借用
        // i += 1;
    }

    // 对于迭代器出现的这两个场景，多个不可变引用与不可引用和可变引用同时使用的问题，可以通过索引来解决
    let mut nums = vec![1, 2, 3, 4];
    let mut i = 0;
    for j in 0..nums.len() {
        if (nums[j] > 2) {
            nums[i] = nums[j];
            i += 1;
        }
    }

    // 使用索引不符合迭代器的初衷，迭代器能够简化代码
    // 此时可以通过 `Cell` 解决这个问题，因此 Cell 可以提供 set get 方法设置数据。
    let mut nums = vec![1, 2, 3, 4];
    // cell_slice 是一个 Cell 的引用类型，内部元素是切片
    let nums_slice = &mut nums[..];
    let cell_slice = Cell::from_mut(&mut nums[..]);

    // as_slice() 方法返回的是一个不可变的切片，这意味着返回的切片不能被修改，也就是nums不能被修改。
    // let cell_slice_ref = Cell::from_mut(&mut nums.as_slice());

    let mut nums = vec![1, 2, 3, 4];
    // 内部元素是切片引用
    let cell_slice_ref = Cell::from_mut(&mut nums.as_slice());

    // 将 nums 中的元素变为 Cell 类型，就能够访问和设置元素数据
    // 手动声明形式
    let slice_nums = vec![Cell::new(1), Cell::new(2), Cell::new(3), Cell::new(4)];

    // Cell::from_mut 与 Cell::as_slice_of_cells 结合生成，两种写法
    let mut nums = vec![1, 2, 3, 4];
    let cell_slice = Cell::from_mut(&mut nums[..]);
    let slice_cell = Cell::as_slice_of_cells(cell_slice);
    let slice_cell = Cell::from_mut(&mut nums[..]).as_slice_of_cells();

    let i = 0;
    for num in slice_cell.iter().filter(|x| (**x).get() > 2) {
        slice_cell[i].set(num.get()); // 通过slice_cell改变nums的数据，避免直接修改nums让不可变引用和可变引用同时存在，导致借用规则检查失败
    }
    println!("{nums:?}");
}
```
