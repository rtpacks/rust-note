## 模式解构赋值

模式匹配谈到过一句话描述模式：模式是一个带有类型**变量**，比较类型的同时也可以比较值

### ref 和 mut 修饰模式中的变量

- https://rust-book.junmajinlong.com/ch10/03_deconstruction.html#ref%E5%92%8Cmut%E4%BF%AE%E9%A5%B0%E6%A8%A1%E5%BC%8F%E4%B8%AD%E7%9A%84%E5%8F%98%E9%87%8F

在 rust 中，变量的所有权是第一个值得关注的点，所以在模式解构中，为了避免进行解构赋值时，
很可能会将变量拥有的所有权转移出去，从而使得原始变量变得不完整或直接失效的情况，有一些特殊的规则需要记住。

```rs
struct Person{
  name: String,
  age: i32,
}

fn main(){
  let p = Person{name: String::from("junmajinlong"), age: 23};
  let Person{name, age} = p;

  println!("{}", name);
  println!("{}", age);
  println!("{}", p.name);  // 错误，name字段所有权已转移
}
```

如果不希望变量失去值的所有权，有以下几个方式

- 方式一：解构表达式的引用 `let Person{name, age} = &p;` 此时的变量是只读的。
- 方式二：解构表达式的克隆，适用于可调用 clone()方法的类型
- 方式三：在模式的某些字段或元素上使用 ref 关键字修饰变量，如果希望变量可变，则使用 mut 关键字。

```rs
let Person{ref name, age} = p;
let Person{name: ref n, age} = p;

let x = 5_i32;         // x的类型：i32
let x = &5_i32;        // x的类型：&i32
let ref x = 5_i32;     // x的类型：&i32
let ref x = &5_i32;    // x的类型：&&i32
```

**在模式中使用 ref 修饰变量名相当于对被解构的字段或元素上使用 `&` 进行引用，ref 就是定义变量是否为引用类型。**
因此，使用 ref 修饰了模式中的变量后，解构赋值时对应值的所有权就不会发生转移，而是以**只读**的方式借用给该变量。

如果想要对解构赋值的变量具有数据的修改权，需要使用 mut 关键字修饰模式中的变量，但这样会转移原值的所有权，此时可不要求原变量是可变的。
如果不想在可修改数据时丢失所有权，可在 mut 的基础上加上 ref 关键字，就像&mut xxx 一样。

ref 与 mut 和&与 mut 的组合是相同的，只是模式解构的 pattern 位置不用&而用 ref 表达属性的引用。

注意，使用 ref 修饰变量只是借用了被解构表达式的一部分值（属性），而不是借用整个值。如果要匹配的是一个引用，则使用&表示。

```rs
let a = &(1,2,3);       // a是一个引用
let (t1,t2,t3) = a;     // t1,t2,t3都是引用类型&i32
let &(x,y,z) = a;       // x,y,z都是i32类型
let &(ref xx,yy,zz) = a;  // xx是&i32类型，yy,zz是i32类型
```

上面几个式子看起来会非常复杂，会担忧如此多的情况该怎么处理。
其实 rust 的模式解构不需要我们去考虑实现方式是什么，只需要我们比对需要的数据格式是什么，这是为了方便使用所有权而设计的。

比如
&(a, b, c) = &(1, 2, 3) 通过比对模式那么 a, b, c 就是 i32, i32, i32，
(a, b, c) = &(1, 2, 3) 通过比对模式，a, b, c 就是 &i32, &i32, &i32

最后，也可以将 match value{} 的 value 进行修饰，例如 match &mut value {}，这样就不需要在模式中去加 ref 和 mut 了。
这对于有多个分支需要解构赋值，且每个模式中都需要 ref/mut 修饰变量的 match 非常有用。
对可变引用进行匹配，利用这个 pattern 将分支的变量变为可变引用，而不是可变变量。前提是可变变量

````rs
fn main() {
  let mut s = "hello".to_string();
  match &mut s {
    // 对可变引用进行匹配，利用这个pattern将分支的变量变为可变引用，而不是可变变量。前提是可变变量。
    // 匹配成功时，变量也是对原数据的可变引用
    x => x.push_str("world"),
  }
  println!("{}", s);
}
```rs

### 匹配守卫(match guard)

匹配守卫允许匹配分支添加额外的后置条件：当匹配了某分支的模式后，再检查该分支的守卫后置条件，如果守卫条件也通过，则成功匹配该分支。

```rs
let x = 33;
match x {
  // 先范围匹配，范围匹配成功后，再检查是否是偶数
  // 如果范围匹配没有成功，则不会检查后置条件
  0..=50 if x % 2 == 0 => {
    println!("x in [0, 50], and it is an even");
  },
  0..=50 => println!("x in [0, 50], but it is not an even"),
  _ => (),
}
````

