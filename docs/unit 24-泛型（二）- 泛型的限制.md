## 泛型

### 泛型的限制

```rs
fn double<T>(i: T) -> T {}
```

在 double 函数的定义中，double 期待的是对数值进行加法操作，但**泛型**却可以代表各种类型。
因此，还需要对泛型 T 进行限制，否则在调用 double 函数时就允许传递字符串类型、Vec 类型等值作为函数参数，这会产生错误。

事实上，在 double 的函数体内对泛型 T 的值 i 进行加法操作，只有实现了 Trait `std::ops::Add` 的类型才能使用+进行加法操作。
因此要限制泛型 T 是那些实现了 std::ops::Add 的数据类型。

**限制泛型**也叫做 Trait 绑定(**Trait Bound**)，其语法有两种：

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
