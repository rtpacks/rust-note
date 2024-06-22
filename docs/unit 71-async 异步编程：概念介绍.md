## async 异步编程：概念介绍

如果想开发 Web 服务器、数据库驱动、消息服务等需要高并发的服务，那么异步编程认真对待和学习。

简单来说，异步编程是一个并发编程模型，目前主流语言基本都支持，当然，可能支持的方式有所不同。
异步编程允许同时并发运行大量的任务，却仅仅需要几个甚至一个 OS 线程或 CPU 核心，现代化的异步编程在使用体验上与同步编程几乎无区别。
例如 Go 语言的 go 关键字，也包括 async/await 语法，该语法是 JavaScript 和 Rust 的核心特性之一。

### 并发编程模型

- OS 线程, 最简单的一种并发模型，无需改变任何编程业务/代码逻辑，非常适合作为语言的原生并发模型。当然，这种模型也有缺点，例如线程间的同步将变得更加困难，线程间的上下文切换损耗较大。使用线程池在一定程度上可以提升性能，但是对于 IO 密集的场景来说，线程池还是不够。
- 协程(Coroutines) 可能是目前最火的并发模型，协程跟线程类似，无需改变编程模型，同时它也跟 async 类似，可以支持大量的任务并发运行。但协程抽象层次过高，导致用户无法接触到底层的细节，这对于系统编程语言和自定义异步运行时是难以接受的
- actor 模型是 erlang 的杀手锏之一，它将所有并发计算分割成一个一个单元，这些单元被称为 actor , 单元之间通过消息传递的方式进行通信和数据传递，跟分布式系统的设计理念非常相像。由于 actor 模型跟现实很贴近，因此它相对来说更容易实现，但是一旦遇到流控制、失败重试等场景时，就会变得不太好用
- 事件驱动(Event driven), 事件驱动模型性能相当好，常常和回调( Callback )一起使用。但事件驱动模型最大的问题就是存在回调地狱的风险：非线性的控制流和结果处理导致了数据流向和错误传播变得难以掌控，并且代码可维护性和可读性也大幅降低，大名鼎鼎的 JavaScript 曾经就存在回调地狱。
- async/await， 该模型性能高，还能支持底层编程，同时又像线程和协程那样无需过多的改变编程模型，但有得必有失，async 模型的问题就是内部实现机制过于复杂，对于用户来说，理解和使用起来也没有线程和协程简单。

Rust 经过权衡取舍后，最终选择了同时提供多线程编程和 async 编程:

- 多线程编程通过标准库实现，当无需高并发，例如需要并行计算时，就可以选择多线程，优点是线程内的代码执行效率更高、实现更直观更简单
- async/await 通过语言特性 + 标准库 + 三方库的方式实现，在需要高并发、异步 I/O 时，就可以选择这种模型

### async vs 多线程

虽然 async 和多线程都可以实现并发编程，但是这两种并发模型适用的场景不一样，并且方式并不互通，从一个方式切换成另一个需要大量的代码重构工作，因此提前为项目选择适合的并发模型就变得至关重要。

**OS 线程非常适合少量任务的并发或 CPU 密集型任务**

因为线程的创建和上下文切换是非常昂贵的，甚至于空闲的线程都会消耗系统资源。虽说线程池可以有效的降低性能损耗，但是也无法彻底解决问题。
当然，线程模型也有其优点，例如它不会破坏代码逻辑和编程模型，原有的同步代码经过少量修改适配后就可以在新线程中直接运行。
同时在某些操作系统中，还可以改变线程的优先级，这对于实现驱动程序或延迟敏感的应用(例如硬实时系统)很有帮助。

对于长时间运行的 CPU 密集型任务，例如并行计算，使用线程将更有优势。这种密集任务往往会让所在的线程持续运行，任何不必要的线程切换都会带来性能损耗，因此高并发反而在此时成为了一种多余。
需要注意，创建的线程数应该等于 CPU 核心数，以充分利用 CPU 的并行能力，甚至还可以将线程绑定到 CPU 核心上，进一步减少线程上下文切换。

**async 更适合 IO 密集型任务（高并发任务）**

web 服务器、数据库连接等等网络服务，因为这些任务绝大部分时间都处于等待状态，如果使用多线程，那线程大量时间会处于空闲状态，再加上线程上下文切换的高昂代价，让多线程做 IO 密集任务变成了一件成本非常高的事。
而使用 async，既可以有效的降低 CPU 和内存的负担，又可以让大量的任务并发的运行，一个任务一旦处于 IO 或者其他等待(阻塞)状态，就会被立刻切走并执行另一个任务，而这里的任务切换的性能开销要远远低于使用多线程时的线程上下文切换。

