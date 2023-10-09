## Trait Object 和泛型

对比一下 Trait 对象和泛型：

- **Trait 对象可以被看作一种数据类型**，它总是以引用的方式被使用，在运行期间，它在栈中保存了具体类型的实例数据和实现自该 Trait 的方法。
- 泛型不是一种数据类型，它可被看作是**数据类型的参数形式或抽象形式**，在编译期间会被替换为具体的数据类型。

Trait Objecct 方式称为动态分派(dynamic dispatch)，它在程序运行期间动态地决定具体类型。
Rust 泛型是静态分派，它在编译期间会代码膨胀（code bloat），将泛型参数转变为使用到的每种具体类型。

### 什么时候使用泛型？

阅读：https://rust-book.junmajinlong.com/ch12/03_trait_obj_generic.html

例如，类型 Square 和类型 Rectangle 都实现了 Trait Area 以及方法 get_area，现在要创建一个 vec，这个 vec 中包含了任意能够调用 get_area 方法的类型实例。
这种需求建议采用 Trait Object 方式：

```rs
trait Area{
  fn get_area(&self)->f64;
}

struct Square(f64);
struct Rectangle(f64, f64);
impl Area for Square{
  fn get_area(&self) -> f64 {self.0 * self.0}
}
impl Area for Rectangle{
  fn get_area(&self) -> f64 {self.0 * self.1}
}

let mut sharps: Vec<&dyn Area> = vec![];
sharps.push(&Square(3.0));
sharps.push(&Rectangle(3.0, 2.0));
println!("{}", sharps[0].get_area());
println!("{}", sharps[1].get_area());
```

在上面的示例中，Vec sharps 用于保存多种不同类型的数据，只要能调用 get_area 方法的数据都能存放在此，而调用 get_area 方法的能力，来自于 Area Trait。
因此，使用动态的类型 dyn Area 来描述所有这类数据。当 sharps 中任意一个数据要调用 get_area 方法时，都会从它的 vtable 中查找该方法，然后调用。

但如果改一下上面示例的需求，不仅要为 f64 实现上述功能，还要为 i32、f32、u8 等类型实现上述功能，这时候使用 Trait Object 就很冗余了，要为每一个数值类型都实现一次。
使用泛型则可以解决这类因数据类型而导致的冗余问题。

```rs
trait Area<T> {
  fn get_area(&self) -> T;
}

enum Sharp<T>{
  Square(T),
  Rectangle(T, T),
}

impl<T> Area<T> for Sharp<T>
  where T: Mul<Output=T> + Clone + Copy
{
  fn get_area(&self) -> T {
    match *self {
      Sharp::Rectangle(a, b) => return a * b,
      Sharp::Square(a) => return a * a,
    }
  }
}
let sharps: Vec<Sharp<_>> = vec![
  Sharp::Square(3.0_f64),
  Sharp::Rectangle(3.0_f64, 2.0_f64),
];
sharps[0].get_area();
```

上面使用了泛型枚举，在这个枚举类型上实现 Area Trait，就可以让泛型枚举统一各种类型，使得这些类型的数据都具有 get_area 方法。

简单来说，Trait Object 是将一个一个亲自实现需要的功能，泛型则是在 Trait Object 的基础上抽象一层，适合更复杂对（多层）的场景。
如 长度为 i32 类型的正方形的 get_area 方法 与 长度不确定整数/浮点数的正方形的 get_area 方法。

- 前者可以使用 Trait Object 实现 Trait 获得 get_area 方法
- 后者使用泛型，在实现 Trait 的基础上，还需抽象一层以便使用 i32/i64/f32/f64

### 进一步深入 Trait 和泛型

Trait 特征与泛型的进阶知识：https://course.rs/basic/trait/advance-trait.html#%E6%B7%B1%E5%85%A5%E4%BA%86%E8%A7%A3%E7%89%B9%E5%BE%81

#### 关联类型

关联类型是在特征定义的语句块中，**申明一个自定义类型**，这样就可以在特征的方法签名中使用该类型。
以下是标准库中的迭代器特征 Iterator，它有一个 Item 关联类型，用于替代遍历的值的类型。
同时，next 方法也返回了一个 Item 类型，不过使用 Option 枚举进行了包裹，假如迭代器中的值是 i32 类型，那么调用 next 方法就将获取一个 Option<i32> 的值。

关联类型写法

```rs
pub trait Iterator {
  type Item;
  fn next(&mut self) -> Option<Self::Item>;
}
```

以上可以改造成泛型写法

```rs
pub trait Iterator<Item> {
    fn next(&mut self) -> Option<Item>;
}
```

代码块 1：当类型定义很复杂时，使用关联类型（别名）可以极大的增加可读性

```rs
pub trait CacheableItem: Clone + Default + fmt::Debug + Decodable + Encodable {
  type Address: AsRef<[u8]> + Clone + fmt::Debug + Eq + Hash;
  fn get_address(&self) -> Address; // 当类型定义很复杂时，使用关联类型可以极大的增加可读性
}
```

代码块 2：使用关联类型，外部（函数）无需声明 trait 所需的泛型

```rs
trait Container<A, B> {
    fn contains(&self, a: A, b: B) -> bool;
}
fn diff<A, B, C>(container: &C) -> i32 // 由于trait需要泛型，导致函数也需要声明泛型trait需要的泛型
  where
    C : Container<A, B> {...}


trait Container{
    type A;
    type B;
    fn contains(&self, a: &Self::A, b: &Self::B) -> bool;
}
fn diff<C: Container>(container: &C) {} // 使用关联类型，函数无需声明trait的泛型
```

关联类型的优势是**自定义类型的定义（类型别名）**，用一个自定义类型简化复杂的类型的编写，对比代码可以发现：

- 当类型定义复杂时（`AsRef<[u8]> + Clone + fmt::Debug + Eq + Hash`），关联类型写法可以极大的增加可读性。
- 使用泛型后，在使用的地方要标记类型 `Iterator<Item>`，而关联类型只需要写 `Iterator`，这在 impl 类型时由于 trait 需要泛型，导致函数也需要声明泛型 trait 需要的泛型，代码较为难读。

在泛型的限制中提到过：应当尽量不在定义类型时限制泛型的范围，除非确实有必要去限制，否则只在 impl 类型时去限制泛型，并且遵守缺失什么功能就添加什么限制的规范，这就是为了避免冗余。

```rs
#[derive(Debug)]
struct Food<T: Debug>(T); // 应当尽量不在定义类型时限制泛型的范围，除非确实有必要去限制，否则很可能是冗余的。
impl<T: Debug> Eatable for Food<T> {}

#[derive(Debug)]
struct Food<T>(T); // 尽量不去限制类型是什么，而是限制类型能做什么，也就是为类型实现功能即impl时限制类型能做什么
impl<T: Debug> Eatable for Food<T> {}
```

### code

```rs
fn main {
  trait Eatable {}

  #[derive(Debug)]
  struct Food1<T: Debug>(T); // 应当尽量不在定义类型时限制泛型的范围，除非确实有必要去限制，否则很可能是冗余的。
  impl<T: Debug> Eatable for Food1<T> {}

  #[derive(Debug)]
  struct Food2<T>(T); // 尽量不去限制类型是什么，而是限制类型能做什么，也就是为类型实现功能即impl时限制类型能做什么
  impl<T: Debug> Eatable for Food2<T> {}
}
```
