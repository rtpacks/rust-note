use ilearn::{run, Config};
use std::{
    array::IntoIter, collections::HashMap, convert::TryInto, env, error::Error, fmt::Display, fs,
    mem::size_of, ops::Index, process, rc::Rc, sync::Arc,
};

fn main() {
    /*
     * ## 类型转换 - 通用类型转换
     * 虽然 as 和 TryInto 很强大，但是只能应用在数值类型上，因此需要考虑其他方案。
     *
     * 首先看手动转换的代码，如果属性数据量大或者深层嵌套对象时，会非常的麻烦且啰嗦：
     * ```rust
     * struct Foo {
     *     x: u32, y: u16,
     * }
     * struct Bar {
     *     a: u32, b: u16,
     * }
     * fn reinterpret(foo: Foo) -> Bar {
     *     let Foo { x, y } = foo;
     *     Bar { a: x, b: y }
     * }
     * ```
     *
     * ### 强制类型转换
     * 在某些情况下，类型是可以进行隐式强制转换的。虽然这些转换弱化了 Rust 的类型系统，但是它们的存在是为了让 Rust 在大多数场景可以工作，而不是报各种类型上的编译错误。
     *
     * 在强制类型转换中，有一个转换规则：在匹配特征时，不会做任何强制转换(除了方法)。一个类型 T 可以强制转换为 U，不代表 impl T 可以强制转换为 impl U。
     * ```rust
     * trait Trait {}
     * impl<'a> Trait for &'a i32 {}
     *
     * // 注意，trait特征是非固定大小的，rust不允许非固定大小的数据作为参数类型，即 trait 不能直接用作为类型。
     * // fn foo(t: Trait) {}
     * fn foo<T: Trait>(t: T) {}
     *
     * // mut是可以向 immut 变化的，但 immut 大部分情况下是不允许向mut变化的
     * let t_1: &mut i32 = &mut 8;
     * let t_2 = t_1 as &i32;
     *
     * // foo(t_1); 错误的
     * foo(t_2);
     * ```
     * > 注意：
     * > - trait 特征是非固定大小的，rust不允许非固定大小的数据作为参数类型，即 trait 不能直接用作为类型。
     * > - mut 是可以向 immut 变化的，但 immut 大部分情况下是不允许向 mut 变化的
     *
     * 在上面的例子中，`&i32` 实现了特征 `Trait`， `&mut i32` 可以转换为 `&i32`，但是 `&mut i32` 依然无法作为 `Trait` 来使用，也就是即使 T 可以强制转换为 U，也不代表表 `impl T` 可以强制转换为 `impl U`。
     *
     * ### 点操作符
     * 方法调用的点操作符看起来简单，实际上非常不简单，它在调用时，会发生很多**魔法般的类型转换**。例如：自动引用、自动解引用，强制类型转换直到类型能匹配等。
     *
     * 在方法(非函数)签名中，参数 Self 常放在第一个位置，它可被称为**接收器（receiver）**，代表着**调用方法**的实例，它的类型有三种 `self &self &mut self`。
     *
     * 假设 value 拥有类型 T（包括特征 trait 和 结构体 struct 等能作为类型的数据），T 拥有 foo 方法，如果调用 `value.foo()`，编译器在调用 foo 之前，根据完全限定语法和下面的流程来确定到底使用哪个 Self 类型来调用：
     * > 完全限定语法：https://course.rs/basic/trait/advance-trait.html#%E5%AE%8C%E5%85%A8%E9%99%90%E5%AE%9A%E8%AF%AD%E6%B3%95
     * > Deref 特征：https://kaisery.github.io/trpl-zh-cn/ch15-02-deref.html
     * > Index 特征：https://doc.rust-lang.org/std/ops/trait.Index.html
     *
     * 1. 编译器检查它是否可以直接调用 T::foo(value)，即检查类型是否具有foo方法，称之为**值方法调用**
     * 2. 如果值方法调用无法完成(例如方法类型错误或者类型没有对应函数的 Self 进行实现)，那么编译器会尝试**增加自动引用**，会尝试以下调用： `<&T>::foo(value)` 和 `<&mut T>::foo(value)`，称之为**引用方法调用**
     * 3. 如果值方法和引用方法两个方法不工作，编译器会试着**解引用 T** ，然后再进行尝试。这里使用了 `Deref` 特征 —— 若 `T: Deref<Target = U>` (T 可以被解引用为 U)，那么编译器会使用 U 类型进行尝试，称之为**解引用方法调用**
     * 4. 如果 T 不能被解引用，且 T 是一个定长类型(在编译期类型长度是已知的)，那么编译器也会尝试将 T 从**定长类型转为不定长类型**，例如将 [i32; 2] 转为 [i32]
     * 5. 如果以上方式均不成功，那编译器将报错
     *
     * 以下面代码为例，跑一遍流程：
     * ```rust
     * let array: Rc<Box<[T; 3]>> = ...;
     * let first_entry = array[0];
     * ```
     * array 数组的底层数据隐藏在了重重封锁之后，那么编译器如何使用 `array[0]` 这种数组原生访问语法通过重重封锁，准确的访问到数组中的第一个元素？
     *
     * 首先先了解 `array[0]` 只是 Index 特征的语法糖，最终编译器会将 array[0] 转换为 `array.index(0)` 调用。
     * 然后根据点操作符魔法般的类型转换流程，确定 `array.index(0)` 是否能调用，以次确定 `array[0]` 是否能调用。
     *
     * 因此 `array[0]` 能不能转换成 `array.index(0)` 以及 `array.index(0)` 能不能调用成功都依赖于 array 是否实现了 Index 特征。
     * 也就是转换/调用前，需要先检查 array 是否实现了 Index 特征。
     *
     * 1. 编译器检查 `Rc<Box<[T; 3]>>` 是否有实现 Index 特征即是否具有 index 方法，`Index::index(array: Rc<Box<[T; 3]>>)`，结果是否，值方法调用失败。
     * 2. 不仅如此，`&Rc<Box<[T; 3]>>` 与 `&mut Rc<Box<[T; 3]>>` 也没有实现 Index 特征，即没有 index 方法，引用方法调用失败。
     * 3. 值方法和引用方法都失败了，编译器开始对 `Rc<Box<[T; 3]>>` 解引用，把它转变成 `Box<[T; 3]>`，然后对 `Box<[T; 3]>` 再尝试值方法和引用方法。
     * 4. `Box<[T; 3]>`， `&Box<[T; 3]>`，和 `&mut Box<[T; 3]>` 都没有实现 Index 特征，所以编译器开始对 `Box<[T; 3]>` 进行解引用，得到了 [T; 3]，再尝试值方法和引用方法。
     * 5. `[T; 3]` 以及它的各种引用都没有实现 Index 特征，它也不能再进行解引用，最后一种尝试定长变为不定长。(很反直觉:D，在直觉中，数组都可以通过索引访问，实际上只有数组切片才可以!)。
     * 6. 将定长转为不定长，`[T; 3]` 被转换成 `[T]`，也就是数组切片，它实现了 Index 特征，因此 `array: Rc<Box<[T; 3]>>` 可以通过 index 方法访问到对应的元素。
     *
     * ```rust
     * let arrayBox = Box::new([1, 2, 3]);
     * arrayBox.index(0);
     * // Index::index(&arrayBox, 0); 错误，因为 Box<[i32, 3]> 没有实现 Index 特征，所以参数错误，也就是完全限定语法调用失败
     *
     * let array = [1, 2, 3];
     * array.index(0);
     * Index::index(&array, 0);
     * ```
     *
     * #### 意想不到的自动引用
     * 看一个复杂的例子：
     * ```rust
     * fn do_stuff<T: Clone>(value: &T) {
     *     let cloned = value.clone();
     * }
     * ```
     *
     * 按照**点操作符魔法般的转换**跑一遍流程：
     * 1. 编译器检查能不能进行**值方法调用**， value 的类型是 `&T`，类型 Clone 具有 `Clone::clone(&self) -> Self` 方法，value 类型 `&T` 符合 Self 类型 `&self` ，因此可以进行值方法调用，cloned 的类型是 T。
     *
     * 如果去掉 `Clone` 限制，代码变为：
     * ```rust
     * fn do_stuff<T>(value: &T) {
     *     let cloned = value.clone();
     * }
     * ```
     * 直觉上这段代码会报错，因为 T 没有实现 Clone 特征。其实这是能正常运行的代码，易混淆点就在于**点操作符魔法般的转换**。根据流程进行推理：
     * 1. T 没有实现 Clone 特征，因为值方法调用 `Clone::clone(value)` 调用失败
     * 2. 增加自动引用，T 变为 `&T`，此时 `&T` 实现了 `Clone` 类型 (所有的引用类型都可以被复制，其实是复制一份地址)，引用方法调用成功
     * 3. 最终，复制出一份引用指针，`cloned` 的类型是 `&T` 
     *
     * 一个更复杂的自动引用生效的例子：
     * ```rust
     * #[derive(Clone)]
     * struct Container<T>(Arc<T>);
     *
     * fn clone_containers<T>(foo: &Container<i32>, bar: &Container<T>) {
     *     let foo_cloned = foo.clone();
     *     let bar_cloned = bar.clone();
     * }
     * ```
     * 复杂类型派生 Clone 的规则：**一个复杂类型能否派生Clone `#[derive(Clone)]`，取决于它内部的所有子类型是否都实现了 Clone 特征。**
     * 
     * 因此，`Container<T>(Arc<T>)` 是否实现 Clone 的关键在于 T 类型是否实现了 Clone 特征。
     * 
     * 上面例子中，clone_containerrs 函数的第一个参数 `foo: &Container<i32>` 由于 i32 实现了 Clone 特征，所以 `Container<i32>` 派生（实现） Clone 特征，且 foo 满足函数 `Clone::clone(&self) -> Self` 的 Self 接收者的 `&self` 类型，因此编译器可以直接进行值方法调用，`foo.clone() == Clone::clone(foo)`，由此也可以看出 foo_cloned 的类型是 `Container<i32>`。
     * 
     * 而第二个参数 `bar: &Containers<T>` 使用泛型 T，由于泛型 `T` 没有实现 Clone 特征，所以 `Containers<T>` 是没有派生（实现）Clone 特征的。按照点操作符魔法般的转换流程，第一步值方法调用，`Containers<T>` 未实现 Clone 特征，所以调用失败 `Clone::clone(&self) -> Self`，第二步引用方法调用成功，因为引用类型实现了 Clone 特征（所有引用都可以被复制，复制一份地址），因此 bar_cloned 是 `&Container<T>` 类型，是一份指针数据。
     * 
     * ```rust
     * Clone::clone(bar); 值方法调用失败，因为 Containers<T> 未实现 Clone 特征，不含有 clone 方法
     * Clone::clone(&bar); 引用方法调用成功，自动增加引用
     * ```
     * 
     * 在判断点操作符的转换流程时，应该从原有的类型下手，即 `T` 中的方法优先级高于 `&T` 的方法。
     * 
     * 
     * 
     *
     *
     *
     *
     *
     *
     * ### 变形记（transmutes）
     * 阅读：https://course.rs/advance/into-types/converse.html#%E5%8F%98%E5%BD%A2%E8%AE%B0transmutes
     *
     */

    trait Trait {}
    impl<'a> Trait for &'a i32 {}

    // 注意，trait特征是非固定大小的，rust不允许非固定大小的数据作为参数类型，即trait 不能直接用作为类型。
    // fn foo(t: Trait) {}
    fn foo<T: Trait>(t: T) {}

    // mut是可以像unmut变化的，但unmut大部分情况下是不允许像mut变化的
    let t_1: &mut i32 = &mut 8;
    let t_2 = t_1 as &i32;

    // foo(t_1); 错误的
    foo(t_2);

    #[derive(Debug)]
    struct Person {
        age: u8,
    }
    impl Person {
        fn get_age(&self) -> u8 {
            self.age
        }
    }
    println!(
        "age = {}, age = {}",
        Person { age: 12 }.get_age(),
        Person::get_age(&Person { age: 18 })
    );

    let arrayBox = Box::new([1, 2, 3]);
    arrayBox.index(0);
    // Index::index(&arrayBox, 0); 错误，因为Box<[i32, 3]> 没有实现Index特征，所以参数错误

    let array = [1, 2, 3];
    array.index(0);
    Index::index(&array, 0);
    Clone::clone(&array);

    fn do_stuff<T>(t: &T) -> &T {
        let cloned = t.clone();
        cloned
    }
    let num = do_stuff(&9);
    let person = do_stuff(&Person { age: 18 });
    println!("{:#?}", person);

    #[derive(Clone)]
    struct Container<T>(Arc<T>);

    fn clone_containers<T>(foo: &Container<i32>, bar: &Container<T>) {
        let foo_cloned = foo.clone();
        let bar_cloned = bar.clone();
    }

    let eight = &8;
    let eight_cloned = Clone::clone(&eight);
}
