use std::fmt::Debug;

fn main() {
    /*
     * ## 使用Trait Object类型
     * 了解Trait Object之后，使用它就不再难了，它也只是一种数据类型罢了。
     * Trait Object最常用在只明确了需要使用功能，但不明确具体是什么对象的情况上。
     * 比如组成页面的元素必须要具备draw方法才能被调用然后绘制到页面上，像button、input、text等。
     * 又比如一个插件系统，需要插件去实现指定的生命周期钩子和install方法，这样插件才能被正常注册使用。
     * 这些都是明确了调用的hook方法，但是没有固定调用的对象。
     *
     * 如以下Audio和Video实现了Trait Playable
     *
     * ```rs
     * trait Playable {
     *   fn play(&self);
     *   fn pause(&self) {println!("pause");}
     *   fn get_duration(&self) -> f32;
     * }
     *
     * // Audio类型，实现Trait Playable
     * struct Audio {name: String, duration: f32}
     * impl Playable for Audio {
     *   fn play(&self) {println!("listening audio: {}", self.name);}
     *   fn get_duration(&self) -> f32 {self.duration}
     * }
     *
     * // Video类型，实现Trait Playable
     * struct Video {name: String, duration: f32}
     * impl Playable for Video {
     *   fn play(&self) {println!("watching video: {}", self.name);}
     *   fn pause(&self) {println!("video paused");}
     *   fn get_duration(&self) -> f32 {self.duration}
     * }
     * ```
     * 现在，可以将Audio的实例或Video的实例当作Playable的Trait Object来使用。
     *
     * 一定记住：将类型的实体转为Trait Object后，只能调用实现于Trait T的方法，而不能调用类型本身实现的方法和类型实现于其他Trait的方法。
     * 也就是说vtable只记录了当前Trait的方法。
     *
     * ```rs
     * let x: &dyn Playable = &Audio{
     *   name: "telephone.mp3".to_string(),
     *   duration: 3.42,
     * };
     * x.play();
     *
     * let y: &dyn Playable = &Video{
     *   name: "Yui Hatano.mp4".to_string(),
     *   duration: 59.59,
     * };
     * y.play();
     * ```
     * 此时，x的数据类型是Playable的Trait Object类型的引用，它在栈中保存了一个指向Audio实例数据的指针，还保存了一个指向包含了它可调用方法的vtable的指针。同理，y也一样。
     *
     * 再比如，有一个Playable的Trait Object类型的数组，在这个数组中可以存放所有实现了Playable的实例对象数据：
     * ```rs
     * let a:&dyn Playable = &Audio{
     *   name: "telephone.mp3".to_string(),
     *   duration: 3.42,
     * };
     *
     * let b: &dyn Playable = &Video {
     *   name: "Yui Hatano.mp4".to_string(),
     *   duration: 59.59,
     * };
     *
     * let arr: [&dyn Playable;2] = [a, b];
     * println!("{:#?}", arr);
     * ```
     *
     * 注意，上面为了使用println!的调试输出格式{:#?}，要让Playable实现名为std::fmt::Debug的Trait，因为Playable自身也是一个Trait，所以使用Trait继承的方式来继承Debug。继承Debug后，要求实现Playable Trait的类型也都要实现Debug Trait，因此在Audio和Video之前使用#[derive(Debug)]来实现Debug Trait。
     */

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
