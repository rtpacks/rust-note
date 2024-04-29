## Deref 解引用

在类型转换（二）通用类型转换中，有一个步骤是自动解引用，这里的自动解引用就和 Deref 特征相关：

1.  编译器检查它是否可以直接调用 T::foo(value)，即检查类型是否具有 foo 方法，称之为**值方法调用**
2.  如果值方法调用无法完成(例如方法类型错误或者类型没有对应函数的 Self 进行实现)，那么编译器会尝试**增加自动引用**，会尝试以下调用： `<&T>::foo(value)` 和 `<&mut T>::foo(value)`，称之为**引用方法调用**
3.  如果值方法和引用方法两个方法不工作，编译器会试着**解引用 T** ，然后再进行尝试。这里使用了 `Deref` 特征 —— 若 `T: Deref<Target = U>` (T 可以被解引用为 U)，那么编译器会使用 U 类型进行尝试，称之为**解引用方法调用**
4.  如果 T 不能被解引用，且 T 是一个定长类型(在编译期类型长度是已知的)，那么编译器也会尝试将 T 从**定长类型转为不定长类型**，例如将 [i32; 2] 转为 [i32]
5.  如果以上方式均不成功，那编译器将报错

### 通过 `*` 获取引用背后的值

> Rust 会在方法调用和字段访问时自动应用解引用强制多态（deref coercions），在一些其他情况下，如在标准比较操作或赋值中，Rust 不会自动应用解引用：**在表达式中不能自动地执行隐式 Deref 解引用操作**。
> println! 实际上调用的就是 Display 特征的方法，所以 println 时存在自动解引用

Deref 特征不仅可以自动解引用智能指针（引用），还可以解引用常规引用。

常规引用是一个指针类型，**包含目标数据存储的内存地址**。对常规引用使用 `*` 操作符，就可以通过解引用的方式获取到内存地址对应的数据值：

```rust
let x = 5;
let y = &5;
// println!("{}", x == y); 在标准比较或赋值中，rust不会自动应用解引用，因此不能直接比较
println!("{}, {}, {}", x, y, *y); // 可以自动解引用
```

### 智能指针解引用

常规指针的解引用与大多数语言并无区别，但 Rust 的解引用功能更为丰富，Rust 将其提升到了一个新高度。

考虑一下智能指针，它是一个结构体类型，如果直接对它进行解引用 `*myStruct`，显然编译器不知道该如何解析。为了避免复杂的人工转换，rust 为智能指针结构体设计了 Deref 特征。

实现 Deref 后的智能指针结构体，就可以像普通引用一样，通过 `*` 进行解引用，例如 `Box<T>` 智能指针，智能指针 x 被 `*` 解引用为 i32 类型的值 1，然后再进行求和：

```rust
let x = Box::new(1);
let sum = *x + 1;
```

#### 实现自定义智能指针

在 newtype 和类型别名章节，曾对 `Meters` 和 `Millimeters` 实现 Add 特征重载 `+`，让`Meters` 和 `Millimeters` 类型能够使用 `+` 操作符：

```rust
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
```

同样的，智能指针 `Box<T>` 实现 Deref 特征，能重载 `*` 操作符，使用 `*` 直接对结构体进行解引用操作。

既然实现某一特征后可以重载对应的操作符，那意味着只需要实现 Deref 特征，就能实现自定义智能指针，也就可以使用 `*` 操作符。

实现一个类似 `Box<T>` 的智能指针，分析：`Box<T>` 只是将实际值存储在堆上，结构体中没有包含长度、最大长度的其他信息，因此用元组结构体就能满足要求。

```rust
struct MyBox<T>(T);

impl<T> MyBox<T> {
     fn new(v: T) -> MyBox<T> {
         MyBox(v)
     }
}

let x = MyBox::new(2);
let y = *x + 1; 错误代码，因为MyBox没有实现Deref特征，直接对结构体使用解引用操作符，编译器不知道该怎么解析
```

