## 模块 Module

一个 `src/lib.rs` 或 `src/main.rs` 就是一个包 Crate，一个包可能是多种功能的集合体。为了项目工程（Package）的组织维护，需要对包进行拆分。

模块 Module（mod）是**rust 代码的构成单元**，是代码拆分的单位（文件/文件夹）。
使用模块可以将包中的代码按照功能性进行重组，而不需要将所有代码都写在 `src/lib.rs` 或者 `src/main.rs`，最终实现更好的可读性及易用性。
同时，使用 mod 还可以非常灵活地去控制代码的可见性，进一步强化 Rust 的安全性。

以 lib 的包（lib crate）为例，该包（crate）的入口在 `src/lib.rs`，也是包的根。在 `src/lib.rs`` 里定义模块（mod）非常简单：

```rs
mod mymod {
    fn test() {
        println!("test");
    }
}
```

但实际项目中不可能将所有的模块（mod）都放在 lib.rs 文件（包的根），而是会将代码按功能等拆分为多个模块（mod）。

### 1. 模块拆分

**一般来说，一个文件都会被视为一个 mod，而且 mod 可以嵌套定义。嵌套定义的 mod 既可以写在同一个文件里，也可以通过文件夹的形式来实现。**

以餐馆为例，使用 `cargo new --lib restaurant` 创建一个餐馆，注意这里创建的是一个库类型的 Package，然后将以下代码放入 src/lib.rs 中：

```rs
// 餐厅前厅，用于吃饭
mod front_of_house {
    // 招待客人
    mod hosting {
        fn add_to_waitlist() {}
        fn seat_at_table() {}
    }
    // 服务客人
    mod serving {
        fn take_order() {}
        fn serve_order() {}
        fn take_payment() {}
    }
}
```

以上的代码创建了三个模块，有几点需要注意的：

- 使用 mod 关键字来创建新模块，后面紧跟着模块名称
- 模块可以嵌套，这里嵌套的原因是招待客人和服务都发生在前厅
- 模块中可以定义各种 Rust 类型，例如函数、结构体、枚举、特征等
- 所有模块均定义在同一个文件中
  类似上述代码中所做的，使用模块就能将功能相关的代码组织到一起，然后通过一个模块名称来说明这些代码为何被组织在一起。

`src/main.rs` 和 `src/lib.rs` 被称为包的根（ crate 根），如此称呼的原因是，这两个文件中**任意一个的内容**都可以构成名为 crate 的模块。
该模块位于包的树形结构(由模块组成的树形结构)的根部（"at the root of the crate’s module structure"）。
这种树形结构展示了模块之间彼此的嵌套关系，因此被称为**模块树**，`src/main.rs`和`src/lib.rs`文件的内容组成了一个**虚拟的模块**，模块的名称就是 `crate`。

```sh
crate
 └── front_of_house
     ├── hosting
     │   ├── add_to_waitlist
     │   └── seat_at_table
     └── serving
         ├── take_order
         ├── serve_order
         └── take_payment
```

注意：以上树形结构中的各个`fn`不是模块，而是模块的一部分，这里为了表现模块树的结构将其展示出来。

#### 父子模块

如果模块 A 包含模块 B，那么 A 是 B 的父模块，B 是 A 的子模块。如 front_of_house 是 hosting 和 serving 的父模块，反之后两者是前者的子模块。

### 2. 路径与引用

模块树的结构和计算机上文件系统目录树非常相似，不仅仅是组织结构上的相似，就连使用方式都很相似：每个文件都有自己的路径，用户可以通过这些路径使用它们。在 Rust 中也是通过路径的方式来引用模块。

路径有两种形式：

- 绝对路径（absolute path）从 crate 根部开始，以 crate 名或者字面量 crate 开头
- 相对路径（relative path）从当前模块开始，以 self、super 或当前模块的标识符开头
  绝对路径和相对路径都后跟一个或多个由双冒号（::）分割的标识符。

如果读者有前端项目经验，绝对路径的导入就类似于路径别名`@/`，相对路径则类似`./`形式。
rust 用`crate`表示包的根（create root），用 `self` 表示当前 `./`，用 `super` 表示上级 `../`，用 `::` 表示下级 `./xx`。

继续拓展 `restaurant` 餐馆例子，给 `src/lib.rs` 加入函数 `eat_at_restaurant` ：

```rs
mod front_of_house {
    // 招待客人
    pub mod hosting {
        pub fn add_to_waitlist() {}
        fn seat_at_table() {}
    }
    // 服务客人
    mod serving {
        fn take_order() {}
        fn serve_order() {}
        fn take_payment() {}
    }
}

