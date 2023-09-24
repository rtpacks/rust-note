## Trait 继承

让类型去实现 Trait，使类型具备该 Trait 的功能，是组合（composite）的方式。

经常和组合放在一起讨论的是继承(inheritance)。继承通常用来描述属于同种性质的**父子关系(is a)**，而组合用来描述**具有某功能(has a)**。

例如，支持继承的语言，可以让轿车类型(Car)继承交通工具类型(Vehicle)，表明轿车是一种(is a)交通工具，它们是同一种性质的东西。而如果是支持组合的语言，可以定义可驾驶功能 Drivable，然后将 Driveable 组合到轿车类型、轮船类型、飞机类型、卡车类型、玩具车类型，等等，表明这些类型具有(has a)驾驶功能。

通过新编程语言的特性可以发现，类型功能的增加，组合方式（composite）是优于继承（inheritance）的。

Rust 除了支持组合，还支持继承。但**Rust 只支持 Trait 之间的继承**，比如 Trait A 继承 Trait B，类型没有继承的概念。

实现继承的方式很简单，在定义 Trait A 时使用冒号加上 Trait B 即可。如果 Trait A 继承 Trait B，当类型 C 想要实现 Trait A 时，将要求同时也要去实现 B。

```rust
trait B{}
trait A: B{}

struct C{}
// C实现Trait A
impl A for C {
  fn func_in_a(&self){
    println!("impl: func_in_a");
  }
}
// C还要实现Trait B
impl B for C {
  fn func_in_b(&self){
    println!("impl: func_in_b");
  }
}
```

### code

```rust
fn main {
    trait Drivable {
        fn run(&self) {
            println!("running")
        }
        fn stop(&self) {
            println!("stopped")
        }
    }

    struct Car {
        name: String,
    }

    impl Drivable for Car {
        fn run(&self) {
            println!("{} is running", self.name);
        }
        fn stop(&self) {
            println!("{} is stopped", self.name)
        }
    }

    let car = Car {
        name: String::from("Benz"),
    };

    car.run();
    car.stop();
}
```