**实现 Deref 特征，创建自定义指针**

```rust
impl<T> Deref for MyBox<T> {
     type Target = T;

     fn deref(&self) -> &Self::T {
         &self.0
     }
}

// 实现Deref特征后，可以使用 `*` 解引用操作符
let y = *x + 1;
// 类型转换，实现Deref特征，自动增加引用并转换为值方法调用
let y = *(Deref::deref(&x)) + 1;
```

#### `*` 背后的原理

很简单，当解引用 MyBox 智能指针时，根据通用类型转换流程：

> 通用类型转换是熟悉 rust 的必备技能，涉及到操作符就需要考虑类型是否发生转换
>
> 1.  编译器检查它是否可以直接调用 T::foo(value)，即检查类型是否具有 foo 方法，称之为**值方法调用**
> 2.  如果值方法调用无法完成(例如方法类型错误或者类型没有对应函数的 Self 进行实现)，那么编译器会尝试**增加自动引用**，会尝试以下调用： `<&T>::foo(value)` 和 `<&mut T>::foo(value)`，称之为**引用方法调用**
> 3.  如果值方法和引用方法两个方法不工作，编译器会试着**解引用 T** ，然后再进行尝试。这里使用了 `Deref` 特征 —— 若 `T: Deref<Target = U>` (T 可以被解引用为 U)，那么编译器会使用 U 类型进行尝试，称之为**解引用方法调用**
> 4.  如果 T 不能被解引用，且 T 是一个定长类型(在编译期类型长度是已知的)，那么编译器也会尝试将 T 从**定长类型转为不定长类型**，例如将 [i32; 2] 转为 [i32]
> 5.  如果以上方式均不成功，那编译器将报错

由于 `*` 操作符要求操作变量为引用类型，根据类型转换和 Deref 特征，`*x` 可以正常转换成 `*(Deref::deref(&x))`，`deref` 方法返回元组结构体中的元素 `&self.0`：

- 在 Deref 特征中声明了关联类型 Target，关联类型主要是为了提升代码可读性
- deref 返回的是一个**常规引用**，可以被 `*` 进行解引用
  因此类型转换成功，`*` 操作符正常解析。

Rust 为何要使用这个有点啰嗦的方式实现？原因在于**所有权系统**的存在。如果 deref 方法直接返回一个值，而不是引用，那么该值的所有权将被转移给调用者。
使用者不希望调用者仅仅只是 `*T` 一下，就拿走了智能指针中包含的值。

需要注意的是，`*` 不会无限递归替换，从 `*y` 到 `*(y.deref())` 只会发生一次，而不会继续进行替换然后产生形如 `*((y.deref()).deref())` 的怪物。这里会在连续解引用和引用归一化解释。

### 函数和方法中的隐式 Deref 转换

对于函数和方法的传参，Rust 提供了一个极其有用的 Deref 隐式转换。

若一个类型实现了 Deref 特征，那么在**类型的引用在传给函数或方法**时，编译器会根据函数的参数签名来决定是否对实参进行隐式的 Deref 转换，例如：

```rust
fn display(s: &str) {
    println!("{}",s);
}

let s = String::from("Hello World");
display(&s);
```

注意： **必须使用类型引用 `&` 的方式来触发 Deref，仅实参的引用类型才会触发自动解引用**。

分析以上代码：

- String 实现了 Deref 特征，可以在需要时自动被转换为 `&str` 类型
- 实参 `&s` 是一个 `&String` 类型，当它被传给 display 函数时，由于是类型的引用类型，并且实现了 Deref 特征，所以触发了编译器自动解引用，通过 Deref 转换将 `&String` 成了 `&str`

### 连续的隐式 Deref 转换

Rust 对解引用操作的提升除了表现在自定义智能指针的解引用外，还表现在在**连续隐式解引用**上，即直到找到适合的参数形式为止。

Box 是一个智能指针（存储在栈的引用和存储在堆上的实际类型数据），对比 `&String` 和 `&Box<String>`：

