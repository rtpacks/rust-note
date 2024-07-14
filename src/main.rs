use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net, thread,
    time::Duration,
};

use ilearn::threadpool::ThreadPool;

use futures::stream;

fn main() {
    /*
     *
     * ## 实战：多线程 Web 服务器
     * 单线程版本只能依次处理用户的请求：同一时间只能处理一个请求连接。随着用户的请求数增多，可以预料的是排在后面的用户可能要等待数十秒甚至超时！
     *
     * ### 模拟慢请求
     * 在单线程版本中，为一个请求增加 5 秒阻塞，前一个请求发起后，再一次发起访问请求，第二个请求就需要等待两个阻塞时间间隔，也就是 10s。
     *
     * ```rust
     * let listener =
     *     net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");
     * for stream in listener.incoming() {
     *     let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
     *     println!("Connection established!");
     *
     *     let buf_reader = BufReader::new(&stream);
     *     let http_request: Vec<_> = buf_reader
     *         .lines()
     *         .map(|line| line.unwrap())
     *         .take_while(|line| !line.is_empty())
     *         .collect();
     *
     *     let (status_line, html) = if &http_request[0] == "GET / HTTP/1.1" {
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
     *     let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
     *     let response_body = html;
     *     let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");
     *
     *     thread::sleep(Duration::from_secs(5));
     *     stream.write_all(http_response.as_bytes());
     * }
     * ```
     *
     * 单线程是处理是很不合理的，需要解决这个问题。
     *
     * ### 多线程提高吞吐量
     * 线程池（thread pool）是一组预先分配的等待或准备处理任务的线程。线程池允许程序并发处理连接，增加 server 的吞吐量。
     * 当程序收到一个新任务时就会指派线程池中的一个线程离开并处理该任务。
     * 当线程仍在处理任务时，新来的任务会交给池中剩余的线程进行处理。当线程处理完任务时，它会返回空闲线程池中等待处理新任务。
     *
     * 同时，线程池需要限制为较少的数量的线程，以防拒绝服务攻击（Denial of Service，DoS）。
     * 假设程序为每一个接收的请求都新建一个线程，那么某人向 server 发起千万级的请求时会耗尽服务器的资源并导致所有请求的处理都被终止。
     *
     * 当然，线程池依然是较为传统的提升吞吐方法，比较新的有单线程异步 IO，例如 redis；多线程异步 IO，例如 Rust 的主流 web 框架。
     *
     * 为每个请求生成一个线程，这种做法难以控制且消耗资源：
     * ```rust
     * let listener =
     *     net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");
     *
     * fn handle_request(mut stream: net::TcpStream) {
     *     let buf_reader = BufReader::new(&stream);
     *     let http_request: Vec<_> = buf_reader
     *         .lines()
     *         .map(|line| line.unwrap())
     *         .take_while(|line| !line.is_empty())
     *         .collect();
     *
     *     let (status_line, html) = if &http_request[0] == "GET / HTTP/1.1" {
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
     *     let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
     *     let response_body = html;
     *     let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");
     *
     *     thread::sleep(Duration::from_secs(5));
     *     stream.write_all(http_response.as_bytes());
     * }
     *
     * for stream in listener.incoming() {
     *     let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
     *     println!("Connection established!");
     *
     *     // 每个请求都生成一个新线程去处理任务，这种做法开销过大，在请求量大时，很容易造成资源不足
     *     let handle = thread::spawn(move || {
     *         handle_request(stream);
     *     });
     * }
     * ```
     *
     * 设想给出一个线程池，存储已经生成好的线程，当任务到达后可以直接从线程池中取出线程运行任务，这样避免了等待线程生成的时间。同时在任务结束后不会销毁线程，而是将线程归还给线程池，用于下一次任务处理。
     * 此外为避免线程数量急速增加，可以设置线程池的线程数量。通过线程池，可以避免每个请求都生成一个新线程方案的很多问题。
     *
     * 在开始之前，这里有一种开发模式，与前后端先接口约定后具体开发的模式、设计数据库表画出 ER 图的流程是类似的，都是先设想整体与局部的功能划分，然后再具体实现局部的功能。
     *
     * 模式描述：在最初就约定客户端接口有助于指导代码设计。以期望的调用方式来构建 API 代码的结构，接着在这个结构之内实现功能。
     * 这种模式称为编译器驱动开发（compiler-driven development）。
     *
     * 具体行为：编写调用所期望的函数的代码，接着观察编译器错误然后决定接下来需要修改什么使得代码可以工作。
     * 这一种方式并不会探索作为起点的技术，而是按照业务流程一步一步补齐。
     *
     * ```rust
     * let listener =
     *     net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");
     *
     * // 生成有 5 个线程的线程池
     * let pool = ThreadPool::new(5);
     *
     * ...
     * ```
     *
     * TODO
     *
     */

    // {
    //     let listener =
    //         net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");
    //     for stream in listener.incoming() {
    //         let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
    //         println!("Connection established!");

    //         let buf_reader = BufReader::new(&stream);
    //         let http_request: Vec<_> = buf_reader
    //             .lines()
    //             .map(|line| line.unwrap())
    //             .take_while(|line| !line.is_empty())
    //             .collect();

    //         let (status_line, html) = if &http_request[0] == "GET / HTTP/1.1" {
    //             (
    //                 "HTTP/1.1 200 OK",
    //                 fs::read_to_string(r"public/http-response-index.html").unwrap(),
    //             )
    //         } else {
    //             (
    //                 "HTTP/1.1 404 NOT FOUND",
    //                 fs::read_to_string(r"public/http-response-404.html").unwrap(),
    //             )
    //         };

    //         let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
    //         let response_body = html;
    //         let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");

    //         thread::sleep(Duration::from_secs(5));
    //         stream.write_all(http_response.as_bytes());
    //     }
    // }

    // {
    //     let listener =
    //         net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");

    //     fn handle_request(mut stream: net::TcpStream) {
    //         let buf_reader = BufReader::new(&stream);
    //         let http_request: Vec<_> = buf_reader
    //             .lines()
    //             .map(|line| line.unwrap())
    //             .take_while(|line| !line.is_empty())
    //             .collect();

    //         let (status_line, html) = if &http_request[0] == "GET / HTTP/1.1" {
    //             (
    //                 "HTTP/1.1 200 OK",
    //                 fs::read_to_string(r"public/http-response-index.html").unwrap(),
    //             )
    //         } else {
    //             (
    //                 "HTTP/1.1 404 NOT FOUND",
    //                 fs::read_to_string(r"public/http-response-404.html").unwrap(),
    //             )
    //         };

    //         let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
    //         let response_body = html;
    //         let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");

    //         thread::sleep(Duration::from_secs(5));
    //         stream.write_all(http_response.as_bytes());
    //     }

    //     for stream in listener.incoming() {
    //         let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
    //         println!("Connection established!");

    //         // 每个请求都生成一个新线程去处理任务，这种做法开销过大，在请求量大时，很容易造成资源不足
    //         let handle = thread::spawn(move || {
    //             handle_request(stream);
    //         });
    //     }
    // }

    {
        let listener =
            net::TcpListener::bind("127.0.0.1:7878").expect("TcpListener started with an error");

        // 生成有 5 个线程的线程池
        let pool = ThreadPool::new(5);

        fn handle_request(mut stream: net::TcpStream) {
            let buf_reader = BufReader::new(&stream);
            let http_request: Vec<_> = buf_reader
                .lines()
                .map(|line| line.unwrap())
                .take_while(|line| !line.is_empty())
                .collect();

            let (status_line, html) = if &http_request[0] == "GET / HTTP/1.1" {
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

            let response_head = format!("Content-Type:text/html\r\nContent-Length:{}", html.len());
            let response_body = html;
            let http_response = format!("{status_line}\r\n{response_head}\r\n\r\n{response_body}");

            thread::sleep(Duration::from_secs(5));
            stream.write_all(http_response.as_bytes());
        }

        for stream in listener.incoming() {
            let mut stream = stream.unwrap(); // 处理连接请求，如果连接请求不成功则报错
            println!("Connection established!");
        }
    }
}
