## 生命周期标注语法

前面提到，一般为了方便表达和使用，如函数调用的生命周期在不同的调用位置（参数）不一样，不会使用行这种概念来表达而是使用生命周期标注。
生命周期标注以 `'` 开头，名称往往是一个单独的小写字母，大多数人都用 `'a` 来作为生命周期的名称。

如果是引用类型的参数，那么生命周期会位于引用符号 & 之后，并用一个空格来将生命周期和引用参数分隔开。
为什么特别指引用类型呢？因为非引用类型数据的所有权都在自身上，不会出现悬垂引用的问题。
同时要特别注意的是：**生命周期标注只是标注，只是帮助编译器推理生命周期，一些错误的标注编译器是不会接受，会报错的。**

```rs
&i32        // 一个引用
&'a i32     // 具有显式生命周期的引用
&'a mut i32 // 具有显式生命周期的可变引用
```

### 1. 函数签名中的生命周期标注

```rs
fn useless<'a>(first: &'a i32, second: &'a i32) {}
```

需要注意的点如下：

- 和泛型一样，使用生命周期参数，需要先声明 <'a>
- x、y 和返回值至少活得和 'a 一样久(因为返回值要么是 x，要么是 y)

```rs
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
   if x.len() > y.len() {
       x
   } else {
       y
   }
}
```

该函数签名表明对于某些生命周期 'a，函数的两个参数都至少跟 'a 活得一样久，同时函数的返回引用也至少跟 'a 活得一样久。
实际上，这意味着**返回值的生命周期与参数生命周期中的较小值一致**。
虽然两个参数的生命周期都是标注了 'a，但是实际上这两个参数的真实生命周期可能是不一样的(生命周期 'a 不代表生命周期等于 'a，而是大于等于 'a)。

#### 深入思考生命周期标注

函数的返回值如果是一个引用类型，那么它的生命周期只会来源于：

- 函数参数的生命周期
- 函数体中某个新建引用的生命周期

若是后者情况，就是典型的悬垂引用场景，引用函数内部的变量，函数运行结束后变量被释放，但返回了引用，可能导致悬垂引用。
这种情况最好的办法就是返回内部字符串的所有权，然后把字符串的所有权转移给调用者。

总结：**在通过函数签名指定生命周期参数时，我们并没有改变传入引用或者返回引用的真实生命周期，而是告诉编译器当不满足此约束条件时，就拒绝编译通过。
生命周期语法用来将函数的多个引用参数和返回值的作用域关联到一起，一旦关联到一起后，Rust 就拥有**充分的信息\*\*来确保我们的操作是内存安全的。

### 2. 结构体的生命周期

不仅仅函数具有生命周期，结构体其实也有这个概念。非引用数据类型在结构体初始化时，只要转移所有权即可，而引用是有特殊限制的。

结构体中使用引用很简单，只要为结构体中的每一个引用标注上生命周期。结构体的生命周期标注语法跟泛型参数语法很像，需要对生命周期参数进行声明 <'a>。

```rs
struct ImportantExcerpt<'a> { part: &'a str }

let i;
{
   let novel = String::from("Call me Ishmael. Some years ago...");
   let first_sentence = novel.split('.').next().expect("Could not find a '.'");
   i = ImportantExcerpt {
       part: first_sentence,
   };
}
println!("{:?}",i); // 报错
```

这个结构体有一个字段，part，它存放了一个字符串 slice，这是一个引用。
类似于泛型参数类型，必须在结构体名称后面的尖括号中声明泛型生命周期参数，以便在结构体定义中使用生命周期参数。有两种表示：

- 结构体 ImportantExcerpt 所引用的字符串 str 必须比该结构体活得更久。
- 结构体 ImportantExcerpt 的实例不能比其 part 字段中的引用存在的更久。

最后的 println 表示，无论什么对象，在什么时候都需要**保证依赖有效。**

### 3. 生命周期省略/生命周期消除（Lifetime Elision）

实际上，对于编译器来说，每一个引用类型都有一个生命周期，那么为什么我们在使用过程中，很多时候无需标注生命周期？例如

```rs
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();

     for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[0..i];
        }
    }

     &s[..]
}
```

