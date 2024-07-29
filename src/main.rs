use mini_redis::Result;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{self, TcpListener},
};

#[tokio::main]
async fn main() -> Result<()> {
    /*
     *
     * ## 实战：mini-redis - client - IO & Frame
     *
     * 在 mini-redis 中，以帧 frame 作为命令和数据的结合作为一次指令操作，要构建帧 frame 需要先认识 tokio 的 io 操作。
     * Tokio 中的 I/O 操作和 std 在使用方式上几乎没有区别，只是 Tokio 是异步的，std 是同步的，例如 Tokio 的读写特征分别是 AsyncRead 和 AsyncWrite。
     *
     * ### AsyncRead 和 AsyncWrite
     *
     * AsyncRead 和 AsyncWrite 是非常基础的特征，很多类型和数据结构都实现了它们：
     * - 部分类型如 TcpStream，File，Stdout 实现了它们，支持异步读写
     * - 部分数据结构如 `Vec<u8>、&[u8]` 也实现了它们：，支持直接将这些**数据结构作为读写器( reader / writer)**，一些常见的 buff 容器其实就可以视为读取器和写入器。
     *
     * 这两个特征为字节流的异步读写提供了便利，通常会使用 `AsyncReadExt` 和 `AsyncWriteExt` 提供的工具方法，这些方法都是 async 声明，需要 .await 调用。
     *
     * buffer 作为读取器还是写入器是根据实际场景决定的：
     * - 如果从 buffer 中读取内容复制到写入器（writer）中，那么 buffer 就是读取器（reader），如 `&[u8]`
     * - 如果从读取器（reader）中读取内容并写入到 buffer 中，那么 buffer 就是写入器（writer），如 `&[u8]`
     *
     * 注意，是切片 `&[u8]` 而不是字节数组引用 `&[u8; length]`
     *
     * #### read read_to_end
     * AsyncReadExt::read 是一个异步方法可以将数据读入缓冲区( buffer )中，然后返回读取的字节数。
     * 需要注意的是：当 read 返回 Ok(0) 时，意味着字节流( stream )已经关闭，在这之后继续调用 read 会立刻完成，依然获取到返回值 Ok(0)。 例如，字节流如果是 TcpStream 类型，那 Ok(0) 说明该连接的读取端已经被关闭(写入端关闭，会报其它的错误)。
     * ```rust
     * use tokio::{self, AsyncReadExt, AsyncWriteExt};
     *
     * // 部分数据结构如 `Vec<u8>、&[u8]` 也实现了它们：，支持直接将这些**数据结构作为读写器( reader / writer)**，一些常见的 buffer 容器其实就可以视为读取器和写入器。
     * let mut file = File::open(r"Cargo.toml").await.unwrap();
     * let mut buffer = [0; 1024]; // 写入器
     *
     * // 由于 buffer 的长度限制，当次的 `read` 调用最多可以从文件中读取 1024 个字节的数据
     * let n = file.write(&mut buffer).await.unwrap();
     * println!("The bytes: {:?}", buffer);
     * ```
     *
     * AsyncReadExt::read_to_end 方法会从字节流中读取所有的字节，直到遇到 EOF。
     * ```rust
     * let mut file = File::open("Cargo.toml").await.unwrap();
     * // 写入器
     * let mut buffer = Vec::new();
     * let n = file.read_to_end(&mut buffer).await.unwrap();
     * println!("The bytes: {:?}", buffer);
     * ```
     *
     * 因为 `&[u8]` 实现了 AsyncRead 特征，所以可以直接将 `&[u8]` 作为读取器。
     *
     * #### write write_all
     *
     * AsyncWriteExt::write 异步方法会尝试将缓冲区的内容写入到写入器( writer )中，同时返回写入的字节数。
     * ```rust
     * let mut file = File::create("public/foo.txt").await?;
     * // 读取器
     * // let buffer = "Hello World".as_bytes();
     * let buffer = b"Hello World";
     * let n = file.write(buffer).await.unwrap();
     * println!("Wrote the first {} bytes of 'some bytes'.", n);
     * ```
     *
     * `b"some bytes"` 写法可以将一个 &str 字符串转变成一个字节数组：&[u8;10]，然后 write 方法又会将这个 &[u8;10] 的数组类型隐式强转为数组切片: &[u8]。
     * `"some bytes".to_bytes()` 函数则可以直接将字符串转变为字节切片。
     *
     * AsyncWriteExt::write_all 将缓冲区的内容全部写入到写入器中，因为全部写入，所以不再返回字节数。
     * ```rust
     * let mut file = File::create(r"public/foo.txt").await?;
     * // 读取器
     * // let buffer = "Hello World".as_bytes();
     * let buffer = b"Hello World";
     * file.write_all(buffer).await.unwrap();
     * ```
     * 因为 `&[u8]` 实现了 AsyncWrite 特征，所以可以直接将 `&[u8]` 作为写入器。
     *
     *
     * 更多函数阅读：https://docs.rs/tokio/latest/tokio/io/index.html
     *
     * ### 实用函数
     * read 和 write 是最基础的操作，和标准库一样，tokio::io 模块包含了多个实用的封装好的函数或 API，可以用于处理标准输入/输出/错误等。
     * 例如，tokio::io::copy 异步的将读取器( reader )中的内容拷贝到写入器( writer )中。
     *
     * ```rust
     * let mut file = File::create(r"public/foo.txt").await?;
     * // 读取器
     * let mut buffer = "Hello World".as_bytes();
     *
     * io::copy(&mut buffer, &mut file).await.unwrap();
     * ```
     *
     * ### 回声服务 （Echo）
     * 如同写代码必写 hello, world，实现 web 服务器，往往会选择实现一个回声服务。该服务会将用户的输入内容直接返回给用户，就像回声壁一样。
     * 具体来说，就是从用户建立的 TCP 连接的 socket 中读取到数据，然后立刻将同样的数据写回到该 socket 中。因此客户端会收到和自己发送的数据一模一样的回复。
     *
     * 和 async Web 服务器实现类似，基本的服务器框架：通过 loop 循环接收 TCP 连接，然后为每一条连接创建一个单独的任务去处理。
     *
     * ```rust
     * let listener = TcpListener::bind("127.0.0.1:6330").await?;
     *
     * async fn process(stream: net::TcpStream) {}
     *
     * loop {
     *     let (stream, addr) = listener.accept().await?;
     *     tokio::spawn(async move { process(stream).await });
     * }
     * ```
     *
     * 然后使用 `io::copy` 函数完成回声服务。copy 函数有两个参数：读取器的可变引用，写入器的可变引用，现在需要将读取器中的数据直接拷贝到写入器中。
     *
     * 在当前服务中，读取器和写入器都是 stream，根据借用规则，copy 不能同时操作两个 stream 的可变引用：
     * ```rust
     * io::copy(&mut stream, &mut stream).await
     * ```
     *
     * 借用规则限制只能操作一个变量的一个可变引用，这里 stream 不能既做读取器又做写入器。
     * 任何一个读写器( reader + writer )都可以使用 io::split 方法进行分离，最终返回一个读取器和写入器，这两者可以单独使用。
     * 实际上，io::split 可以用于任何同时实现了 AsyncRead 和 AsyncWrite 的值，它的内部使用了 Arc 和 Mutex 来实现相应的功能。
     *
     * ```rust
     * let listener = TcpListener::bind("127.0.0.1:6330").await?;
     *
     * async fn process(mut stream: net::TcpStream) {
     *     let (mut reader, mut writer) = io::split(stream);
     *
     *     if io::copy(&mut reader, &mut writer).await.is_err() {
     *         eprintln!("failed to copy");
     *     };
     * }
     *
     * loop {
     *     let (stream, addr) = listener.accept().await?;
     *     tokio::spawn(async move { process(stream).await });
     * }
     * ```
     *
     * `io::split` 利用 Mutex 会有一定的性能损耗，还有两种方式可以分离读写器：
     * - TcpStream::split会获取字节流的引用，然后将其分离成一个读取器和写入器。但由于使用了引用的方式，它们俩必须和 split 在同一个任务中。 优点就是，这种实现没有性能开销，因为无需 Arc 和 Mutex。
     * - TcpStream::into_split还提供了一种分离实现，分离出来的结果可以在任务间移动，内部是通过 Arc 实现。
     *
     * ```rust
     * let listener = TcpListener::bind("127.0.0.1:6330").await?;
     *
     * async fn process(mut stream: net::TcpStream) {
     *     let (mut reader, mut writer) = stream.split();
     *
     *     if io::copy(&mut reader, &mut writer).await.is_err() {
     *         eprintln!("failed to copy");
     *     };
     * }
     *
     * loop {
     *     let (stream, addr) = listener.accept().await?;
     *     tokio::spawn(async move { process(stream).await });
     * }
     * ```
     *
     *
     */

    // {
    //     // 部分数据结构如 `Vec<u8>、&[u8]` 也实现了它们：，支持直接将这些**数据结构作为读写器( reader / writer)**，一些常见的 buffer 容器其实就可以视为读取器和写入器。
    //     let mut file = File::open(r"Cargo.toml").await.unwrap();
    //     // 写入器
    //     let mut buffer = [0; 1024];
    //     // 由于 buffer 的长度限制，当次的 `read` 调用最多可以从文件中读取 1024 个字节的数据
    //     let n = file.write(&mut buffer).await.unwrap();
    //     println!("The bytes: {:?}", buffer);

    //     let mut file = File::open("Cargo.toml").await.unwrap();
    //     // 写入器
    //     let mut buffer = Vec::new();
    //     let n = file.read_to_end(&mut buffer).await.unwrap();
    //     println!("The bytes: {:?}", buffer);
    // }

    // {
    //     // 部分数据结构如 `Vec<u8>、&[u8]` 也实现了它们：，支持直接将这些**数据结构作为读写器( reader / writer)**，一些常见的 buffer 容器其实就可以视为读取器和写入器。
    //     let mut file = File::create("public/foo.txt").await?;
    //     // 读取器
    //     // let buffer = "Hello World".as_bytes();
    //     let buffer = b"Hello World";
    //     let n = file.write(buffer).await.unwrap();
    //     println!("Wrote the first {} bytes of 'some bytes'.", n);

    //     let mut file = File::create(r"public/foo.txt").await?;
    //     // 读取器
    //     // let buffer = "Hello World".as_bytes();
    //     let buffer = b"Hello World";
    //     file.write_all(buffer).await.unwrap();
    // }

    // {
    //     let mut file = File::create(r"public/foo.txt").await?;
    //     // 读取器
    //     let mut buffer = "Hello World".as_bytes();

    //     io::copy(&mut buffer, &mut file).await.unwrap();
    // }

    // {
    //     let listener = TcpListener::bind("127.0.0.1:6330").await?;

    //     async fn process(mut stream: net::TcpStream) {
    //         let (mut reader, mut writer) = io::split(stream);

    //         if io::copy(&mut reader, &mut writer).await.is_err() {
    //             eprintln!("failed to copy");
    //         };
    //     }

    //     loop {
    //         let (stream, addr) = listener.accept().await?;
    //         tokio::spawn(async move { process(stream).await });
    //     }
    // }

    {
        let listener = TcpListener::bind("127.0.0.1:6330").await?;

        async fn process(mut stream: net::TcpStream) {
            let (mut reader, mut writer) = stream.split();

            if io::copy(&mut reader, &mut writer).await.is_err() {
                eprintln!("failed to copy");
            };
        }

        loop {
            let (stream, addr) = listener.accept().await?;
            tokio::spawn(async move { process(stream).await });
        }
    }

    Ok(())
}
