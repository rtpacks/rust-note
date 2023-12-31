## 泛型的限制

```rs
fn double<T>(i: T) -> T {}
```

在 double 函数的定义中，double 期待的是对数值进行加法操作，但**泛型**却可以代表各种类型。
因此，还需要对泛型 T 进行限制，否则在调用 double 函数时就允许传递字符串类型、Vec 类型等值作为函数参数，这会产生错误。

事实上，在 double 的函数体内对泛型 T 的值 i 进行加法操作，只有实现了 Trait `std::ops::Add` 的类型才能使用+进行加法操作。
因此要限制泛型 T 是那些实现了 std::ops::Add 的数据类型。

**限制泛型**也叫做 Trait 绑定，Trait 约束(**Trait Bound**)，其语法有两种：

- 在定义泛型类型 T 时，使用类似于 T: Trait_Name 这种语法进行限制
- 在返回值后面、大括号前面使用 where 关键字，如 where T: Trait_Name
  `T: trait_name` 这种形式中，`:` 不代表特征 Trait 的继承，而是表示对数据类型的限制，即 T: Trait_Name 表示将泛型 T 限制为那些实现了 Trait_Name Trait 的数据类型。

以下两种写法是等价的，但 where 关键字在复杂的定义中，可读性更好。

```rs
fn f<T: Clone + Copy>(i: T) -> T{}

fn f<T>(i: T) -> T
 where T: Clone + Copy {}
```

复杂场景

```rs
// 更复杂的示例：
fn query<M: Mapper + Serialize, R: Reducer + Serialize>(
    data: &DataSet, map: M, reduce: R) -> Results
{
    ...
}

// 此时，下面写法更友好、可读性更高
fn query<M, R>(data: &DataSet, map: M, reduce: R) -> Results
    where M: Mapper + Serialize,
          R: Reducer + Serialize
{
    ...
}
```

因此，在 `double` 函数的声明是：
T: std::ops::Add 表示泛型 T 只能代表那些实现了 std::ops::Add Trait 的数据类型，比如各种数值类型都实现了 Add Trait，因此 T 可以代表数值类型，而 Vec 类型没有实现 Add Trait，因此 T 不能代表 Vec 类型。

观察指定变量数据类型的写法 i: i32 和限制泛型的写法 T: Trait_Name，由此可知，Trait 其实是泛型的数据类型，Trait 限制了泛型所能代表的类型，正如数据类型限制了变量所能存放的数据格式。
有时候需要对泛型做多重限制，这时使用+即可。例如 `T: Add<Output=T>+Copy+Clone`，表示限制泛型 T 只能代表那些同时实现了 Add、Copy、Clone 这三种 Trait 的数据类型。

之所以要做多重限制，是因为有时候限制少了，泛型所能代表的类型不够精确或者缺失某种功能。比如，只限制泛型 T 是实现了 std::ops::Add Trait 的类型还不够，还要限制它实现了 Copy Trait 以便函数体内的参数 i 被转移所有权时会自动进行 Copy，但 Copy Trait 是 Clone Trait 的子 Trait，即 Copy 依赖于 Clone，因此限制泛型 T 实现 Copy 的同时，还要限制泛型 T 同时实现 Clone Trait。

简而言之，要对泛型做限制，一方面是函数体内需要某种 Trait 提供的功能(比如函数体内要对 i 执行加法操作，需要的是 std::ops::Add 的功能)，另一方面是要让泛型 T 所能代表的数据类型足够精确化(如果不做任何限制，泛型将能代表任意数据类型)。

### 总结

观察变量数据类型的写法 `i: i32` 和限制泛型的写法 `T: Trait` 可以得到一个结果，Trait 就是泛型的数据类型，泛型就是代表“已知的数据类型”的一个变量（和类型），正如数据类型限制了变量所能存放的数据格式（默认携带了类型）。

通常，应当尽量不在定义类型时限制泛型的范围，除非确实有必要去限制。

这有两点原因：

一方面泛型本就是为了描述更为抽象、更为通用的类型而存在的，限制泛型将使得类型变得更具体化，适用场景也更窄。
但是在 impl 类型时，应当去限制泛型，并且遵守缺失什么功能就添加什么限制的规范，这样可以使得所定义的方法不会过度泛化，也不会过度具体化。
简单来说，尽量不去限制类型是什么，而是限制类型能做什么。

另一方面，即使定义 `struct Food<T: Debug>`，在 `impl Food<T>` 时，也仍然要在 impl 时指定泛型的限制，否则将编译错误。
也就是说，如果某个泛型类型有对应的 impl，那么**在定义类型时指定的泛型限制很可能是多余的**。
但如果没有对应的 impl，那么可能有必要在定义泛型类型时加上泛型限制。

```rs
#[derive(Debug)]
struct Food<T: Debug>(T); // 应当尽量不在定义类型时限制泛型的范围，除非确实有必要去限制，否则很可能是冗余的。
impl<T: Debug> Eatable for Food<T> {}

#[derive(Debug)]
struct Food<T>(T); // 尽量不去限制类型是什么，而是限制类型能做什么
impl<T: Debug> Eatable for Food<T> {}
```

### code

```rs
fn main {
      trait Moveable {
        fn run() {
            println!("running")
        }
    }

    fn playSimple<T: Moveable + Debug, K: Moveable + Debug>(p: T) {}

    fn playComplex<T, K>()
    where
        T: Moveable + Debug,
        K: Moveable + Debug,
    {
    }
}
```