事实上, async 底层也是基于线程实现，但是它基于线程封装了一个运行时，可以将多个任务映射到少量线程上，然后将线程切换变成了任务切换，任务切换仅仅是内存中的访问，因此要高效的多。
当然 async 也有其缺点，编译器会为 async 函数生成状态机，然后将整个运行时打包进来，这会造成编译出的二进制可执行文件体积显著增大。

总之，async 编程并没有比多线程更好，最终还是根据使用场景作出合适的选择。如果无需高并发，或者也不在意线程切换带来的性能损耗，那么多线程使用起来会简单、方便的多！

- 有大量 IO 任务需要并发运行时，选 async 模型
- 有部分 IO 任务需要并发运行时，选多线程模型，如果想要降低线程创建和销毁的开销，可以使用线程池
- 有大量 CPU 密集任务需要并行运行时，例如并行计算，选多线程模型，且让线程数等于或者稍大于 CPU 核心数
- 根据性能测试做出选择，或者统一选多线程，因为 async 模型可能要处理额外的问题

> 如果使用 tokio，那 CPU 密集的任务尤其注意需要用多线程模型去处理，例如使用 spawn_blocking 创建一个阻塞的线程去完成相应 CPU 密集任务。
> 至于具体的原因，不仅是多线程适合 CPU 密集型任务，还有一个是：tokio 是协作式的调度器，如果某个 CPU 密集的异步任务是通过 tokio 创建的，那理论上来说，该 CPU 密集型异步任务需要跟其它的异步任务交错执行，最终都得到了执行，皆大欢喜。
> 但实际情况是，CPU 密集的任务很可能会一直霸占着 CPU，此时 tokio 的调度方式决定了该任务会一直被执行，这意味着，其它的异步任务无法得到执行的机会，最终这些任务都会因为得不到资源而**饿死**。
> 而使用 spawn_blocking 后，会创建一个单独的 OS 线程，该线程并不会被 tokio 所调度(被 OS 所调度)，因此它所执行的 CPU 密集任务也不会导致 tokio 调度的那些异步任务被饿死。

**性能对比**
|操作 | async | 线程|
| ---- | ---- | ---- |
|创建 | 0.3 微秒 | 17 微秒|
|线程切换 | 0.2 微秒 | 1.7 微秒|

可以看出，async 在线程切换的开销显著低于多线程，对于 IO 密集的场景，这种性能开销累计下来会非常可怕！

在一些批量下载场景中就可以选择 async 模型，因为涉及到的 IO 密集场景，多线程模型的性能开销更高。

当然 async 和多线程并不是二选一，在同一应用中，可以根据情况两者一起使用，同理，其他并发模型也可以自由搭配使用。

### async/await

目前已经有诸多语言通过 async 的方式提供了异步编程，例如 JavaScript ，但 Rust 在实现上有所区别:

- Future 在 Rust 中是惰性的，只有在被轮询(poll)时才会运行，因此丢弃一个 future 会阻止它未来再被运行，即不再有可能被运行。可以将 Future 理解为一个在未来某个时间点被调度执行的任务
- Async 在 Rust 中使用开销是零，意味着只有能看到的代码(自己编写的代码)才有性能损耗，你看不到的(async 内部实现)都没有性能损耗。简单理解，async 的状态机等相关是在编译器就已经生成，无需运行时再转换。可以无需分配任何堆内存、也无需任何动态分发来使用 async，这对于热点路径的性能有非常大的好处。正是得益于此，Rust 的异步编程性能才会这么高
- Rust 没有内置异步调用所必需的运行时，但 Rust 社区生态中已经提供了非常优异的运行时实现，例如 tokio
- 运行时同时支持单线程和多线程，这两者拥有各自的优缺点

目前 async 并发模型还没有达到多线程并发模型的成熟度，其中一部分内容还在不断进化中。当然，这并不影响在生产级项目中使用，社区中也有 tokio 这种成熟的库。

#### 语言和库的支持

async 的底层实现非常复杂，且会导致编译后文件体积显著增加，因此 Rust 没有选择像 Go 语言那样内置了完整的特性和运行时，而是选择了通过 Rust 语言提供了必要的特性支持，再通过社区来提供 async 运行时的支持。

所以，如果要完整的使用 async 异步编程，需要依赖以下语言特性特性、标准库以及外部库:

- 最基础的特征(例如 Future)、类型和函数，由标准库提供实现
- 关键字 async/await 由 Rust 语言提供，并进行了编译器层面的支持
- 众多实用的类型、宏和函数由官方开发的 futures 包提供(不是标准库)，它们可以用于任何 async 应用中。
- async 代码的执行、IO 操作、任务创建和调度等等复杂功能由社区的 async 运行时提供，例如 tokio 和 async-std