pub fn eat_at_restaurant() {
    // 绝对路径
    crate::front_of_house::hosting::add_to_waitlist();
    // 相对路径
    front_of_house::hosting::add_to_waitlist();
}
```

#### 绝对路径

因为 eat_at_restaurant 和 add_to_waitlist 都定义在一个包中（lib crate），因此在绝对路径引用时可以直接以 crate 开头，然后逐层引用，每一层之间使用 `::` 分隔：

```rs
crate::front_of_house::hosting::add_to_waitlist();
```

#### 相对路径

1. 在 `restaurant` 的代码示例中，可以直接访问当前包内的模块，而不需要绝对路径：

```rs
front_of_house::hosting::add_to_waitlist();
```

2. 除了直接访问包内模块，相对路径还可以使用 `self`、`super`、`crate` 访问其他的模块，注意需要在库类型的包（ `lib crate src/lib.rs` ）中测试：

```rs
fn cleanTable() {}

mod front_of_house {
    // 招待客人
    pub mod hosting {
        pub fn add_to_waitlist() {}
        fn seat_at_table() {
             super::clean(); // 调用父模块的方法
             self::add_to_waitlist(); // 调用自身模块的方法
        }
    }
    // 服务客人
    mod serving {
        fn take_order() {}
        fn serve_order() {}
        fn take_payment() {}
    }
    fn clean() {
         crate::cleanTable(); // 调用的是crate包中的方法
         super::cleanTable(); // 同样可以使用super调用父级模块的方法
    }
}
```

`clean` 方法可以使用 `super` 的原因是：在之前提到过，`src/lib.rs` 和 `src/main.rs` 的任一文件的内容都可以组成名称为 `crate` 的虚拟模块，子模块调用父模块的方法使用 `super`。

#### 绝对路径还是相对路径？

如果不确定哪个好，你可以考虑优先使用绝对路径，因为调用的地方和定义的地方往往是分离的，而定义的地方较少会变动。

### 3. 代码可见性

在路径引用中，加入了`pub`关键字避免无法访问模块，这是因为 Rust 出于安全的考虑，默认情况下所有的类型都是私有化的，包括函数、方法、结构体、枚举、常量，就连模块本身也是私有化的。
如果希望被外部访问，那么需要给指定的项加上 `pub` 关键字。
值得注意的是：虽然父模块完全无法访问子模块中的私有项，但是**子模块却可以访问父模块、祖父模块等上级模块的私有项。**

```rs
mod front_of_house {
    pub fn clean() {
         super::cleanTable(); // 模块内的项用pub声明的项才可以被外部访问，但内部的子项访问父、祖父的私有项不受 `pub` 的限制
    }
}
```

### 4. 结构体和枚举的可见性

为何要把结构体和枚举的可见性单独拿出来呢？因为结构体和枚举的成员字段拥有完全不同的可见性：

- 将结构体设置为 pub，但它的所有字段依然是私有的
- 将枚举设置为 pub，它的所有字段也将对外可见

原因在于，枚举和结构体的使用方式不一样。如果枚举的成员对外不可见，那该枚举将一点用都没有，因此枚举成员的可见性自动跟枚举可见性保持一致，这样可以简化用户的使用。

而结构体的应用场景比较复杂，其中的字段也往往部分在 A 处被使用，部分在 B 处被使用，因此无法确定成员的可见性，那索性就设置为全部不可见，将选择权交给程序员。

### 5. 将模块拆分到不同的文件

在模块拆分时提到过，嵌套的模块（Module mod）不仅可以在一个文件，也可以通过文件夹（多文件）的形式管理。

#### 1. 拆分成文件

现在将模块 `front_of_house` 从 `src/lib.rs` 中抽出来作为一个 `src/front_of_house.rs` 模块文件：

```rs
// 招待客人
pub mod hosting {
    pub fn add_to_waitlist() {}
    fn seat_at_table() {
         super::clean(); // 调用父模块的方法
         self::add_to_waitlist(); // 调用自身模块的方法
    }
}
// 服务客人
mod serving {
    fn take_order() {}
    fn serve_order() {}
    fn take_payment() {}
}
fn clean() {
     crate::cleanTable(); // 调用的是crate包中的方法
     super::cleanTable(); // 同样可以使用super调用父级模块的方法
}
```

抽离单独文件后还需要关联原有的关系，让 rust 编译器知道抽离的模块文件属于当前模块的子模块。在 `src/lib.rs` 中用 `mod` 声明：

```rs
mod front_of_house; // 声明 front_of_house 属于子模块，编译时编译器会将子模块的代码插入到此位置

