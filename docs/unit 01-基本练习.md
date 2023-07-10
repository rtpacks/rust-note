```rs
fn main() {
    let southern_germany = "Grüß Gott!";
    let chinese = "世界，你好";
    let english = "world, hello";

    let regions = [southern_germany, chinese, english];

    for region in regions.iter() {
        println!("{}", &region); // 默认会解引用，即不打印地址，而是打印值
        println!("{:p}", &region); // 打印地址
    }

    let x = 3;
    let y = 5;

    assert_eq!(
        0.1 + 0.2,
        0.3,
        "we are test conditions {}, {}",
        0.1 + 0.2,
        0.3
    )
}
```
