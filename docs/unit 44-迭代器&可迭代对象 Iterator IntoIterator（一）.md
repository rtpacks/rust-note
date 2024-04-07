## 迭代器

从用途来看，迭代器跟 for 循环颇为相似，都是去遍历一个集合，但是实际上它们存在不小的差别，其中最主要的差别就是：是否通过索引来访问集合。

在 JavaScript 中，`for` 和 `for..of` 就是 for 循环和迭代器的代表：

```js
const arr = [1, 2, 3];

// for 循环
for (let i = 0; i < arr.length; i++) {
  console.log(`index = ${i}, value = ${arr[i]}`);
}

// 迭代器
for (const value of arr) {
  console.log(`value = ${value}`);
}

// 迭代器，调用特殊的方法，它的key是Symbol.iterator，可以理解成 arr.into_iter() 形式，它将返回一个迭代器
arr[Symbol.iterator]();
```

JavaScript 生成迭代器和可迭代对象非常容易：

- 迭代器是指实现迭代器协议的对象(iterator protocol)，其具有特殊属性 `next` 函数的对象，next 函数返回一个 `{value: any, done: boolean}` 形式的对象。
- 可迭代对象是指实现可迭代协议(iterable protocol)的对象，它具有特殊的属性 `Symbol.iterator` 函数，这个函数返回迭代器，一个对象能否使用迭代器就看是否实现 `Symbol.iterator`。
  迭代协议：https://developer.mozilla.org/zh-CN/docs/Web/JavaScript/Reference/Iteration_protocols

```javascript
let index = 0;
const iter = {
  next() {
    if (index <= 1) return { value: index, done: false };
    return { value: null, done: true };
  },
};

iter.next(); // {value: 0, done: false}
iter.next(); // {value: 1, done: false}
iter.next(); // {value: null, done: true}
```

迭代器 next 函数返回值是 {value: any, done: boolean} 形式，所以只需要实现一个函数，循环调用一个对象（迭代器） next 函数取出值，当 next 函数给出终止信号 done = true 时，停止函数流程，就能伪实现可迭代。

```javascript
function into_iter(arr: number[]) {
  let index = 0;
  return {
    next: () => {
      if (index < arr.length) return { value: arr[index], done: false };
      return { value: null, done: true };
    },
  };
}

const arr = [1, 2];
const iterator = into_iter(arr);
iterator.next(); // {value: 0, done: false};
iterator.next(); // {value: 1, done: false};
iterator.next(); // {value: null, done: true};
```

**流程再分析**

持续迭代终止时需要一个终止信号，它首先需要一个询问的对象，再者需要固定方式来获取终止信息的流程如循环调用 next 函数，现在询问对象是迭代器，循环调用 next 函数的流程就是迭代，也就是迭代迭代器以获取终止信号。

至此，补上迭代迭代器的流程，整体的可迭代就完成了。

```js
function forOf(arr: number[]) {
  const iterator = into_iter(arr);
  while (iterator.next().done === true) break;
}
```

在上面的模拟流程中，可以将迭代器视为单元算子，将可迭代过程视为整体流程，启动迭代后当单元算子给出终止信号时停止流程。

### For 循环与迭代器

rust 的迭代器协议和可迭代协议也是不同的概念，可以先记住名词。

```rust
let arr = [1, 2, 3];
for v in arr {
    println!("{}",v);
}
```

Rust 的 `for..in` 没有使用索引，它把 arr 数组当成一个迭代器，直接去遍历其中的元素，从哪里开始，从哪里结束，都无需操心。因此严格来说，Rust 中的 **for 循环是编译器提供的语法糖**，最终还是对迭代器中的元素进行遍历。

同时值得关注数组不是迭代器，但数组实现了 IntoIterator （可迭代）特征，Rust 通过 for 语法糖，自动把实现了该特征的数组类型转换为迭代器，以方便 `for..in` 进行迭代。
类似的快捷操作有 `for i in 1..10` 直接对数值序列进行迭代。

IntoIterator 特征拥有一个 `into_iter` 方法，因此我们还可以显式的把数组转换成迭代器 `for v in arr.into_iter() {}`，迭代器是函数语言的核心特性，它赋予了 Rust 远超于循环的强大表达能力。