同时，在同步( synchronous )代码中使用的一些语言特性可能无法在 async 中使用，Rust 也不允许你在特征中声明 async 函数(可以通过三方库实现)。

#### 编译和错误

在大多数情况下，async 中的编译错误和运行时错误与普通的编译错误和运行时错误没有区别，但是依然有以下几点值得注意：

- 编译错误，由于 async 编程时需要经常使用复杂的语言特性，例如生命周期和 Pin，因此相关的错误可能会出现的更加频繁
- 运行时错误，编译器会为每一个 async 函数生成状态机，这会导致在栈跟踪时会包含这些状态机的细节，同时还包含了运行时对函数的调用，因此，栈跟踪记录(例如 panic 时)将变得更加难以解读
- 一些隐蔽的错误也可能发生，例如在一个 async 上下文中去调用一个阻塞的函数，或者没有正确的实现 Future 特征都有可能导致这种错误，这种错误可能会悄无声息的通过编译检查甚至有时候会通过单元测试

#### 兼容性考虑

与 JavaScript 不一样，**rust 的异步代码和同步代码并不总能和睦共处**。例如，无法在一个同步函数中去调用一个 async 异步函数，同步和异步代码也往往使用不同的设计模式，这些都会导致两者融合上的困难。

甚至异步代码之间也会存在类似的问题，例如如果一个库依赖于特定的 async 运行时来运行，那么这个库非常有必要告诉它的用户，它用了某个运行时。否则一旦用户选了不同的或不兼容的运行时，就会导致不可预知的麻烦。

#### 性能特性

目前主流的 async 运行时都使用了多线程实现，相比单线程虽然增加了并发表现，但是对于执行性能会有所损失，因为多线程实现会有同步和切换上的性能开销。

如果需要**极致的顺序执行性能**，那么 async 目前并不是一个好的选择。
同样的，对于**延迟敏感的任务**来说，任务的执行次序需要能被严格掌控，而不是交由运行时去自动调度，后者会导致不可预知的延迟。
例如一个 web 服务器总是有 1% 的请求，它们的延迟会远高于其它请求，因为调度过于繁忙导致了部分任务被延迟调度，最终导致了较高的延时。
正因为此，这些延迟敏感的任务非常依赖于运行时或操作系统提供调度次序上的支持。

以上的两个需求，目前的 async 运行时并不能很好的支持，在未来可能会有更好的支持，但在此之前，可以尝试用多线程解决。

### async/await 简单使用

async/.await 是 Rust 内置的语言特性，可以用类似同步的方式去编写异步的代码，这一点与 JavaScript 非常像。
与 JavaScript 中 async/await 是 Promise + Generator 的优化语法糖类似，rust 的 async 也可以看成是优化语法糖。

在 rust 中，通过 async 标记的语法块会被转换成**实现了 Future 特征的状态机**。
与同步调用阻塞当前线程不同，当 Future 执行并遇到阻塞时，它会让出当前线程的控制权，这样其它的 Future 就可以在该线程中运行，这种方式完全不会导致当前线程的阻塞。

使用 async 需要导入 `futures` 包：

```toml
[dependencies]
futures = "0.3.30"
```

async 函数返回的是一个 Future，可以将 Future 理解为一个在未来某个时间点被调度执行的任务。
因此调用 async 函数时，函数不一定会立刻执行，这也是 Future 在 Rust 中是惰性的，只有在被轮询(poll)时才会运行的由来。

想要等待 Future 在当前线程中执行完成后再继续执行线程的其他代码，有两种方式：

- 使用执行器的 block_on，`block_on`会阻塞当前线程直到指定的`Future`执行完成，这种阻塞当前线程以等待任务完成的方式较为简单、粗暴。当然也有运行时的执行器(executor)会提供更加复杂的行为，例如将多个`future`调度到同一个线程上执行。
- 使用 await，这个方式要求调用 async 函数所在的函数也是一个 async 函数，即 await 只能在 async 中使用。await 最大的好处是能够以同步的形式实现异步的执行效果，非常简单、高效，而且很好理解，未来也绝对不会有回调地狱的发生。

**重点注意**：
与 block_on 不同，`future.await` 暂停的是当前 future 所在的上层 async 函数，而不是阻塞当前线程。
.await 暂停所在的上层 async 函数后会让出当前线程的执行权，异步的等待当前 future 的完成。当前线程可以执行与 `future.await` 上层同级的其它异步 Future，最终实现并发处理的效果。

rust 的 .await 位置与 JavaScript await 关键字不一样，再加上 .await 非阻塞线程的概念，可能会有 `future.await` 只是暂停了当前的 `future` 错误理解。
实际上，.await 暂停的是 `future` 所在的上层 `async fn func() { future.await; }` func 函数，让出执行权后当前线程去执行与 `async fn func` 同级的其他异步任务，并不是与当前 `future.await` 同级的其他异步任务。