pub fn eat_at_restaurant() {
    // 绝对路径
    crate::front_of_house::hosting::add_to_waitlist();
    // 相对路径
    front_of_house::hosting::add_to_waitlist();
}
```

`mod front_of_house;` 用 `mod` 声明子模块，后面的名称就是模块对应的文件名，同时也是模块的名称！

因此在 rust 项目的代码中，不仅可以看到直接用 `mod` 声明模块的写法，也可以看到用 `mod` 加载同名模块文件的写法。

#### 2. 拆分成文件夹

一个模块可能不只有一个子模块，如 `restaurant` 这个虚拟模块不只有 `front_of_house`，还有 `back_of_house`。
同样的 `front_of_house` 也可能有自己的多个子模块，如果子模块都存放在与父模块同级的目录下，势必会造成管理混乱。
比如祖父模块、父模块、子模块都是单独的模块文件放在同一目录下，对开发人员来说难以管理。
在这种情况下，需要将不同层级的子模块存放在不同层级的文件夹中，形成模块文件树，也就是：**将模块作为一个目录进行管理。**

用文件夹管理模块可以分为两种形式：

1. 文件夹包含模块的入口文件（文件夹完全管理模块）

文件夹完全管理模块，需要在模块文件夹内创建一个 mod.rs 入口文件。在 rustc 版本 1.30 之前，这是文件夹管理模块唯一的形式：

```sh
src
 │─ lib.rs
 └─ front_of_house
     │─ hosting.rs
     │─ mod.rs // 入口文件
     └─ serving.rs
```

2. 文件夹不包含模块的入口文件（文件夹不完全管理模块）

在文件夹完全管理模块的形式中，每一个模块文件夹都定义 `mod.rs` 入口文件会造成大量同名的 mod.rs 文件，所以将入口文件 `mod.rs` 提出来并重命名为模块目录名称：

```sh
src
 │─ lib.rs
 │─ front_of_house.rs // 入口文件，与模块目录名称相同
 └─ front_of_house
     │─ hosting.rs
     └─ serving.rs
```

细心观察就可以发现，文件夹不完全管理模块只是将子模块放入文件夹并在入口文件声明子模块关系，与模块文件的形式非常像。
两者之间的最大区别在于：**文件夹不完全管理模块将所有的子模块放入与模块同名的文件夹中管理，而模块文件形式是所有子模块放在与父模块同级的层级下。**

### code

更多 code 参考 `src/lib.rs`、`src/front_of_house`、`src/back_of_house`

```rs
fn main {
    // 餐厅前厅，用于吃饭
    mod front_of_house {
        fn clean() {}

        // 招待客人
        pub mod hosting {
            pub fn add_to_waitlist() {}

            fn seat_at_table() {}
        }
        // 服务客人
        mod serving {
            fn take_order() {}
            fn serve_order() {}
            fn take_payment() {}
        }
    }

    front_of_house::hosting::add_to_waitlist()
}
```
