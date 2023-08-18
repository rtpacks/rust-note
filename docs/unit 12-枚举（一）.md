## 枚举 enum

详细介绍

- https://rust-book.junmajinlong.com/ch9/01_enum_basis.html
- https://course.rs/basic/compound-type/enum.html

枚举(Enum)类型通常用来归纳多种可穷举的具体事物，与函数型语言的函数式编程语言中的 代数数据类型类似。简单点说，枚举是一种包含零个、一个或多个具体值的数据类型。枚举类型不能用来描述无法穷举的事物。
例如【整数】虽然包含 0、1、2、......，但这样的值无穷无尽，此时不应该直接用枚举类型，而应该使用具有概括性的方式去描述它们，比如枚举正整数、0、负整数这三种情况，也可以枚举所需的 1、2、3 后，再用一个额外的 Other 来通配所有其他情况。
Rust 支持枚举类型，且 Rust 的枚举类型比其他语言的枚举类型更为强大。

任何类型的数据都可以放入枚举成员中: 例如字符串、数值、结构体甚至另一个枚举。

Rust 的结构体更像是面向对象语言中的接口或父类，它能统一某一种类型的变量，不通过继承做到多态。现代语言推崇使用组合而不是继承。

Java 中，多态的实现是通过实现接口或继承父类来实现的，达到复用的目的。

```java
class Message {
     String message;
}

class Quit extend Message {}
class Move extend Message {}
class Write extend Message {}


String send(Message msg) {
     return msg.message
}
```

在 Rust 中，没有继承这个概念，Rust 推崇使用组合！如以下代码。

```rs
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}
```

解释

- Quit 没有任何关联数据
- Move 包含一个匿名结构体
- Write 包含一个 String 字符串

Rust 中结构体是没有继承这个概念的，所以在实现不同的类型需要多个结构体

```rs
struct QuitMessage; // 单元结构体
struct MoveMessage {
     x: i32,
     y: i32,
}
struct WriteMessage(String); // 元组结构体
struct ChangeColorMessage(i32, i32, i32); // 元组结构体
```

由于每个结构体都有自己的类型，因此我们无法在需要同一类型的地方进行使用，例如某个函数它的功能是接受消息并进行发送，那么用枚举的方式，就可以接收不同的消息，但是用结构体，该函数无法接受 4 个不同的结构体作为参数。
平时常见到的名词，同一化类型说的就是这个道理，将具有类似行为但不同类型的“数据”抽象一层，同一化为某一个类型，能提高代码的复用程度和内聚性。

代码规范角度来看，枚举的实现更简洁，代码内聚性更强，不像结构体的实现，分散在各个地方。

如判断性别

```rs
enum Gender {
     Male,
     Female
}

fn is_male(g: Gender) -> bool {
     // some code
}
```

## 拓展

如果只是声明了枚举具有哪些属性，但是没有指定类型，那枚举的每个值属性会有默认类型吗？

其实，前文定义的枚举类型，其每个成员都有对应的数值。默认第一个成员对应的数值为 0，第二个成员的对应的数值为 1，后一个成员的数值总是比其前一个数值大 1。并且，可以使用=为成员指定数值，但指定值时需注意，不同成员对应的数值不能相同。

```rs
enum E {
     A, // 0
     B, // 1
     C = 32, // 32
     D, // 33，自动根据上一个数值 + 1
}
```

例如快速对应常用的连续值

```rs
enum Week {
   Monday = 1, // 1
   Tuesday,    // 2
   Wednesday,  // 3
   Thursday,   // 4
   Friday,     // 5
   Saturday,   // 6
   Sunday,     // 7
}

fn main(){
  // mon等于1
  let mon = Week::Monday as i32;
}
```

注意：可在 enum 定义枚举类型的前面使用#[repr]来指定枚举成员的数值范围，超出范围后将编译错误。当不指定类型限制时，Rust 尽量以可容纳数据大小的最小类型。例如，最大成员值为 100，则用一个字节的 u8 类型，最大成员值为 500，则用两个字节的 u16。

```rs
// 最大数值不能超过255
#[repr(u8)]  // 限定范围为`0..=255`
  enum E {
  A,
  B = 254,
  C,
  D,        // 256，超过255，编译报错
}
```