### 惰性初始化

在 Rust 中，迭代器是惰性的，意味着如果你不使用它，那么它将不会发生任何事,

```rust
let v1 = vec![1, 2, 3];
let v1_iter = v1.iter(); // 迭代器惰性初始化，不会立即加载，而是使用到时才会开始加载

for val in v1_iter { // 迭代器开始加载
    println!("{}", val);
}
```

在迭代过程之前，只是简单的创建了一个迭代器 v1_iter，此时不会发生任何迭代行为，只有在 for 循环开始后，迭代器才会开始迭代其中的元素。
这种惰性初始化的方式确保了创建迭代器不会有任何额外的性能损耗，其中的元素也不会被消耗，只有使用到该迭代器的时候，一切才开始。

### next 函数

rust 中 for 循环通过调用迭代器的 `next` 函数取出迭代器内的元素，迭代器之所以成为迭代器，就是因为实现了 `Iterator` 特征，要实现该特征，最主要的就是实现其中的 next 方法，该方法控制如何从集合中取值，最终返回值的类型是关联类型 Item。

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // 省略其余有默认实现的方法
}
```

与 JavaScript 手动创建迭代器对象非常相似，两者都有 `next` 函数， for 循环通过不停调用迭代器上的 next 方法，来获取迭代器中的元素。

当然也可以手动执行 `next` 函数来获取迭代器中的元素，因为涉及到 rust 的所有权模型，所以 rust 的 next 调用需要牢记几点：

- `next(&mut self)` 是可变引用，调用者必须要可变（mut）。即手动迭代必须将迭代器声明为 mut，因为调用 next 会改变迭代器其中的状态数据（当前遍历的位置等），而 for 循环迭代自动标注 mut 可变。
- rust 中有 Option 没有 undefined/null，next 的返回是一个 Option 类型，即有值为 Some，无值时为 None
- next 方法对迭代器的遍历是**消耗性**的，每次消耗它一个元素，最终迭代器中将没有任何元素，只能返回 None。
- 遍历是按照迭代器中元素的排列顺序依次进行的

```rust
let v = vec![1, 2, 3];
let mut iter = v.into_iter();
iter.next(); // Some(1)
iter.next(); // Some(2)
iter.next(); // None
```

JavaScript 代码实例

```javascript
let arr = [1, 2];
let iter = arr.values();
iter.next(); // {value: 1, done: false}
iter.next(); // {value: 1, done: false}
iter.next(); // {value: undefined, done: true}
```

对比 JavaScript 代码可以猜测到，rust 迭代也是通过循环调用迭代器的 next 函数来实现的。

#### 实现 for 伪迭代

```rust
let v = vec![1, 2, 3];
let mut iter = v.into_iter();
loop {
     match iter.next() {
         Some(x) => println!("{x}");
         None => break;
     }
}
```

和 JavaScript 将可迭代对象转换成迭代器一样，使生成的对象拥有 next 函数，然后循环调用，最终完成迭代。将其改造成 `forFn`：

```rust
fn forFn<T>(iter: T)
where
     T: Iterator,
     T::Item: std::fmt::Debug,
{
     loop {
         match iter.next() {
             Some(x) => println!("{#?}", x);
             None => break;
         }
     }
}

let v = vec![1, 2, 3];
forFn(v.into_iter()); // 调用forFn进行迭代
```

rust 的所有权决定了 into_iter 函数不好实现，因此可以先参考 JavaScript 实现方式来理解整个流程。

此外，可迭代对象除了通过 `into_iter` 函数转换成迭代器，还可以通过完全限定的方式即 `IntoIterator::into_iter(values)` 生成迭代器，这种调用方式跟 `values.into_iter()` 是等价的：

> 完全限定：https://course.rs/basic/trait/advance-trait.html#%E5%AE%8C%E5%85%A8%E9%99%90%E5%AE%9A%E8%AF%AD%E6%B3%95

```rust
let values = vec![1, 2, 3];
{
    let result = match IntoIterator::into_iter(values) {
        mut iter => loop {
            match iter.next() {
                Some(x) => { println!("{}", x); },
                None => break,
            }
        },
    };
    result
}
```

### IntoIterator 特征

由于 Vec 动态数组实现了 IntoIterator 特征，因此可以通过 into_iter 将其转换为迭代器，那如果本身就是一个迭代器，该怎么办？实际上，迭代器自身也实现了 IntoIterator 特征：

```rust
impl<I: Iterator> IntoIterator for I {
    type Item = I::Item;
    type IntoIter = I;

