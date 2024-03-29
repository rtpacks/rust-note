use ilearn::{run, Config};
use std::{env, error::Error, fs, process};

fn main() {
    /*
     * ## 迭代器
     *
     * 从用途来看，迭代器跟 for 循环颇为相似，都是去遍历一个集合，但是实际上它们存在不小的差别，其中最主要的差别就是：是否通过索引来访问集合。
     *
     * 在 JavaScript 中，`for` 和 `for..of` 就是for循环和迭代器的代表：
     * ```js
     * const arr = [1, 2, 3];
     *
     * // for 循环
     * for(let i=0; i<arr.length; i++) {
     *      console.log(`index = ${i}, value = ${arr[i]}`);
     * }
     *
     * // 迭代器
     * for(const value of arr) {
     *      console.log(`value = ${value}`)
     * }
     *
     * // 迭代器，调用特殊的方法，它的key是Symbol.iterator，可以理解成 arr.into_iter() 形式，它将返回一个迭代器
     * arr[Symbol.iterator]()
     * ```
     *
     * JavaScript生成迭代器和可迭代对象非常容易：
     * - 迭代器是指实现迭代器协议的对象(iterator protocol)，其具有特殊属性 `next` 函数的对象，next 函数返回一个 `{value: any, done: boolean}` 形式的对象。
     * - 可迭代对象是指实现可迭代协议(iterable protocol)的对象，它具有特殊的属性 `Symbol.iterator` 函数，这个函数返回迭代器，一个对象能否使用迭代器就看是否实现 `Symbol.iterator`。
     * 迭代协议：https://developer.mozilla.org/zh-CN/docs/Web/JavaScript/Reference/Iteration_protocols
     *
     * ```javascript
     * let index = 0;
     * const iter = {
     *      next() {
     *          if(index <= 1) return { value: index, done: false };
     *          return { value: null, done: true };
     *      }
     * }
     *
     * iter.next() // {value: 0, done: false}
     * iter.next() // {value: 1, done: false}
     * iter.next() // {value: null, done: true}
     * ```
     *
     * 迭代器 next 函数返回值是 {value: any, done: boolean} 形式，所以只需要实现一个函数，循环调用一个对象（迭代器） next 函数取出值，当 next 函数给出终止信号 done = true 时，停止函数流程，就能伪实现可迭代。
     *
     * ```rust
     * function into_iter(arr: number[]) {
     *      let index = 0;
     *      return {
     *          next: () => {
     *              if(index < arr.length) return { value: arr[index], done: false };
     *              return { value: null, done: true };
     *          }
     *      }
     * }
     *
     * const arr = [1, 2];
     * const iterator = into_iter(arr);
     * iterator.next(); // {value: 0, done: false};
     * iterator.next(); // {value: 1, done: false};
     * iterator.next(); // {value: null, done: true};
     * ```
     *
     * **流程再分析**
     *
     * 持续迭代终止时需要一个终止信号，它首先需要一个询问的对象，再者需要固定方式来获取终止信息的流程如循环调用next函数，现在询问对象是迭代器，循环调用next函数的流程就是迭代，也就是迭代迭代器以获取终止信号。
     *
     * 至此，补上迭代迭代器的流程，整体的可迭代就完成了。
     * ```js
     * function forOf(arr: number[]) {
     *      const iterator = into_iter(arr);
     *      while(iterator.next().done === true) break;
     * }
     * ```
     *
     * 在上面的模拟流程中，可以将迭代器视为单元算子，将可迭代过程视为整体流程，启动迭代后当单元算子给出终止信号时停止流程。
     *
     * ### For 循环与迭代器
     * rust的迭代器协议和可迭代协议也是不同的概念，可以先记住名词。
     *
     * ```rust
     * let arr = [1, 2, 3];
     * for v in arr {
     *     println!("{}",v);
     * }
     * ```
     *
     * Rust的 `for..in` 没有使用索引，它把 arr 数组当成一个迭代器，直接去遍历其中的元素，从哪里开始，从哪里结束，都无需操心。因此严格来说，Rust 中的 **for 循环是编译器提供的语法糖**，最终还是对迭代器中的元素进行遍历。
     *
     * 同时值得关注数组不是迭代器，但数组实现了 IntoIterator （可迭代）特征，Rust 通过 for 语法糖，自动把实现了该特征的数组类型转换为迭代器，以方便 `for..in` 进行迭代。
     * 类似的快捷操作有 `for i in 1..10` 直接对数值序列进行迭代。
     *
     * IntoIterator 特征拥有一个 `into_iter` 方法，因此我们还可以显式的把数组转换成迭代器 `for v in arr.into_iter() {}`，迭代器是函数语言的核心特性，它赋予了 Rust 远超于循环的强大表达能力。
     *
     * ### 惰性初始化
     * 在 Rust 中，迭代器是惰性的，意味着如果你不使用它，那么它将不会发生任何事,
     *
     * ```rust
     * let v1 = vec![1, 2, 3];
     * let v1_iter = v1.iter(); // 迭代器惰性初始化，不会立即加载，而是使用到时才会开始加载
     *
     * for val in v1_iter { // 迭代器开始加载
     *     println!("{}", val);
     * }
     * ```
     *
     * 在迭代过程之前，只是简单的创建了一个迭代器 v1_iter，此时不会发生任何迭代行为，只有在 for 循环开始后，迭代器才会开始迭代其中的元素。
     * 这种惰性初始化的方式确保了创建迭代器不会有任何额外的性能损耗，其中的元素也不会被消耗，只有使用到该迭代器的时候，一切才开始。
     *
     * ### next函数
     * rust中for循环通过调用迭代器的 `next` 函数取出迭代器内的元素，迭代器之所以成为迭代器，就是因为实现了 `Iterator` 特征，要实现该特征，最主要的就是实现其中的 next 方法，该方法控制如何从集合中取值，最终返回值的类型是关联类型 Item。
     *
     * ```rust
     * pub trait Iterator {
     *     type Item;
     *     fn next(&mut self) -> Option<Self::Item>;
     *     // 省略其余有默认实现的方法
     * }
     * ```
     *
     * 与 JavaScript 手动创建迭代器对象非常相似，两者都有 `next` 函数， for 循环通过不停调用迭代器上的 next 方法，来获取迭代器中的元素。
     *
     * 当然也可以手动执行 `next` 函数来获取迭代器中的元素，因为涉及到rust的所有权模型，所以rust的next调用需要多注意几步：
     * - `next(&mut self)` 是可变引用，调用者必须要可变（mut）。手动迭代必须将迭代器声明为 mut 可变，因为调用 next 会改变迭代器其中的状态数据（当前遍历的位置等），而 for 循环去迭代则无需标注 mut，因为它会帮我们自动完成
     * - rust中只有Option，没有undefined/null，next的返回是一个Option类型，即有值为Some，无值时为None
     * - next 方法对迭代器的遍历是消耗性的，每次消耗它一个元素，最终迭代器中将没有任何元素，只能返回 None。
     * - 遍历是按照迭代器中元素的排列顺序依次进行的
     *
     *
     * ```javascript
     * let arr = [1, 2];
     * let iter = arr.values();
     * iter.next(); // {value: 1, done: false}
     * iter.next(); // {value: 1, done: false}
     * iter.next(); // {value: undefined, done: true}
     * ```
     *
     * ```rust
     * let v = vec![1, 2, 3];
     * let mut iter = v.into_iter();
     * iter.next(); // Some(1)
     * iter.next(); // Some(2)
     * iter.next(); // None
     * ```
     */

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
}
