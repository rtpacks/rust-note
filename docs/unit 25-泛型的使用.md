## 泛型的使用

泛型的存在让抽象程度更高，因此在 rust 中泛型用处很多，如结构体、枚举、方法中都可以使用泛型。

注意：**使用的泛型都需要提前声明。**

### 结构体中使用泛型

```rs
// 声明泛型T，x，y是同一类型
struct Point<T> {
    x: T,
    y: T,
}

fn main() {
    let integer = Point { x: 5, y: 10 };
    let float = Point { x: 1.0, y: 4.0 };
}
```

### 枚举中使用泛型

提到枚举类型，Option 永远是第一个应该被想起来的，Option<T> 是一个拥有泛型 T 的枚举类型，它第一个成员是 Some(T)，存放了一个类型为 T 的值。
得益于泛型的引入，我们可以在任何一个需要返回值的函数中，使用 Option<T> 枚举类型来做为返回值，用于返回一个任意类型的值 Some(T)，或者没有值 None。

另外，Result 也是常见的枚举：如果函数正常运行，则最后返回一个 Ok(T)，T 是函数具体的返回值类型，如果函数异常运行，则返回一个 Err(E)，E 是错误类型。

```rs
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### 方法中使用泛型

方法的定义一般是存在结构体中 `impl Struct { fn function() {} }`，泛型又是代表数据类型的变量（变量自身也可以用作类型），所以在方法中使用泛型一般是如下形式：

```rs
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

fn main() {
    let p = Point { x: 5, y: 10 };

    println!("p.x = {}", p.x());
}
```

提前声明泛型 `T`，只有提前声明了，我们才能在 Point<T>中使用它，这样 Rust 就知道 Point 的尖括号中的类型是泛型而不是具体类型。

需要注意的是，这里的 Point<T> 不再是泛型声明，而是一个完整的结构体类型，因为我们定义的结构体就是 Point<T> 而不再是 Point。
即当我们声明了 `let p = Point{x: 5, y: 10}` 后，`p`的类型就不再是 `Point` 或 `Point<T>` 了，而是具体的 `Point<i32>` 类型。

此外，还可以定义多个泛型

```rs
struct Point<T, U> {
    x: T,
    y: U,
}

impl<T, U> Point<T, U> {
    fn mixup<V, W>(self, other: Point<V, W>) -> Point<T, W> {
        Point {
            x: self.x,
            y: other.y,
        }
    }
}
```

### 为具体的泛型类型实现方法

对于 Point<T> 类型，你不仅能定义基于 T 的方法，还能针对特定的具体类型进行方法定义，这些方法只在对应的数据类型生效。
如以下方法指挥在数据类型为 f64 时生效，这样我们就**能针对特定的泛型类型实现某个特定的方法，对于其它泛型类型则没有定义该方法**。

```rs
struct Point<T> {
    x: T,
    y: T,
}

impl Point<f64> {
    fn distance_from_origin(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
```

### 泛型的引用类型

泛型的引用类型常出现在实现相同 Trait 但不同类型的数据类型上，如字符数组、整数数组，更具体还有 i32 数组，i64 数组等。

如果参数是一个引用，且又使用泛型，则需要使用泛型的引用 `&T或&mut T`，&T 是不可变泛型引用，&mut T 是可变泛型引用。

如打印不同类型的数组，实现也不难，唯一要注意的是需要对 T 加一个限制 std::fmt::Debug，该限制表明 T 可以用在 println!("{:?}", arr) 中，因为 {:?} 形式的格式化输出需要 arr 实现该特征。

```rs
fn display_arr<T: std::fmt::Debug>(arr: &[T]) {
    println!("{:#?}", arr);
}

let arr: [i32, 4] = [1, 2, 3, 4];
display_arr(&arr);

let arr: [char, 4] = ['1', '2', '3', '4'];
display_arr(&arr);
```
