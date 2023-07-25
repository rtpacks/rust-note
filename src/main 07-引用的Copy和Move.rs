fn main() {
    /*
     * ## 引用类型的Copy和Clone
     * 值得注意的是，对一个引用使用clone方法，可能会产生不同的结果。
     * https://rust-book.junmajinlong.com/ch6/06_ref_copy_clone.html
     *
     * 具体来说，一个引用的clone可以分为两类
     * - 对引用自身的clone，即引用类型的Copy
     * - 对引用指向数据的clone，即对指向数据的Clone
     * 两类方式的实现主要看指向数据是否实现了Clone，如果指向数据实现了Clone，那么引用的clone就是指向数据本身的clone
     *
     * ```rs
     *
     * struct Person;
     *
     * let a = Person;
     * let b = &a;
     * let c = b.clone(); // 和 let c = b; 一样，实现的是对引用本身的clone
     *
     *
     * // 如果Person实现了Clone
     * #[derive(Clone)]
     * struct Person;
     *
     * let a = Person;
     * let b = &a;
     * let c = b.clone() // 这是指向数据的clone，c的类型是Person。这是由于方法调用的符号.会自动解引用，首先看指向数据是否实现了clone，如果没有则clone引用。即调用方法时是有优先级的，找到即停。虽然方法名称一致，但是实现的效果不一样。
     * ```
     *
     * ### 注意
     * 方法调用的符号.会自动解引用，首先看指向数据是否实现了clone，如果没有则clone引用。即调用方法时是有优先级的，找到即停。虽然方法名称一致，但是实现的效果不一样。
     *
     * > 例如，某方法要求返回Person类型，但在该方法内部却只能取得Person的引用类型(比如从HashMap的get()方法只能返回值的引用)，所以需要将引用&Person转换为Person，直接解引用是一种可行方案，但是对未实现Copy的类型去解引用，将会执行Move操作，很多时候这是不允许的，比如不允许将已经存入HashMap中的值Move出来，此时最简单的方式，就是通过克隆引用的方式得到Person类型。
     *
     * 提醒：正因为从集合(比如HashMap、BTreeMap等)中取数据后很有可能需要对取得的数据进行克隆，因此建议不要将大体量的数据存入集合，如果确实需要克隆集合中的数据的话，这将会严重影响性能。
     *
     * 作为建议，可以考虑先将大体量的数据封装在智能指针(比如Box、Rc等)的背后，再将智能指针存入集合。
     *
     * 其它语言中集合类型的使用可能非常简单直接，但Rust中需要去关注这一点。
     *
     * 总结起来，在Rust中由于所有权的存在，在数据传递的过程中，引用更为常见。
     * - 对未实现Copy的类型去解引用，将会执行Move操作（这在 [unit 06-再次理解Move.md](../docs/unit 06-再次理解Move.md) 提及），很多时候这是不允许的，比如不允许将已经存入HashMap中的值Move出来
     * - 为了能够不使用值（转移所有权）的情况下，方便使用值进行操作，就可以使用这种形式。它不转移所有权，只是clone。
     */
    #[derive(Clone)]
    struct Person;

    let p = Person;

    let ptr = &p;

    let b = ptr;
    let c = ptr.clone();

    println!("{:p}, {:p}", b, &c)
}
