use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};

use bytes::Bytes;
use mini_redis::{
    Command::{self, Get, Set},
    Connection, Frame, Result,
};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    type DB = Arc<Mutex<HashMap<String, Bytes>>>;

    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let db: DB = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (stream, addr) = listener.accept().await?;
        let _db = Arc::clone(&db);
        tokio::spawn(async move {
            process(stream, _db).await;
        });
    }

    async fn process(stream: TcpStream, db: DB) {
        // `mini-redis` 提供的便利函数，使用返回的 `connection` 可以用于从 socket 中读取数据并解析为数据帧
        // 根据 stream 生成 Connection 实例，它支持以数据帧读取数据
        let mut connection = Connection::new(stream);

        // 在一个连接中可以传送多个帧数据，因此需要使用 while let 而不是 if let
        while let Some(frame) = connection.read_frame().await.unwrap() {
            println!("GOT: {}", frame);

            let response = match Command::from_frame(frame).unwrap() {
                Set(cmd) => {
                    // 值被存储为 `Vec<u8>` 的形式
                    db.lock()
                        .unwrap()
                        .insert(cmd.key().to_string(), cmd.value().clone());
                    Frame::Simple("OK".to_string())
                }
                Get(cmd) => {
                    // `Frame::Bulk` 期待数据的类型是 `Bytes`，`&Vec<u8>` 可以使用 `into()` 方法转换成 `Bytes` 类型
                    if let Some(value) = db.lock().unwrap().get(cmd.key()) {
                        Frame::Bulk(value.clone().into())
                    } else {
                        Frame::Null
                    }
                }
                cmd => panic!("unimpement {:?}", cmd),
            };

            connection.write_frame(&response).await.unwrap();
        }
    }
}