```rust
fn display(s: &str) {
    println!("{}",s);
}

let s = String::from("Hello World");
display(&s);

let s = Box::new(String::from("Hello World"));
display(&s);

```

`&Box<String>` 和 `&String` 一样，是能够正常被隐式转换的，关键在于**连续隐式转换**：
Box 实现了 Deref 特征，**实参传递的是引用类型，触发编译器自动解引用操作**，然后被 Deref 成 String 类型，结果编译器发现不能满足 display 函数参数 `&str` 的要求，接着发现 String 实现 Deref 特征，把 String Deref 成 &str，最终成功的匹配了函数参数。

如果不能连续隐式解引用，就需要手动拟合参数类型：

```rust
let x = &(*s)[..];
display(x);

display(&(*s)[..]);
```

结果不言而喻，肯定是 &s 的方式优秀得多。

总之，当参与其中的类型实现了 Deref 特征时，Rust 会分析该类型并且连续使用 Deref 直到最终获得一个引用来匹配函数或者方法的参数类型，这种行为是在编译期完成的，完全不会造成任何的性能损耗。

但是 Deref 并不是没有缺点，缺点就是：如果你不知道某个类型是否实现了 Deref 特征，那么在看到某段代码时，并不能在第一时间反应过来该代码发生了隐式的 Deref 转换。

事实上，不仅仅是 Deref，在 Rust 中还有各种 `From/Into` 等等会给阅读代码带来一定负担的特征。还是那句话，一切选择都是权衡，有得必有失，得了代码的简洁性，往往就失去了可读性，Go 语言就是一个刚好相反的例子。

这种隐式转换/连续隐式转换不仅可以用在函数的参数类型上，还可以用在赋值过程中：

```rust
let s = Box::new(String::from("Hello World"));
let s1 = &s;
let s2: &str = &s;
let s2 = &s as &str;
let s3 = s.to_string();
```

- 对于 s1，只是简单的取引用，因此类型是 `&Box<String>`，
- 对于 s2，通过两次 Deref 将 &str 类型的值赋给了它（**赋值操作需要手动解引用**，即在赋值过程中手动标注类型）
- 对于 s3，直接调用方法 to_string，实际上 Box 根本没有没有实现该方法，能调用 to_string，是因为编译器对 Box 应用了 Deref 的结果（**方法调用会自动解引用**），即通用类型转换（五个步骤）

> Rust 会在方法调用和字段访问时自动应用解引用强制多态（deref coercions），在一些其他情况下，如在标准比较操作或赋值中，Rust 不会自动应用解引用：**在表达式中不能自动地执行隐式 Deref 解引用操作**。

不仅是 Box 内置的智能指针，自定义智能指针 `MyBox` 也能实现相同的功能：

```rust
struct MyBox<T>(T);
impl<T> MyBox<T> {
    fn new(v: T) -> MyBox<T> {
        MyBox(v)
    }
}

// 为自定义类型实现Deref特征，变为智能指针
impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn display(s: &str) {
     println!("{s}");
}

let s = String::from("Hello World");
let s = MyBox::new(String::from("Hello World"));
display(&s); // 通过传递实参的引用类型，触发编译器自动解引用操作

let s1 = &s;
let s2: &str = &s;
let s2 = &s as &str;
let s3 = s.to_string();
```

### Deref 规则总结

**一个类型为 T 的对象 `foo`，如果 `T: Deref<Target=U>` 即 T 实现了 Deref 特征，那么 foo 的引用 `&foo` 在需要的时候会被自动转换为 `&U`**。

```rust
let s = String::from("Hello World");
let s1 = &s;
let s2: &str = &s;
let s2 = &s as &str;
let s3 = s.to_string();
```

#### 引用归一化

引用归一化 `T: Deref<Target=U>` 包含两部分内容：

第一是把**内置智能指针（Box、Rc、Arc、Cow 等）或自定义智能指针**，根据 `T: Deref<Target=U>` 重载的 Deref 特征的 `deref` 方法，从结构体脱壳，并将其变为内部类型的引用类型 `&v`

