use std::{pin::Pin, rc::Rc};

use futures::{executor, Future};
use tokio::runtime::{Builder, Runtime};

fn main() {
    /*
     *
     * ## 实战：Web 服务器
     * 一般来说，现代化的 web 服务器由于 IO 频繁往往会基于更加轻量级的协程或 async/await 等模式实现，协程和 async/await 的好处是提供并发量。
     *
     * 虽然单线程和多线程 Web 服务器现在并不常见，但实现一个简单版本的线程服务器对理解 Web 有很多好处。
     *
     * 当然新技术也是不可缺少的，所以这里会实现三种 Web 服务器，提升对 Web 的理解：
     * 1. 单线程 Web 服务器
     * 2. 多线程 Web 服务器
     * 3. async/await Web 服务器
     *
     */
}