    #[inline]
    fn into_iter(self) -> I {
        self
    }
}
```

也就是说，迭代器可以使用 `into_iter` 方法，能形成以下代码，**迭代器能够调用 into_iter 方法，返回的还是迭代器**：

```rust
let v = vec![1,2,3];
for v in values.into_iter().into_iter().into_iter() {
    println!("{}",v)
}
```

#### into_iter, iter, iter_mut

在之前的代码中，只使用了 into_iter 的方式将数组转化为迭代器，除此之外，还有 iter 和 iter_mut 两种变体：

- into_iter 会夺走所有权，next 函数返回的是 Option<T>，即 Some<T>和 None
- iter 是不可变借用，next 函数返回的是 Option<&T>，即 Some<&T>和 None
- iter_mut 是可变借用，next 函数返回的是 Option<&mut T>，即 Some<&mut T>和 None

rust 方法的命名一般都遵守这个规则：**into\_ 之类的都是拿走所有权，\_mut 之类的都是可变借用，剩下的就是不可变借用**

```rust
let v = vec![1, 2, 3];
let mut iter = v.into_iter(); // 移除v所有权
// println!("{:#?}", v); v被移除所有权，无法再次使用
iter.next(); // Option<i32>

let mut v = vec![1, 2, 3];
let mut iter = v.iter_mut(); // 对 values 中的元素进行可变借用，注意可变引用需要来自可变变量，因此要求v mut可变
iter.next(); // Option<&mut i32>

let values = vec![1, 2, 3];
let mut iter = values.iter(); // 对 values 进行不可变借用（只读）
iter.next(); // Option<&i32>
```

#### Iterator 和 IntoIterator 的区别

Iterator 和 IntoIterator 两者与 JavaScript 迭代器中的迭代器协议和可迭代协议是类似的。

Iterator 是迭代器特征，只有实现了它才能称为迭代器，才能调用 next 方法。而 IntoIterator 可以称为可迭代对象特征，强调的是某一个类型如果实现了该特征，它可以通过 into_iter，iter_mut，iter 等方法变成一个迭代器。


### Code
```rust
fn main() {
    let v = vec![1, 2, 3];
    let mut iter = v.into_iter();
    iter.next();
    iter.next();
    iter.next();
    iter.next();

    println!("{}", iter.next().unwrap_or_default());
    println!("{}", iter.next().unwrap_or_default());
    println!("{}", iter.next().unwrap_or_default());
    println!("{}", iter.next().unwrap_or_default());

    // 实现for循环迭代功能
    let v = vec![1, 2, 3];
    let mut iter = v.into_iter();

    fn forFn<T>(mut iter: T)
    where
        T: Iterator,
        T::Item: std::fmt::Debug,
    {
        loop {
            match iter.next() {
                Some(x) => {
                    println!("{:?}", x);
                }
                None => break,
            }
        }
    }

    forFn(iter);

    let v = vec![1, 2, 3];
    forFn(v.into_iter());

    let v = vec![1, 2, 3];
    // println!("{:#?}", v);
    v.into_iter(); // 移除v所有权

    // into_mut 移动所有权
    let v = vec![1, 2, 3];
    let mut iter = v.into_iter();
    let x = iter.next(); // 需要移动所有权，即需要iter可变

    // iter_mut 可变借用
    let mut v = vec![1, 2, 3];
    let mut iter = v.iter_mut();
    // 取出第一个元素，并修改为0
    if let Some(v) = iter.next() {
        *v = 0;
    }
    println!("{:#?}", v);

    // iter 不可变借用
    let v = vec![1, 2, 3];
    let mut iter = v.iter();
    let x = iter.next();
}
```