后置条件的优先级很低。例如：

```rs
// 下面两个分支的写法等价
4 | 5 | 6 if bool_expr => println!("yes"),
(4 | 5 | 6) if bool_expr => println!("yes"),
```

### 解构引用类型

在解构赋值时，如果解构的是一个引用，则被匹配的变量也将被赋值为对应元素的引用。
在解构引用类型的变量时，特别注意 rust 的自动解引用如 `a.b` 中的 a 可以是普通类型，也可以是引用数据类型，否则不容易判断变量的实际类型。

```rs
let t = &(1,2,3);    // t是一个引用
let (t0,t1,t2) = t;  // t0,t1,t2的类型都是&i32
let t0 = t.0;   // t0的类型是i32而不是&i32，因为t.0等价于(*t).0
let t0 = &t.0;  // t0的类型是&i32而不是i32，&t.0等价于&(t.0)而非(&t).0
```

因此，当使用模式匹配语法 for i in t 进行迭代时：

- 如果 t 不是一个引用，则 t 的每一个元素都会 move 给 i（所有权转移）
- 如果 t 是一个引用，则 i 将是每一个元素的引用
- 同理，for i in &mut t 和 for i in mut t 也一样

### 解构解引用类型

当 match VALUE 的 VALUE 是一个解引用*xyz 时(因此，xyz 是一个引用)，可能会发生所有权的转移，
此时可使用 xyz 或&*xyz 来代替\*xyz。具体原因请参考：[解引用(deref)的所有权转移问题](https://rust-book.junmajinlong.com/ch6/05_re_understand_move.html#whenmove)。

```rs
fn main() {
  // p是一个Person实例的引用
  let p = &Person {
    name: "junmajinlong".to_string(),
    age: 23,
  };

  // 使用&*p或p进行匹配，而不是*p
  // 使用*p将报错，因为会转移所有权
  match &*p {
    Person {name, age} =>{
      println!("{}, {}",name, age);
    },
    _ => (),
  }
}

struct Person {
  name: String,
  age: u8,
}
```

### code

```rs
fn main {

    // 解构可能会让原始值失去所有权
    struct Person {
        name: String,
        age: i32,
    }
    let p = Person {
        name: String::from("jockey"),
        age: 12,
    };

    // 失去所有权
    let Person { name, age } = p;
    println!("{name}, {age}");
    // println!("{}", p.name);

    // 使用ref和mut修饰pattern变量
    let (a, b, c) = &(1, 2, 3);
    let &(a, b, c) = &(1, 2, 3);
    // let &(a, b, c) = (1, 2, 3); 这是错误的
    let &(ref a, b, c) = &(1, 2, 3);
    let ref a = 2;

    // 使用mut
    let mut s = "hello".to_string();
    match &mut s {
        // 对可变引用进行匹配，利用这个pattern将分支的变量变为可变引用，而不是可变变量。前提是可变变量。
        // 匹配成功时，变量也是对原数据的可变引用
        x => x.push_str("world"),
    }
    println!("{}", s);

    let p = Person {
        name: String::from("jockey"),
        age: 12,
    };
    // 匹配守卫
    match p {
        // pattern 中 age: 12表示是什么类型，同时也是值，因为一个类型是12的变量，它的值只能是12，这个称为字面量类型
        // 模式是一个带有类型**变量**，比较类型的同时也可以比较值
        Person { name, age: 12 } if 2 == 2 => println!("matched {name}"),
        _ => println!("no match"),
    }

    // 解构引用类型
    let t = &(1, 3, 4);
    let (t0, t1, ..) = t;
    let t0 = t.0; // 注意rust的自动解引用

    // 解构解引用类型
    let p = &Person {
        name: String::from("Jockey"),
        age: 12,
    };
    // 不同的解构类型形成的类型不一样
    let name = &*p.name;
    let name = &p.name;

    // 使用&*p或p进行匹配，而不是*p
    // 使用*p将报错，因为会转移所有权
    match &*p {
        Person { name: String, age } => {
            println!("{}, {}", name, age);
        }
        _ => (),
    }
}
```