第二是针对多重引用归一化，如将引用类型的引用 `&&v` 归一成 `&v`。这是因为在标准库中为引用类型实现了 Deref 特征：`&T: Deref<Target=U>`，当 T 是一个引用类型时，`&T` 就代表引用类型的引用：

```rust
impl<T: ?Sized> Deref for &T {
    type Target = T;

    fn deref(&self) -> &T {
        *self
    }
}
```

以上的实现就是将多重引用归一化的关键，为 `&T` 实现 Deref 特征，意味着 Self 为 `Self: &T` 类型，那么 deref 方法的接收者 `&self == self: &Self` 为 `self: &(&T)` 类型，输入为 `&&T` 返回为 `&T` 类型，即针对引用的引用，最终归一化成 `&T`。

```shell
Self = &T -> &self = self: &Self = self: &&T -> *self = *&Self = *&&T = &T
```

案例：

```rust
let s = String::from("Hello World");
let s1 = &s;
let s2: &str = &s;
let s2 = &s as &str;
let s3 = s.to_string();
let s4 = (&s1).to_string(); // 归一化

let s = MyBox::new(String::from("Hello World"));
let s1 = &s;
let s2: &str = &s;
let s2 = &s as &str;
let s3 = s.to_string();
let s4 = (&s1).to_string(); // 归一化

fn display(s: &str) {
     println!("{s}");
}
display(&s); // 智能指针可以被自动脱壳为内部的 `String` 引用 `&String`，然后 `&String` 再自动解引用为 `&str`
```

### 三种 Deref 转换

以上的案例都是不可变的 Deref 转换，Rust 除了支持不可变引用的 Deref 转换外，还支持以下两种引用的转换：

- 一个可变的引用转换成另一个可变的引用
- 一个可变引用转换成不可变的引用

转换的规则如下：

- 当 `T: DerefMut<Target=U>`，可以将 `&mut T` 转换成 `&mut U`，即将可变引用 DerefMut 为可变引用
- 当 `T: Deref<Target=U>`，可以将 `&mut T` 转换成 `&U`，即将可变引用 Deref 不可变引用
- 当 `T: Deref<Target=U>`，可以将 `&T` 转换成 `&U`，即将不可变引用 Deref 不可变引用

```rust
struct MyBox<T>(T);
impl<T> MyBox<T> {
    fn new(v: T) -> MyBox<T> {
        MyBox(v)
    }
}

// 为自定义类型实现Deref特征，变为智能指针
impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// 实现DerefMut特征，DerefMut的前提是实现了Deref特征
impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// 不可变引用Deref转换为不可变引用
fn display(s: &str) {
    println!("{s}");
}

// 可变引用DerefMut转换为可变引用，实现DerefMut的前提是实现了Deref特征，因为 `pub trait DerefMut: Deref`
fn display_mut(s: &mut String) {
    s.push_str("world");
    println!("{}", s);
}

let s = MyBox::new(String::from("Hello World"));
display(&s); // 不可变引用Deref转换为不可变引用

let mut s = MyBox::new(String::from("Hello World"));
display_mut(&mut s); // 可变引用通过DerefMut转换为新的可变引用

display(&mut s); // 可变引用通过Deref转换为新的不可变引用
```

需要注意的几点：

- 只有类型的引用才会触发编译器自动解引用功能
- 要实现 DerefMut 必须要先实现 Deref 特征：`pub trait DerefMut: Deref`
- `T: DerefMut<Target=U>` 解读：将 `&mut T` 类型通过 DerefMut 特征的方法转换为 `&mut U` 类型，对应上例中，就是将 `&mut MyBox<String>` 转换为 `&mut String`

对于上述三条规则中的第二条，它比另外两条稍微复杂了点：Rust 可以把可变引用隐式的转换成不可变引用，但反之则不行。

