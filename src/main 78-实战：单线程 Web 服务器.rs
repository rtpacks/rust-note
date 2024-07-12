use std::{
    fs,
    io::{BufRead, BufReader, Write},
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
     * ### 解析报文
     * 在请求连接建立后，需要解析客户端发送的数据，并给出对应的响应，这次请求才算是完整的。
     *
     * > 阅读 HTTP 协议报文：
     * > - https://developer.mozilla.org/zh-CN/docs/Web/HTTP/Overview#http_报文
     * > - https://cloud.tencent.com/developer/article/1953222
     * > - https://web.archive.org/web/20240712035832/https://cloud.tencent.com/developer/article/1953222
     * > 回车符 `\r`，换行符 `\n`
     *
     * #### 解析请求报文
     * 现在先看如何获取客户端发送的报文。TcpStream 返回的是字节流，这里使用 BufReader 读取报文，并使用给定的迭代器简化实现读取逻辑：
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
     * > - 迭代器是 Rust 的 零成本抽象（zero-cost abstractions）之一，意味着抽象并不会引入运行时开销
     * > - 迭代器分为消费性和迭代性适配器，消费性适配器（内部调用 next 方法）消耗元素，迭代性适配器返回一个迭代器
     * > - 迭代器最终需要一个消费性适配器来收尾，最终将迭代器转换成一个具体的值
     *
     * 各种迭代器的作用：
     * - lines 是一个迭代性适配器，作用是将字节流按照 `\r\n | \n` 分割，供后续迭代器使用
     * - map 是一个常用的迭代性适配器，在这里将 lines 方法分割的元素从 Result 中取出
     * - take_while 是一个迭代性适配器，作用是过滤非空白的字符串。take_while 不同于 filter，它具有终止作用，即无论给多少数据，只要遇到第一个 false 条件，take_while 就会终止
     * - collect 是最常用的消费性适配器，作用是收集数据，需要在变量上提前标注收集类型，或者使用 turbofish collect::<Vec<_>>() 语法
     *
     * 最终可以看到请求的报文，这就是常见的 http 请求报文。控制台打印的报文，具体看代码区域：
     * ```shell
     * [
     *     "GET / HTTP/1.1",
     *     "Host: 127.0.0.1:7878",
     *      ...
     *     "Accept-Language: zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6",
     * ]
     * ```
     *
     * 按照 **HTTP 协议**的定义，请求报文分为四部分，请求行、请求头、空行、请求体，其中请求体是可选的：
     * ```shell
     * Method Request-URI HTTP-Version CRLF
     * headers CRLF
     * CRLF
     * message-body
     * ```
     * > 请求头部的最后会有一个空行，表示请求头部结束，接下来为请求正文，这一行非常重要，必不可少。
     *
     * 对比控制台输出的报文，由于使用 take_while 过滤了空行后的所有内容，所以输出的报文只有请求行和请求头：
     * |      协议                       |       报文       |
     * | -----------------------         | -------------   |
     * | Method Request-URI HTTP-Version | GET / HTTP/1.1  |
     * | Host: 127.0.0.1:7878            | headers         |
     *
     * #### 返回响应报文
     * 与客户端发送的请求报文类似，服务器应该返回响应给客户端，HTTP 对响应报文也有定义。
     * 具体来看，HTTP 响应报文由状态行、响应头部、空行、响应正文 4 部分组成，其中响应正文是可选的。
     *
     * ```shell
     * HTTP-Version Status-Code Reason-Phrase CRLF
     * headers CRLF
     * CRLF
     * message-body
     * ```
     * 应答的格式与请求相差不大，其中 Status-Code 是最重要的，它用于告诉客户端，当前的请求是否成功，若失败，大概是什么原因，它就是著名的 HTTP 状态码，常用的有 200: 请求成功，404 目标不存在，等等。
     *
     * 构造一个简单的 http 响应返回给浏览器，让浏览器认为连接正常完成，避免出现错误状态页面：
     * ```rust
     * let listener =
     *     net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error!");
     * for stream in listener.incoming() {
     *     let mut stream = stream.unwrap();
     *     println!("Connection established!");
     *
     *     let buf_reader = BufReader::new(&stream);
     *     let http_request: Vec<_> = buf_reader
     *         .lines() // 迭代性适配器，不会消耗元素，是惰性的
     *         .map(|line| line.unwrap()) // 迭代性适配器，不会消耗元素，是惰性的
     *         .take_while(|line| !line.is_empty()) // 迭代性适配器，不会消耗元素，是惰性的
     *         .collect(); // 消费性适配器，依赖迭代器的 next 函数，用于消费元素
     *
     *     println!("{:#?}", http_request);
     *
     *     let http_response = "HTTP/1.1 200 OK\r\n\r\n";
     *
     *     stream.write_all(http_response.as_bytes());
     * }
     * ```
     * `"HTTP/1.1 200 OK\r\n\r\n"` 返回空的响应正文，现在增加字符串正文，让浏览器显式文字 `Hello World`。
     * ```rust
     * let http_response = "HTTP/1.1 200 OK\r\n\r\nHello World";
     * ```
     *
     * 接着就能看到浏览器显式 `Hello World` 文字，查看接口返回响应是字符串 `Hello World`。
     * 可以再尝试一下返回 JSON 数据，并且指定响应头 `Content-Type:application/json;`：
     *
     * ```rust
     * let http_response = "HTTP/1.1 200 OK\r\nContent-Type:application/json;\r\n\r\n{ \"name\": \"Mr.F\", \"age\": 18 }";
     * ```
     *
     * 这里的 stream.write_all 和 BufReader 一样使用的是字节流，所以需要使用 `as_bytes()` 将字符串转换为字节。
     *
     * ##### 返回 HTML
     * 只返回字符串会有点粗糙，现代网页基本上都是由 HTML (骨架)、CSS (样式)、JavaScript (行为) 构成的。这里实现将本地的 HTML 返回给浏览器。
     *
     * ```rust
     * let listener =
     *     net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error!");
     * for stream in listener.incoming() {
     *     let mut stream = stream.unwrap();
     *     println!("Connection established!");
     *
     *     let buf_reader = BufReader::new(&stream);
     *     let http_request: Vec<_> = buf_reader
     *         .lines() // 迭代性适配器，不会消耗元素，是惰性的
     *         .map(|line| line.unwrap()) // 迭代性适配器，不会消耗元素，是惰性的
     *         .take_while(|line| !line.is_empty()) // 迭代性适配器，不会消耗元素，是惰性的
     *         .collect(); // 消费性适配器，依赖迭代器的 next 函数，用于消费元素
     *
     *     println!("{:#?}", http_request);
     *
     *     let status_line = "HTTP/1.1 200 OK";
     *     // 以项目根路径为标准
     *     let html = fs::read_to_string(r"public/http-response.html").unwrap();
     *     let response_head = format!(
     *         "Content-Type:{}\r\nContent-Length:{}",
     *         "text/html",
     *         html.len()
     *     );
     *     let response_body = html;
     *     let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");
     *
     *     stream.write_all(http_response.as_bytes());
     * }
     * ```
     * web 框架(例如 rocket)将解析请求数据和返回应答数据都封装在 API 中，非常简单易用，无需开发者手动编写。
     *
     * ### 验证请求和选择性应答
     * 以上所有的请求都会返回同一个响应，很明显这是不合理的，现在模拟不同的请求返回不同的响应。
     * 规定只能返回根路径 `/`，访问其他路径返回 404 状态。
     *
     * ```rust
     * let listener =
     *     net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error!");
     * for stream in listener.incoming() {
     *     let mut stream = stream.unwrap();
     *     println!("Connection established!");
     *
     *     let buf_reader = BufReader::new(&stream);
     *     let http_request: Vec<_> = buf_reader
     *         .lines() // 迭代性适配器，不会消耗元素，是惰性的
     *         .map(|line| line.unwrap()) // 迭代性适配器，不会消耗元素，是惰性的
     *         .take_while(|line| !line.is_empty()) // 迭代性适配器，不会消耗元素，是惰性的
     *         .collect(); // 消费性适配器，依赖迭代器的 next 函数，用于消费元素
     *     println!("{:#?}", http_request);
     *
     *     let (status_line, html) = if http_request[0] == "GET / HTTP/1.1" {
     *         (
     *             "HTTP/1.1 200 OK",
     *             fs::read_to_string(r"public/http-response-index.html").unwrap(),
     *         )
     *     } else {
     *         (
     *             "HTTP/1.1 404 NOT FOUND",
     *             fs::read_to_string(r"public/http-response-404.html").unwrap(),
     *         )
     *     };
     *
     *     // 以项目根路径为标准
     *     let response_head = format!(
     *         "Content-Type:{}\r\nContent-Length:{}",
     *         "text/html",
     *         html.len()
     *     );
     *     let response_body = html;
     *     let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");
     *
     *     stream.write_all(http_response.as_bytes());
     * }
     * ```
     *
     */

    {
        let listener =
            net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error!");
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            println!("Connection established!");
        }
    }

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
            // ]
        }
    }

    {
        let listener =
            net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error!");
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            println!("Connection established!");

            let buf_reader = BufReader::new(&stream);
            let http_request: Vec<_> = buf_reader
                .lines() // 迭代性适配器，不会消耗元素，是惰性的
                .map(|line| line.unwrap()) // 迭代性适配器，不会消耗元素，是惰性的
                .take_while(|line| !line.is_empty()) // 迭代性适配器，不会消耗元素，是惰性的
                .collect(); // 消费性适配器，依赖迭代器的 next 函数，用于消费元素

            println!("{:#?}", http_request);

            // let http_response = "HTTP/1.1 200 OK\r\n\r\n";
            let http_response = "HTTP/1.1 200 OK\r\n\r\nHello World";
            let http_response = "HTTP/1.1 200 OK\r\nContent-Type:application/json;\r\n\r\n{ \"name\": \"Mr.F\", \"age\": 18 }";

            stream.write_all(http_response.as_bytes());
        }
    }

    {
        let listener =
            net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error!");
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            println!("Connection established!");

            let buf_reader = BufReader::new(&stream);
            let http_request: Vec<_> = buf_reader
                .lines() // 迭代性适配器，不会消耗元素，是惰性的
                .map(|line| line.unwrap()) // 迭代性适配器，不会消耗元素，是惰性的
                .take_while(|line| !line.is_empty()) // 迭代性适配器，不会消耗元素，是惰性的
                .collect(); // 消费性适配器，依赖迭代器的 next 函数，用于消费元素

            println!("{:#?}", http_request);

            let status_line = "HTTP/1.1 200 OK";
            // 以项目根路径为标准
            let html = fs::read_to_string(r"public/http-response.html").unwrap();
            let response_head = format!(
                "Content-Type:{}\r\nContent-Length:{}",
                "text/html",
                html.len()
            );
            let response_body = html;
            let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");

            stream.write_all(http_response.as_bytes());
        }
    }

    {
        let listener =
            net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error!");
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            println!("Connection established!");

            let buf_reader = BufReader::new(&stream);
            let http_request: Vec<_> = buf_reader
                .lines() // 迭代性适配器，不会消耗元素，是惰性的
                .map(|line| line.unwrap()) // 迭代性适配器，不会消耗元素，是惰性的
                .take_while(|line| !line.is_empty()) // 迭代性适配器，不会消耗元素，是惰性的
                .collect(); // 消费性适配器，依赖迭代器的 next 函数，用于消费元素
            println!("{:#?}", http_request);

            let (status_line, html) = if http_request[0] == "GET / HTTP/1.1" {
                (
                    "HTTP/1.1 200 OK",
                    fs::read_to_string(r"public/http-response-index.html").unwrap(),
                )
            } else {
                (
                    "HTTP/1.1 404 NOT FOUND",
                    fs::read_to_string(r"public/http-response-404.html").unwrap(),
                )
            };

            // 以项目根路径为标准
            let response_head = format!(
                "Content-Type:{}\r\nContent-Length:{}",
                "text/html",
                html.len()
            );
            let response_body = html;
            let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");

            stream.write_all(http_response.as_bytes());
        }
    }
}