该函数的参数和返回值都是引用类型，尽管我们没有显式的为其标注生命周期，编译依然可以通过。其实原因不复杂，编译器为了简化用户的使用，运用了生命周期消除大法。
对于 first_word 函数，它的返回值是一个引用类型，那么该引用只有两种情况：

- 从参数获取
- 从函数体内部新创建的变量获取

如果是后者，就会出现悬垂引用，最终被编译器拒绝，因此只剩一种情况：返回值的引用是获取自参数，这就意味着参数和返回值的生命周期是一样的。因此，就算我们不标注生命周期，也不会产生歧义，编译器通过编译。
实际上，在 Rust 1.0 版本之前，这种代码果断不给通过，因为 Rust 要求必须显式的为所有引用标注生命周期：

```rs
fn first_word<'a>(s: &'a str) -> &'a str {
```

后来经过实践，Rust 团队发现在特定情况下 Rust 开发者们总是重复地编写一模一样的生命周期标注。这些场景是可预测的并且遵循几个明确的模式。
接着 Rust 团队就把这些模式编码进了 Rust 编译器中，如此借用检查器在这些情况下就能推断出生命周期而不再强制开发者显式的增加标注。

被编码进 Rust 引用分析的模式被称为 生命周期省略（消除）规则（lifetime elision rules）。这些规则是一系列特定的场景，此时编译器会考虑代码如果符合这些场景，就无需明确指定生命周期。

消除规则中，有两点需要注意：

- 消除规则不是万能的，若编译器不能确定某件事是正确时，会直接判为不正确，此时需要手动标注生命周期。
- 函数或者方法中，参数的生命周期被称为 **输入生命周期**，返回值的生命周期被称为 **输出生命周期**。

编译器采用三条规则来判断引用何时不需要明确的标注。第一条规则适用于输入生命周期，第二、三条规则适用于输出生命周期。
如果编译器检查完这三条规则后仍然存在没有计算出生命周期的引用，编译器将会停止并生成错误。

1. **每一个引用参数都有独自的生命周期。**
   例如一个引用参数的函数就有一个生命周期标注: fn foo<'a>(x: &'a i32)，两个引用参数的有两个生命周期标注:fn foo<'a, 'b>(x: &'a i32, y: &'b i32), 依此类推。
2. **若只有一个输入生命周期(函数参数中只有一个引用类型)，那么该生命周期会被赋给所有的输出生命周期。** 也就是所有返回值的生命周期都等于该输入生命周期。
   例如函数 fn foo(x: &i32) -> &i32，x 参数的生命周期会被自动赋给返回值 &i32，因此该函数等同于 fn foo<'a>(x: &'a i32) -> &'a i32。
3. **若存在多个输入生命周期，且其中一个是 &self 或 &mut self，则 &self 的生命周期被赋给所有的输出生命周期。**
   拥有 &self 形式的参数，说明该函数是一个 方法，该规则让方法的使用便利度大幅提升。

如假设实际项目中代码与应用规则对比

```rs
fn longest(x: &str, y: &str) -> &str { // 实际项目中的手写代码
```

首先，编译器会应用第一条规则，为每个参数都标注生命周期：

```rs
fn longest<'a, 'b>(x: &'a str, y: &'b str) -> &str { // 第一条规则，为每个引用参数生成独立的生命周期
```

第二条规则却无法被使用，因为输入生命周期有两个。第三条规则也不符合，因为它是函数，不是方法，因此没有 &self 参数。
在套用所有规则后，编译器依然无法为返回值标注合适的生命周期，因此，编译器就会报错，提示我们需要手动标注生命周期。

### 4. 方法中的生命周期

方法中的生命周期和生命周期消除规则有很大的关联，其中有几点需要注意的：

- impl 中必须使用结构体的完整名称，包括 <'a>，因为**生命周期标注也是结构体类型的一部分！**
- 方法签名中，往往不需要标注生命周期，得益于生命周期消除的第一和第三规则（存在多个输入生命周期，其中一个为&self 或者&mut self，则&self 的生命周期被赋给所有的输出生命周期）。

第三规则会给所有的输出生命周期赋予&self 的生命周期，那么需要自定义生命周期呢？这需要满足一定的条件，首先是业务代码：