如果从 Rust 的所有权和借用规则的角度考虑，当你拥有一个可变的引用，那该引用肯定是对应数据的唯一借用，那么此时将可变引用变成不可变引用并不会破坏借用规则；但是如果你拥有一个不可变引用，那同时可能还存在其它几个不可变的引用，如果此时将其中一个不可变引用转换成可变引用，就变成了可变引用与不可变引用的共存，最终破坏了借用规则。

### 总结

Deref 可以说是 Rust 中最常见的**隐式类型转换**，它虽然复杂，但是还是属于类型转换中的一种。Deref 最重要的特点就是归一化，包含两个方面：

- 只要链条上的类型实现了 Deref 特征，它可以实现如 `Box<String> -> String -> &str` 连续的隐式转换
- 针对多重引用类型，如引用的引用类型 `&&T`，可以实现将 `&&T` 归一成 `&T`

在程序中也可以为自定义类型实现 Deref 特征，但是原则上来说，只应该为**自定义的智能指针**实现 Deref。
例如，虽然可以为自定义数组类型实现 Deref 以避免 `myArr.0[0]` 的使用形式，但是 Rust 官方并不推荐这么做，特别是在开发三方库时。

> Box 是有很多特殊性质的, 完全可以把他当作原生类型看待. 比如他的 Deref 实现就很特殊,(明明看上去是无限递归, 但是编译器却明白他的含义)。
>
> Box 的 Deref 的实现是对自身解引用后将引用传递出去，而且不会无限递归。。。看上去 Box 的解引用其实在编译器内另有黑魔法，和常规类型的 Deref 不是一回事

### Code

```rust
fn main() {
    let x = 5;
    let y = &5;
    // println!("{}", x == y); 在标准比较或赋值中，rust不会自动应用解引用，因此不能直接比较
    println!("{}, {}, {}", x, y, *y); // 可以自动解引用

    struct MyBox<T>(T);
    impl<T> MyBox<T> {
        fn new(v: T) -> MyBox<T> {
            MyBox(v)
        }
    }

    let x = MyBox::new(1);
    // let y = *x + 1; 还未实现Deref特征，直接使用 `*` 解引用操作符，编译器不知道怎么解析，因此报错

    // 为自定义类型实现Deref特征，变为智能指针
    impl<T> Deref for MyBox<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    // 实现Deref特征后，可以使用 `*` 解引用操作符
    let y = *x + 1;
    // 类型转换，实现Deref特征，转换为值方法调用
    let y = *(Deref::deref(&x)) + 1;

    let s = String::from("value");
    let p = Deref::deref(&s);

    fn display(s: &str) {
        println!("{s}");
    }
    // 实参需要传递引用类型才能触发编译器自动解引用操作
    let s = String::from("Hello World");
    display(&s);
    // 连续解引用操作
    let s = Box::new(String::from("Hello World"));
    display(&s);

    let x = &(*s)[..];
    display(x);

    // 隐式转换和连续隐式转换可以用在赋值过程中
    let s = Box::new(String::from("Hello World"));
    let s1 = &s;
    let s2: &str = &s;
    let s2 = &s as &str;
    let s3 = s.to_string();

    // 自定义指针也能实现连续转换
    let s = MyBox::new(String::from("Hello World"));
    let s1 = &s;
    let s2: &str = &s;
    let s2 = &s as &str;
    let s3 = s.to_string();
    display(&s);

    s1.to_string();

    String::to_string(&s);

    &s.to_string();

    // 实现DerefMut特征，DerefMut的前提是实现了Deref特征
    impl<T> DerefMut for MyBox<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    display(&s); // 不可变引用 Deref 变为不可变引用

    // 可变引用转变为可变引用
    fn display_mut(s: &mut String) {
        s.push_str("world");
        println!("{}", s);
    }
    let mut s = MyBox::new(String::from("Hello World"));
    display_mut(&mut s); // 可变引用通过DerefMut转换为新的可变引用

    display(&mut s); // 可变引用通过Deref转换为新的不可变引用
}
```
