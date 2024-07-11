use std::{
    io::{BufRead, BufReader},
    net,
};

fn main() {
    /*
     *
     * ## 实战：单线程 Web 服务器
     * 构建所需的网络协议: HTTP 和 TCP。这两种协议都是请求-应答模式的网络协议，意味着在客户端发起请求后，服务器会监听并处理进入的请求，最后给予应答，至于这个过程怎么进行，取决于具体的协议定义。
     * 与 HTTP 有所不同， TCP 是一个底层协议，它仅描述客户端传递了信息给服务器，至于这个信息长什么样，怎么解析处理，则不在该协议的职责范畴内。
     * HTTP 协议是更高层的通信协议，一般来说都基于 TCP 来构建 (HTTP/3 是基于 UDP 构建的协议)，更高层的协议也意味着它会对传输的信息进行解析处理。
     *
     * 使用 std::net 模块监听进入的请求连接，IP 和端口是 127.0.0.1:7878。
     *
     * ```rust
     * let listener = net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error!");
     * for stream in listener.incoming() {
     *     let stream = stream.unwrap();
     *     println!("Connection established!");
     * }
     * ```
     * TcpListener start error! 修改为正确的语句
     * bind 非常类似 new 操作符，它生成一个 TcpListener 实例。之所以不用 new，是因为一般都说 "绑定到某个端口"，因此 bind 这个名称会更合适。
     * 生成并绑定到指定的端口是可能失败的，比如给定的地址是错误的、端口已经被绑定等。
     *
     * `listener.incoming` 返回**请求建立连接的 TcpStream**，请求连接的操作是有可能失败的，比如超出最大的连接数，所以需要使用 Result 包裹。
     * 注意：这个时候 for 迭代的是请求建立连接的 TcpStream，并不是已经建立的连接。
     *
     * cargo run 运行后，浏览器访问 `127.0.0.1:7878`，控制台可能会输出多条 `Connection established!`：
     * ```shell
     * Connection established!
     * Connection established!
     * Connection established!
     * ```
     *
     * 这是因为 stream 完成连接建立后，因为 for 单次循环的结束导致 stream 被 drop，所以连接断开，又因为浏览器有连接重试的特性，所以控制台可能会输出多条 `Connection established!`。
     *
     * 由于 listener.incoming 是阻塞式监听，所以 main 线程会被阻塞，最后需要通过 `ctrl + c` 来结束程序进程。
     *
     * ### 解析请求报文
     * 在请求连接建立后，需要解析客户端发送的数据，并给出对应的响应，这次请求才算是完整的。现在先看如何获取客户端发送的报文。
     * 
     * TcpStream 返回的是字节流，这里使用 BufReader 读取报文，并使用给定的迭代器简化实现读取逻辑：
     * ```rust
     * let listener =
     *     net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error!");
     * for stream in listener.incoming() {
     *     let stream = stream.unwrap();
     *     println!("Connection established!");
     *
     *     let buf_reader = BufReader::new(stream);
     *     let http_request: Vec<_> = buf_reader
     *         .lines() // 迭代性适配器，不会消耗元素，是惰性的
     *         .map(|line| line.unwrap()) // 迭代性适配器，不会消耗元素，是惰性的
     *         .take_while(|line| !line.is_empty()) // 迭代性适配器，不会消耗元素，是惰性的
     *         .collect(); // 消费性适配器，依赖迭代器的 next 函数，用于消费元素
     *
     *     println!("{:#?}", http_request);
     * }
     * ```
     * // TODO lines 方法是一个迭代性适配器，作用是将字节流按照 `\r\n | \n` 分割，供后续迭代器使用
     * // TODO map 是常用的迭代性适配器，在这里将 lines 方法分割的元素从 Result 中取出
     * // TODO take_while 是一个迭代性适配器，它不同于 filter，take_while 具有终止作用，即无论给多少数据，只要遇到第一个 false 条件，take_while 就会终止
     * // TODO collect 是最常用的消费性适配器，用于收集数据，需要在变量上提前标注收集类型，或者使用 turbofish collect::<Vec<_>>() 语法
     * 
     * // TODO 最终可以看到请求的报文，这就是常见的 http 请求报文，具体来看，它分为四部分：
     * // TODO http 报文解释
     *
     * 控制台打印的报文，具体看代码区域：
     * ```shell
     *
     * ```
     *
     *
     */

    // {
    //     let listener =
    //         net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error!");
    //     for stream in listener.incoming() {
    //         let stream = stream.unwrap();
    //         println!("Connection established!");
    //     }
    // }

    {
        let listener =
            net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error!");
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            println!("Connection established!");

            let buf_reader = BufReader::new(stream);
            let http_request: Vec<_> = buf_reader
                .lines() // 迭代性适配器，不会消耗元素，是惰性的
                .map(|line| line.unwrap()) // 迭代性适配器，不会消耗元素，是惰性的
                .take_while(|line| !line.is_empty()) // 迭代性适配器，不会消耗元素，是惰性的
                .collect(); // 消费性适配器，依赖迭代器的 next 函数，用于消费元素

            println!("{:#?}", http_request);

            // [
            //     "GET / HTTP/1.1",
            //     "Host: 127.0.0.1:7878",
            //     "Connection: keep-alive",
            //     "Cache-Control: max-age=0",
            //     "sec-ch-ua: \"Not/A)Brand\";v=\"8\", \"Chromium\";v=\"126\", \"Microsoft Edge\";v=\"126\"",
            //     "sec-ch-ua-mobile: ?0",
            //     "sec-ch-ua-platform: \"Windows\"",
            //     "Upgrade-Insecure-Requests: 1",
            //     "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 Edg/126.0.0.0",
            //     "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
            //     "Sec-Fetch-Site: none",
            //     "Sec-Fetch-Mode: navigate",
            //     "Sec-Fetch-User: ?1",
            //     "Sec-Fetch-Dest: document",
            //     "Accept-Encoding: gzip, deflate, br, zstd",
            //     "Accept-Language: zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6",
            //     "Cookie: _ga=GA1.1.89819984.1702807852; _ga_L7WEXVQCR9=GS1.1.1702807851.1.1.1702807915.0.0.0",
            // ]
        }
    }
}
