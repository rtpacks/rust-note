use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;
    client.set("foo", "Hello".into()).await?;
    println!("从服务器获取的结果为：foo = {:?}", client.get("foo").await);
    println!("从服务器获取的结果为：bar = {:?}", client.get("bar").await);
    Ok(())
}
