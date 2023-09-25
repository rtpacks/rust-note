## 使用 Trait Object 类型

了解 Trait Object 之后，使用它就不再难了，它也只是一种数据类型罢了。
Trait Object 最常用在只明确了需要使用功能，但不明确具体是什么对象的情况上。
比如组成页面的元素必须要具备 draw 方法才能被调用然后绘制到页面上，像 button、input、text 等。
又比如一个插件系统，需要插件去实现指定的生命周期钩子和 install 方法，这样插件才能被正常注册使用。
这些都是明确了调用的 hook 方法，但是没有固定调用的对象。

如以下 Audio 和 Video 实现了 Trait Playable

```rs
trait Playable {
  fn play(&self);
  fn pause(&self) {println!("pause");}
  fn get_duration(&self) -> f32;
}

// Audio类型，实现Trait Playable
struct Audio {name: String, duration: f32}
impl Playable for Audio {
  fn play(&self) {println!("listening audio: {}", self.name);}
  fn get_duration(&self) -> f32 {self.duration}
}

// Video类型，实现Trait Playable
struct Video {name: String, duration: f32}
impl Playable for Video {
  fn play(&self) {println!("watching video: {}", self.name);}
  fn pause(&self) {println!("video paused");}
  fn get_duration(&self) -> f32 {self.duration}
}
```

现在，可以将 Audio 的实例或 Video 的实例当作 Playable 的 Trait Object 来使用。

一定记住：将类型的实体转为 Trait Object 后，只能调用实现于 Trait T 的方法，而不能调用类型本身实现的方法和类型实现于其他 Trait 的方法。
也就是说 vtable 只记录了当前 Trait 的方法。

```rs
let x: &dyn Playable = &Audio{
  name: "telephone.mp3".to_string(),
  duration: 3.42,
};
x.play();

let y: &dyn Playable = &Video{
  name: "Yui Hatano.mp4".to_string(),
  duration: 59.59,
};
y.play();
```

此时，x 的数据类型是 Playable 的 Trait Object 类型的引用，它在栈中保存了一个指向 Audio 实例数据的指针，还保存了一个指向包含了它可调用方法的 vtable 的指针。同理，y 也一样。

再比如，有一个 Playable 的 Trait Object 类型的数组，在这个数组中可以存放所有实现了 Playable 的实例对象数据：

```rs
let a:&dyn Playable = &Audio{
  name: "telephone.mp3".to_string(),
  duration: 3.42,
};

let b: &dyn Playable = &Video {
  name: "Yui Hatano.mp4".to_string(),
  duration: 59.59,
};

let arr: [&dyn Playable;2] = [a, b];
println!("{:#?}", arr);
```

注意，上面为了使用 println!的调试输出格式{:#?}，要让 Playable 实现名为 std::fmt::Debug 的 Trait，因为 Playable 自身也是一个 Trait，所以使用 Trait 继承的方式来继承 Debug。继承 Debug 后，要求实现 Playable Trait 的类型也都要实现 Debug Trait，因此在 Audio 和 Video 之前使用#[derive(Debug)]来实现 Debug Trait。

### code

```rs
fn main {
    trait Playable: Debug {
        fn play(&self);
        fn pause(&self) {
            println!("pause");
        }
        fn get_duration(&self) -> f32;
    }

    // Audio类型，实现Trait Playable
    #[derive(Debug)]
    struct Audio {
        name: String,
        duration: f32,
    }
    impl Playable for Audio {
        fn play(&self) {
            println!("listening audio: {}", self.name);
        }
        fn get_duration(&self) -> f32 {
            self.duration
        }
    }

    // Video类型，实现Trait Playable
    #[derive(Debug)]
    struct Video {
        name: String,
        duration: f32,
    }
    impl Playable for Video {
        fn play(&self) {
            println!("watching video: {}", self.name);
        }
        fn pause(&self) {
            println!("video paused");
        }
        fn get_duration(&self) -> f32 {
            self.duration
        }
    }

    let audio = Audio {
        name: "夜空中最亮的星".to_string(),
        duration: 4.0,
    };

    let video = Video {
        name: "光之国".to_string(),
        duration: 118.0,
    };

    let a: &dyn Playable = &audio;
    let v: &dyn Playable = &video;

    let playables = [a, v];

    for x in playables {
        println!("{}", x.get_duration())
    }

    println!("{:#?}", playables);
}
```