```rs
struct ImportantExcerpt<'a> { part: &'a str };

 impl<'a> ImportantExcerpt<'a> {
    fn announce_and_return_part(&self, announcement: &str) -> &str {
        println!("Attention please: {}", announcement);
        self.part
    }
}
```

这段代码能通过第一规则和第三规则的校验，所以可以通过编译，首先是第一规则，为每个引用参数添加独立的生命周期标注。
需要注意的是，编译器不知道 announcement 的生命周期到底多长，因此它无法简单的给予它生命周期 'a，而是重新声明了一个全新的生命周期 'b。

```rs
impl<'a> ImportantExcerpt<'a> {
    fn announce_and_return_part<'b>(&'a self, announcement: &'b str) -> &str {
        println!("Attention please: {}", announcement);
        self.part
    }
}
```

接着，编译器应用第三规则，将 &self 的生命周期赋给返回值 &str：

```rs
impl<'a> ImportantExcerpt<'a> {
    fn announce_and_return_part<'b>(&'a self, announcement: &'b str) -> &'a str {
        println!("Attention please: {}", announcement);
        self.part
    }
}
```

可以发现，业务代码中尽管我们没有给方法标注生命周期，但是在第一和第三规则的配合下，编译器依然可以通过编译。
回答刚开始的问题，如果需要自定义生命周期呢，需要满足什么条件？实现方法比想象中简单：加一个约束，就能暗示编译器引用的内容是不会出现问题的。

生命周期约束语法和泛型约束非常相似，有两种形式：

- 'a: 'b，是生命周期约束语法，用于说明 'a 必须比 'b 活得久
- 可以把 'a 和 'b 都在同一个地方声明（如上），或者分开声明但通过 where 'a: 'b 约束生命周期关系

回到生命名周期中，我们提到过一句话，**保证依赖有效**，只需要保证自定义的输出生命周期内，&self 生命周期是有效的就能保证依赖有效。
换句话说：&self 生命周期包含自定义的生命周期。这个道理很简单，`self.part` 是 part 依赖于 self，self 的生命周期比 part 大才不会出现悬垂引用问题。

### 5. 静态生命周期

在 Rust 中有一个非常特殊的生命周期，那就是 'static，拥有该生命周期的引用可以和整个程序活得一样久。
字符串字面量是被硬编码进 Rust 的二进制文件中，因此这些字符串变量全部具有 'static 的生命周期：

```rs
let s: &'static str = "I have a static lifetime";
```

当生命周期不知道怎么标时，对类型施加一个静态生命周期的约束 T: 'static 是不是可以解决问题？
在不少情况下，'static 约束确实可以解决生命周期编译不通过的问题，但是问题来了：本来该引用没有活那么久，但是你非要说它活那么久，万一引入了潜在的 BUG 怎么办？

因此，遇到因为生命周期导致的编译不通过问题，首先想的应该是：**是否是我们试图创建一个悬垂引用，或者是试图匹配不一致的生命周期，而不是简单粗暴的用 'static 来解决问题。**
有时候，'static 确实可以帮助我们解决非常复杂的生命周期问题甚至是无法被手动解决的生命周期问题，那么此时就应该放心大胆的用，只要你确定：**你的所有引用的生命周期都是正确的，只是编译器太笨不懂罢了。**

总结下：

- 生命周期 'static 意味着能和程序活得一样久，例如字符串字面量和特征对象
- 实在遇到解决不了的生命周期标注问题，可以尝试 T: 'static，有时候它会给你奇迹
- 字符串字面量是'static 的生命周期，在分析生命周期时需要明确标出字符串字面量，否则容易混淆

### 一个复杂的例子

```rs
use std::fmt::Display;
fn longest_with_an_announcement<'a, T>(
    x: &'a str,
    y: &'a str,
    ann: T,
) -> &'a str
where
    T: Display,
{
    println!("Announcement! {}", ann);
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

```rs
struct ImportantExcerpt<'a> {
   part: &'a str,
}
impl<'a> ImportantExcerpt<'a> {
   fn announce_and_return_part(&'a self, announcement: &'a str) -> &'a str {
       println!("Attention please: {}", announcement);
       self.part
   }
}

