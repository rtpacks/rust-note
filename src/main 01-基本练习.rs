fn main() {
    let southern_germany = "Grüß Gott!";
    let chinese = "世界，你好";
    let english = "world, hello";

    let regions = [southern_germany, chinese, english];

    for region in regions.iter() {
        println!("{}", &region);
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
