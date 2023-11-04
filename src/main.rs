use core::panic;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::ErrorKind;
use std::io::{self, Read};

fn main() {
    /*
     * ## 单元 Module
     * 一个 `src/lib.rs` 或 `src/main.rs` 就是一个包Crate，一个包可能是多种功能的集合体。为了项目工程Package的组织维护，需要对包进行拆分。
     *
     * 模块Module（mod）是**rust代码的构成单元**，是代码拆分的单位（文件/文件夹）。
     * 使用模块可以将包中的代码按照功能性进行重组，而不需要将所有代码都写在 `src/lib.rs` 或者 `src/main.rs`，最终实现更好的可读性及易用性。
     * 同时，使用mod还可以非常灵活地去控制代码的可见性，进一步强化 Rust 的安全性。
     * 
     * 
     * 
     * 
     *
     * 以lib类型的crate为例，该crate的入口在src/lib.rs，也是crate的根。
     */
}
