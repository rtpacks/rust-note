## 项目 Package 和包 Crate

将大的代码文件拆分成包和模块，有利于实现代码抽象和复用。Rust 也提供了相应概念用于代码的组织管理：

- 项目(Packages)：一个 Cargo 提供的 feature，可以用来构建、测试和分享包
- 工作空间(WorkSpace)：对于大型项目，可以进一步将多个包联合在一起，组织成工作空间
- 包(Crate)：一个由多个模块组成的树形结构，可以作为三方库进行分发，也可以生成可执行文件进行运行
- 模块(Module)：可以一个文件多个模块，也可以一个文件一个模块，模块可以被认为是真实项目中的代码组织单元

### 1. 包 Crate

对于 Rust 而言，包（Crate）是一个独立的可编译单元，它编译后会生成一个可执行文件或者一个库。
一个包会将相关联的功能打包在一起，使得该功能可以很方便的在多个项目中分享。例如标准库中没有提供但是在三方库中提供的 rand 包，它提供了随机数生成的功能，我们只需要将该包通过 use rand; 引入到当前项目的作用域中，就可以在项目中使用 rand 的功能：rand::XXX。
同一个包中不能有同名的类型，但是在不同包中就可以。例如，虽然 rand 包中，有一个 Rng 特征，可是我们依然可以在自己的项目中定义一个 Rng，前者通过 rand::Rng 访问，后者通过 Rng 访问，对于编译器而言，这两者的边界非常清晰，不会存在引用歧义。

### 2. 项目 Package

项目 Package 与包 Crate 很容易被搞混，Package 可以认为是整个项目工程，它分为两类：**二进制 Package**和**库 Package**。

#### 1. 二进制 Package

使用 `cargo new my-project` 命令创建一个项目 Package，发现如下结构：

```sh
src
   main.rs
cargo.toml
```

Cargo 创建了一个名称是 my-project 的项目 Package，同时在其中创建了 ` Cargo.toml`` 和  `src/main.rs` 文件。
Cargo 有一个惯例：src/main.rs 是**二进制包 Crate**的根文件，该二进制包的包名跟所属的 二进制项目 Package 相同，在这里都是 my-project，所有的代码执行都从该文件中的 fn main() 函数开始。

#### 2. 库 Package

使用 `cargo new my-lib --lib` 命令创建一个项目 Package，发现如下结构

```sh
src
   lib.rs
cargo.toml
```

如果试图运行 my-lib **库 Package**，会报错：

```sh
error: a bin target must be available for `cargo run`
```

原因是库类型的项目 Package 只能作为三方库被其它项目引用，不能独立运行，只有之前的二进制项目 Package 才可以运行。
与 `src/main.rs` 类似，Cargo 的惯例：如果一个项目 Package 包含有 src/lib.rs，意味**它包含有一个库类型的包 Crate**，该库类型包的报名与所属的库类型 Package 相同，在这里是 my-lib，该包的根文件是 src/lib.rs。

#### 3. 易混淆的 Package 和包

学完二进制 Package 和库 Package 后，就可以理清项目 Package 和包 Crate 了。

- 首先，用 cargo new 创建的**项目 Package 和它其中包含的包 Crate 是同名的！** 项目 Package 根据包含的包 Crate 的类型不同分为**二进制项目 Package**和**库类型项目**。二进制项目包含 main.rs 和最多一个 lib.rs，库项目只含有 lib.rs。
- 其次，项目 Package 是一个项目工程，而包 Crate 只是一个编译单元，如 src/main.rs 和 src/lib.rs 都是编译单元，因此它们都是包 Crate。

#### 4. 典型的 Package 结构

如果项目 Package 中仅包含 src/main.rs 文件，意味着它仅包含一个二进制同名包 my-project。

如果项目 Package 同时拥有 src/main.rs 和 src/lib.rs，那就意味着它包含两个包：库包和二进制包，这两个包名也都是 my-project —— 都与项目 Package 同名。

一个真实项目中典型的 Package，会包含**多个二进制包**，这些包文件被放在 src/bin 目录下，每一个文件都是独立的二进制包，同时也会包含一个库包，该包只能存在一个 src/lib.rs

```sh
.
├── Cargo.toml
├── Cargo.lock
├── src
│   ├── main.rs
│   ├── lib.rs
│   └── bin
│       └── main1.rs
│       └── main2.rs
├── tests
│   └── some_integration_tests.rs
├── benches
│   └── simple_bench.rs
└── examples
    └── simple_example.rs
```

由于 Package 就是一个项目，因此它包含有独立的 Cargo.toml 文件，以及因为功能性被组织在一起的一个或多个包。
**一个 Package 只能包含一个库(library)类型的包 Crate，但是可以包含多个二进制可执行类型的包 Crate。**

- 唯一库包：src/lib.rs
- 默认二进制包：src/main.rs，编译后生成的可执行文件与 Package 同名
- 其余二进制包：src/bin/main1.rs 和 src/bin/main2.rs，它们会分别生成一个文件同名的二进制可执行文件
- 集成测试文件：tests 目录下
- 基准性能测试 benchmark 文件：benches 目录下
- 项目示例：examples 目录下

这种目录结构基本上是 Rust 的标准目录结构。

#### 5. 总结

Package 是对一个工程项目的统称，是各种包 Crate 的载体/容器，是 cargo 中便于管理包 Crate 的概念。
Crate 是编译单位，即真实承载代码的概念。Crate 被称为包，它有 lib 和 bin 两种，供别人调用的包或者是一个可执行的包。
`src/main.rs` 表示是一个可执行的包（bin），如果项目 Package 含有这个可执行的包，则被称为可执行的项目，也叫二进制项目。
`src/lib.rs` 表示是一个可供别人调用的包（lib），如果项目 Package 含有这个可供别人调用的包，则被称为库类型项目。
lib 和 bin 并不是互斥的，项目可以同时含有 lib 和 bin 即有 `src/main.rs` 和 `src/lib.rs`，表示是一个可执行又可供别人调用的项目。

包 Crate 是真实承载代码的概念，只有包 Crate 才能寻到代码，因此可供别人调用，也就是别人调用的代码一定是来自包，并且是 lib 包。
项目 Package 是各种包 Crate 的载体/容器，是 cargo 管理包的概念。
如果想调用别人的代码并且是通过 cargo 依赖形式的调用，就需要通过 cargo 去调用项目的 lib 包。
如果一个项目含有多个 lib 包，cargo 就无法知道需要使用哪个包。
因此每一个项目都是最多只能有一个 lib 包，别人通过包的形式调用这个项目的代码，cargo 就知道应该从这唯一的 lib 包中寻找相应的代码。
将别人的项目 Package 作为依赖，其实就是在调用别人的项目中唯一的 lib 包 Crate。

为什么项目 Package 中又允许存在多个 bin 包呢？
因为在项目中不需要通过 cargo 去寻找哪些是可执行包，这些可执行包是供开发者使用而不是供外部调用的。
因此 cargo 不寻找可执行包，只由开发者管理的项目可执行包不存在唯一的条件限制，因为开发很清楚自己的可执行包（bin crate）在哪。

既然可执行包有多个，那么一个 src/main.rs 就不够用了，因为一个`main.rs`就表示一个可执行包。
其他的可执行包就得放到 src/bin 下面，每个 crate 一个文件，换句话说，每个文件都是一个不同的 bin crate。

### 3. 阅读

- https://course.rs/basic/crate-module/crate.html
- http://liubin.org/blog/2021/01/19/packages-slash-crate-slash-modules-in-rust/