这一点和 JavaScript 的 await 是一样的，`async function a () { await a_1() }` 内部的 `await a_1()` 后暂停的是 a 函数，当让出执行权后当前线程去执行与 a 同级的其他异步任务。

所以 `.await` 是非阻塞线程的，当 `future.await` 时，它暂停的是当前 future 所在的上层 async 函数，并让出当前线程的执行权，当前线程可以执行与 `future.await` 上层同级的其它异步 Future。

```rust
use tokio::{runtime::Runtime, time::sleep};

// await 不会阻塞当前线程，而是让出当前线程的执行权，在 async 函数结束前会一直异步等待 await 结束
async fn do1(time: u64) {
    sleep(Duration::from_secs(time)).await;
    println!("do 1");
}
async fn do2(time: u64) {
    sleep(Duration::from_secs(time)).await;
    println!("do 2");
}
async fn do3() {
    // 先执行 do1，由于 do1 函数被 .await 暂停执行，异步等待完成，并让出执行权给 do2
    // do2 虽然也有 .await 会经历一段暂停执行，并让出执行权的流程，但是暂停时间短，结束的更快，所以先输出的是 do2，后输出的是 do1
    tokio::join!(do1(2), do2(1));
    println!("do 3");
}
let rt = Runtime::new().unwrap();
rt.block_on(do3());
```

当然，block_on 与 .await 是可以搭配使用的。

#### 简单的并发场景

一个厨房，多个人做饭往往是煲饭和做菜同时进行的，简单来看，煲饭的流程分为淘米和蒸饭，做菜的流程分为洗菜、上火炒菜。
具体来看，煲饭流程中的淘米要在蒸饭之前执行完成，做菜流程中的洗菜要在上火炒菜前执行完成。

可以设计一个简单并发流程，并发煲饭做菜任务，直至两者都完成后才能吃饭。

```rust
// 做饭任务的简单并发
// 淘米
async fn wash_rice() {
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("wash_rice");
}
// 蒸饭
async fn steamed_rice() {
    tokio::time::sleep(Duration::from_secs(6)).await;
    println!("steamed_rice");
}
async fn rice_flow() {
    wash_rice().await;
    steamed_rice().await;
}

// 洗菜
async fn wash_vegetables() {
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("wash_vegetables");
}
// 烧菜
async fn make_vegetables() {
    tokio::time::sleep(Duration::from_secs(4)).await;
    println!("make_vegetables");
}
async fn vegetables_flow() {
    wash_vegetables().await;
    make_vegetables().await;
}

async fn cook() {
    tokio::join!(rice_flow(), vegetables_flow());
    println!("eating");
}
rt.block_on(cook());
```

### Code

```rust
fn main() {
    async fn do_something() {
        println!("Hello World");
    }
    let x = do_something(); // async 函数调用后，不一定会立刻执行
    executor::block_on(x); // 等待 Future

    // await 不会阻塞当前线程，而是让出当前线程的执行权，在 async 函数结束前会一直异步等待 await 结束
    async fn do1(time: u64) {
        sleep(Duration::from_secs(time)).await;
        println!("do 1");
    }
    async fn do2(time: u64) {
        sleep(Duration::from_secs(time)).await;
        println!("do 2");
    }
    async fn do3() {
        // 先执行 do1，由于 do1 函数被 .await 暂停执行，异步等待完成，并让出执行权给 do2
        // do2 虽然也有 .await 会经历一段暂停执行，并让出执行权的流程，但是暂停时间短，结束的更快，所以先输出的是 do2，后输出的是 do1
        tokio::join!(do1(2), do2(1));
        println!("do 3");
    }
    let rt = Runtime::new().unwrap();
    rt.block_on(do3());

    // 做饭任务的简单并发
    // 淘米
    async fn wash_rice() {
        tokio::time::sleep(Duration::from_secs(3)).await;
        println!("wash_rice");
    }
    // 蒸饭
    async fn steamed_rice() {
        tokio::time::sleep(Duration::from_secs(6)).await;
        println!("steamed_rice");
    }
    async fn rice_flow() {
        wash_rice().await;
        steamed_rice().await;
    }

    // 洗菜
    async fn wash_vegetables() {
        tokio::time::sleep(Duration::from_secs(3)).await;
        println!("wash_vegetables");
    }
    // 烧菜
    async fn make_vegetables() {
        tokio::time::sleep(Duration::from_secs(4)).await;
        println!("make_vegetables");
    }
    async fn vegetables_flow() {
        wash_vegetables().await;
        make_vegetables().await;
    }

    async fn cook() {
        tokio::join!(rice_flow(), vegetables_flow());
        println!("eating");
    }
    rt.block_on(cook());
}
```
