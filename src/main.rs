use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Cell, RefCell},
    os::raw,
    ptr,
    rc::{Rc, Weak},
};

use ilearn::{run, Config};

fn main() {
    /*
     * ## 结构体的自引用
     *
     * 在 JavaScript/TypeScript 中，自引用很常见，例如 `this`：
     *
     * ```typescript
     * interface Person {
     *      name: string,
     *      nickname: string,
     * }
     * const person: Person = {
     *      name: "M",
     *      nickname: "N",
     * }
     * person.nickname = name;
     * ```
     *
     * 在没有所有权机制的语言、特别是带GC自动引用的语言中，自引用非常简单。而在具有所有权机制的 rust，自引用是一个非常困难的问题。
     *
     * ```rust
     * struct SelfRef<'a> {
     *     value: String,
     *     pointer_to_value: &'a str, // 该引用指向上面的value
     * }
     *
     * let s = String::from("Hello World");
     * let selfRef = SelfRef {
     *      value: s,
     *      pointer_to_value: &s
     * }
     * ```
     *
     * 由于转移所有权和使用借用同时发生，不符合借用规则，最后编译报错。
     *
     * rust 中有几种解决这种问题的方案，如类似解决循环引用的组合 `Rc + RefCell`，又或者绕过借用规则的 unsafe 操作，但最终**最好的方式是不使用自引用结构体**。
     *
     * https://course.rs/advance/circle-self-ref/self-referential.html#rc--refcell-%E6%88%96-arc--mutex
     *
     * ### Rc + RefCell
     * 在循环引用章节中，为了方便值的初始化，选择 enum 枚举来定义空值状态，现在改成 Option 以便定义空值状态。
     *
     * **第一步**
     * 因为节点的 prev、next 可以指向任意一个节点，所以这两个属性应该是 `RefCell<Rc<Node>>` 类型，而不是 `Rc<RefCell<Node>>`。
     * 这是因为根据**内部可变性**， `RefCell<Rc<Node>>` 可以更改 `Rc<Node>` 的内容，而 `Rc<RefCell<Node>>` 不能更改指向的节点，只能修改节点信息。
     *
     * **第二步**
     * `RefCell<Rc<Node>>` 组合在链表状态下，很有可能形成循环强引用进而触发 OOM，为了避免循环强引用，将 prev 改为 Weak，利用 Weak 阻断循环强引用。
     * `next: RefCell<Rc<Node>>` 和 `prev: RefCell<Weak<Node>>`
     *
     * **第三步**
     * 初始化 Node 结构体时，prev 和 next 不可能永远有值，因此需要定义一个空值。此时 prev 和 next 的定义应该是：一个能够指向任意节点或指向空的数据。
     * prev 和 next 指针可以指向任意节点或指向空，意味着 prev 和 next 是可变的。
     *
     * 因此，为了避免所有权的 mutable 的特性（整体可变，要求内部所有字段都可变）需要把类型定义为 `RefCell<Option<Rc<Node>>>`，而不是 `Option<RefCell<Rc<Node>>>`。
     * 因为 `Option<T>` 涉及到所有权的 mutable 特性，当需要更改指向节点时，所有权变更/管理非常麻烦，而 `RefCell<T>` 因为内部可变性将会很简单。
     * 同理，不能用 `Rc` 包裹 `RefCell`，而是 `RefCell` 包裹 `Rc`。
     *
     * 因此，next 的类型为 `RefCell<Option<Rc<Node>>>`，prev 的类型为 `RefCell<Option<Weak<Node>>>`
     *
     * ```rust
     * #[derive(Debug)]
     * struct Node {
     *     value: i32,
     *     prev: RefCell<Option<Weak<Node>>>,
     *     next: RefCell<Option<Rc<Node>>>,
     * }
     *
     * let a = Rc::new(Node {
     *     value: 1,
     *     prev: RefCell::new(None),
     *     next: RefCell::new(None),
     * });
     * let b = Rc::new(Node {
     *     value: 2,
     *     prev: RefCell::new(None),
     *     next: RefCell::new(None),
     * });
     *
     * // 将 a 的 next 指向 b
     * *a.next.borrow_mut() = Some(Rc::clone(&b));
     * // 将 b 的 prev 指向 a
     * *b.prev.borrow_mut() = Some(Rc::downgrade(&a));
     * println!("{:#?}", a);
     * ```
     *
     * 当使用到自引用时，只需要复制 `Rc` 指针即可。
     *
     * `Rc + RefCell` 虽然可以解决问题，但是增加了许多类型标识，可读性受到很大的影响。
     *
     * ### unsafe 操作
     * 既然自引用受到借用规则的限制，那么可以通过绕过借用规则来实现自引用。绕过借用规则的最简单方式就是通过 unsafe。
     *
     * unsafe 中不操作 rust 的引用，而是直接存储/操作**裸指针**（原始指针），不再受到 Rust 借用规则和生命周期的限制，实现起来非常清晰、简洁。
     *
     * ```rust
     * // 通过 unsafe 操作裸指针实现自引用
     * struct SelfRef {
     *     value: String,
     *     pointer_to_value: *const String, // 该裸指针指向上面的 value
     * }
     *
     * let mut selfRef = SelfRef {
     *     value: String::from("Hello World"),
     *     pointer_to_value: ptr::null(),
     * };
     * // 直接存储裸指针信息，引用转换裸指针需要类型标注，否则就是rust引用
     * selfRef.pointer_to_value = &selfRef.value;
     * // 操作裸指针取值时需要unsafe绕过借用规则和生命周期检查
     * let pointer_to_value = unsafe { &(*selfRef.pointer_to_value) };
     * println!("{}, {}", selfRef.value, pointer_to_value);
     * ```
     *
     * `ptr::null()` 是不可变裸指针类型的空值，除了 `*const` 不可变裸指针类型外，还有一种 `*mut` 可变裸指针类型：
     * ```rust
     * // *mut 可变裸指针
     * struct SelfRefMut {
     *     value: String,
     *     pointer_to_value: *mut String, // 该裸指针指向上面的 value
     * }
     * let mut selfRef = SelfRefMut {
     *     value: String::from("Hello World"),
     *     pointer_to_value: ptr::null_mut(),
     * };
     * selfRef.pointer_to_value = &mut selfRef.value;
     *
     * // *mut 无论是取值还是赋值，都需要在 unsafe 中操作
     * let pointer_to_value = unsafe { &(*selfRef.pointer_to_value) };
     * println!("{}, {}", selfRef.value, pointer_to_value);
     * let pointer_to_vlaue = unsafe {
     *     *selfRef.pointer_to_value = String::from("Hi");
     *     &(*selfRef.pointer_to_value)
     * };
     * println!(
     *     "{}, {}, {:?}",
     *     selfRef.value, pointer_to_vlaue, selfRef.pointer_to_value
     * );
     * ```
     * unsafe 虽然简单好用，但是它不太安全，当 value 的地址改变后，访问 `pointer_to_value` 即访问 value 原地址就存在未定义行为的风险。
     * 比如完全克隆一份 `selfRef` 数据，然后释放原有的 `selfRef` 数据，此时新数据的 `selfRef.pointer_to_value` 存储的原 value 地址就是一个未定义的数据，访问此时的 `selfRef.pointer_to_value` 就是未定义的行为。
     *
     * ### Pin 固定值的地址
     * unsafe 实现不能保证安全的原因是 value 的地址可能发生更改，`pointer_to_value` 记录 value 的原地址可能是一个未定义的数据，访问存在未定义行为的风险。
     * > 比如完全克隆一份 `selfRef` 数据，然后释放原有的 `selfRef` 数据，此时新数据的 `selfRef.pointer_to_value` 存储的原 value 地址就是一个未定义的数据，访问此时的 `selfRef.pointer_to_value` 就是未定义的行为。
     * 
     * 如果将 value 的地址固定，`pointer_to_value: *const` 就是一直有效的地址，不会存在未定义行为的风险。rust 提供的 Pin 智能指针(结构体)提供了固定地址的功能。
     *
     * 从**是否可以在内存中安全的被移动**的角度，rust 的类型分类两类，`Unpin` 和 `!Unpin`，具体的区分如下：
     * - `Unpin` 表示类型可以在内存中安全地移动，即能安全的改变地址不会带来意外的错误。绝大多数标准库类型都实现了 Unpin。
     * - `!Unpin` 表示类型不可以在内存中安全的移动，即在改变地址的时会发生意外的副作用，比如裸指针实现的自引用结构体，改变结构体地址后，存储的裸指针还是访问原地址，存在未定义行为的风险。
     * 
     * > 特征前的 `!` 代表没有实现某个特征的意思，`!Unpin` 说明类型没有实现 Unpin 特征。
     * 
     * - 如果自定义的类型成员都实现了 Unpin 的，那么该自定义类型会自动实现 Unpin。
     * - `!Unpin` 类型需要通过 PhantomPinned 标记，只要自定义构体里面有一个成员是 `!Unpin`，则该结构也是 `!Unpin`。
     *
     * // TODO 为什么值可以移动，值是怎么移动的，完成Pin
     * Pin 是一个智能指针（结构体），`Unpin` 和 `!Unpin` 则是特征。
     * 如果 Pin 的是 Unpin 类型，则还是可以被移动走的。因为实现 Unpin 就表示移动是安全的。
     * 如果 Pin 的是 !Unpin 类型，则无法被移动走。因为 !Unpin 就表示移动是不安全的。
     * 可以将值固定到栈上，也可以固定到堆上。将 !Unpin 值固定到栈上需要使用 unsafe，将 !Unpin 值固定到堆上无需 unsafe，可以通过 Box::pin 来简单的实现
     *
     *
     *
     * 
     * 可以将值固定到栈上，也可以固定到堆上 将 !Unpin 值固定到栈上需要使用 unsafe 将 !Unpin 值固定到堆上无需 unsafe ，可以通过 Box::pin 来简单的实现
     *
     *
     * TODO 等某一天使用到自引用结构时再来补齐
     */

    #[derive(Debug)]
    struct Node {
        value: i32,
        prev: RefCell<Option<Weak<Node>>>,
        next: RefCell<Option<Rc<Node>>>,
    }

    let a = Rc::new(Node {
        value: 1,
        prev: RefCell::new(None),
        next: RefCell::new(None),
    });
    let b = Rc::new(Node {
        value: 2,
        prev: RefCell::new(None),
        next: RefCell::new(None),
    });

    // 将 a 的 next 指向 b
    *a.next.borrow_mut() = Some(Rc::clone(&b));
    // 将 b 的 prev 指向 a
    *b.prev.borrow_mut() = Some(Rc::downgrade(&a));
    println!("{:#?}", a);

    // 通过 unsafe 操作裸指针实现自引用
    struct SelfRef {
        value: String,
        pointer_to_value: *const String, // 该裸指针指向上面的 value
    }

    let mut selfRef = SelfRef {
        value: String::from("Hello World"),
        pointer_to_value: ptr::null(),
    };
    // 直接存储裸指针信息，引用转换裸指针需要类型标注，否则就是rust引用
    selfRef.pointer_to_value = &selfRef.value;
    // 操作裸指针取值时需要unsafe绕过借用规则和生命周期检查
    let pointer_to_value = unsafe { &(*selfRef.pointer_to_value) };
    println!("{}, {}", selfRef.value, pointer_to_value);

    // *mut 可变裸指针
    struct SelfRefMut {
        value: String,
        pointer_to_value: *mut String, // 该裸指针指向上面的 value
    }
    let mut selfRef = SelfRefMut {
        value: String::from("Hello World"),
        pointer_to_value: ptr::null_mut(),
    };
    selfRef.pointer_to_value = &mut selfRef.value;

    // *mut 无论是取值还是赋值，都需要在 unsafe 中操作
    let pointer_to_value = unsafe { &(*selfRef.pointer_to_value) };
    println!("{}, {}", selfRef.value, pointer_to_value);
    let pointer_to_vlaue = unsafe {
        *selfRef.pointer_to_value = String::from("Hi");
        &(*selfRef.pointer_to_value)
    };
    println!(
        "{}, {}, {:?}",
        selfRef.value, pointer_to_vlaue, selfRef.pointer_to_value
    );

    
}