let a = ImportantExcerpt { part: "part" };
let mut res = "字符串是 'static 生命周期";
{
   let announcement = "hello world".to_string();
   // res = a.announce_and_return_part(&announcement); error: `announcement` does not live long enough
   // 编译器只选最短的生命周期那一个，所以即使self和announcement都标注 'a 生命周期在函数内部没有问题，但是最终返回的生命周期是最短的那个生命周期。
   // 很明显，announcement 活得最短，编译器选了 announcement 的生命周期。
   // 同时 announcement 活的时间没有返回值 res 的时间长，编译器就直接报错了：announcement does not live long enough
}
println!("{}", res);
```

编译器只选最短的生命周期那一个，所以即使 self 和 announcement 都标注 'a 生命周期在函数内部没有问题，但是最终返回的生命周期是最短的那个生命周期。
在函数内部没有问题，但是函数运行完成，被其他变量接收后，可能就会发生问题。

### 谈论

- https://github.com/sunface/rust-course/discussions/609#discussioncomment-2868469

### 阅读

- https://www.zhihu.com/question/435470652
- https://rustwiki.org/zh-CN/book/ch10-03-lifetime-syntax.html#%E6%B7%B1%E5%85%A5%E7%90%86%E8%A7%A3%E7%94%9F%E5%91%BD%E5%91%A8%E6%9C%9F
- https://course.rs/basic/lifetime.html#%E7%94%9F%E5%91%BD%E5%91%A8%E6%9C%9F%E6%A0%87%E6%B3%A8%E8%AF%AD%E6%B3%95


### code
```rs
fn main {
    // 悬垂引用示例
    let mut x = &0;
    {
        let y = 1;
        x = &y;
        println!("{x}");
    }
    // println!("{x}"); error

    // fn longestE(x: &str, y: &str) -> &str {
    //     if x.len() > y.len() {
    //         x
    //     } else {
    //         y
    //     }
    // }

    // 函数内的生命周期
    fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
        if x.len() > y.len() {
            x
        } else {
            y
        }
    }
    println!("{}", longest("测试数据1", "测试数据2"));

    // 只需要标注关联的参数生命周期
    fn longest2<'a>(x: &'a str, y: &str) -> &'a str {
        x
    }
    println!("{}", longest2("x", "y"));

    // 结构体的生命周期标注
    #[derive(Debug)]
    struct Person<'a> {
        name: &'a str,
    }
    let p = Person { name: "Jack" };
    println!("{:#?}", p);

    // 在符合三条消除规则的前提下，可以不用手动标注生命周期
    fn first_word(s: &str) -> &str {
        let bytes = s.as_bytes();
        for (i, &item) in bytes.iter().enumerate() {
            if b' ' == item {
                return &s[0..i];
            }
        }
        &s[..]
    }
    println!("{}", first_word("Hello World"));

    // 复杂的生命周期标注
    fn speak_word<'a, T>(x: &'a str, y: &'a str, keywords: T) -> &'a str
    where
        T: Display,
    {
        println!("{}", keywords);

        if x.len() > y.len() {
            x
        } else {
            y
        }
    }
    speak_word("x", "y", "keywords");

    // 使用返回的生命周期
    struct ImportantExcerpt<'a> {
        part: &'a str,
    }
    impl<'a> ImportantExcerpt<'a> {
        fn announce_and_return_part(&'a self, announcement: &'a str) -> &'a str {
            println!("Attention please: {}", announcement);
            self.part
        }
    }

    let a = ImportantExcerpt { part: "part" };
    let mut res = "字符串是 'static 生命周期";
    {
        let announcement = "hello world".to_string();
        // res = a.announce_and_return_part(&announcement); error: `announcement` does not live long enough
        // 编译器只选最短的生命周期那一个，所以即使self和announcement都标注 'a 生命周期在函数内部没有问题，但是最终返回的生命周期是最短的那个生命周期。
        // 很明显，announcement 活得最短，编译器选了 announcement 的生命周期。
        // 同时 announcement 活的时间没有返回值 res 的时间长，编译器就直接报错了：announcement does not live long enough
    }
    println!("{}", res);
}
```