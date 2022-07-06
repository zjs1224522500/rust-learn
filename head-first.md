## 安装
- Linux 下使用 rustup 工具安装
```
# 首先下载 rustup，可以直接选择默认选项进行安装
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
...
Rust is installed now. Great!
...
```
- Rust 对运行环境的依赖和 Go 语言很像，几乎所有环境都可以无需安装任何依赖直接运行。但是，Rust 会依赖 libc 和链接器 linker。安装 gcc 或 clang
- 检查 rust 是否安装成功，Linux 上失败了的话，可能需要重启一些 shell
```
root@shunzi-ca65:/shunzi/os/rust-learn# rustc -V
rustc 1.62.0 (a8314ef7d 2022-06-27)
```

### 卸载
- `rustup self uninstall`

## Cargo
- 一个包管理工具。提供了一系列的工具，从项目的建立、构建到测试、运行直至部署，为 Rust 项目的管理提供尽可能完整的手段。同时，与 Rust 语言及其编译器 rustc 紧密结合。
- 项目管理
  - 新建：`cargo new project_name`
    - 使用该命令成生成了相应的目录结构
  - 编译运行：`cargo run`
    - 首先对项目进行编译，然后再运行，因此它实际上等同于运行了两个指令（**默认使用了 debug 模式，编译快，运行慢**）
      - 编译：`cargo build`
      - 运行：`./target/debug/world_hello`
    - 使用 release 模式来编译运行，相应的会生成 release 目录
      - `cargo run --release`
      - `cargo build --release`
  - 检查代码是否能编译通过：`cargo check`
    - Rust 的编译速度是门学问
```Bash
root@shunzi-ca65:/shunzi/os/rust-learn# cargo new world_hello
     Created binary (application) `world_hello` package
root@shunzi-ca65:/shunzi/os/rust-learn# tree
.
├── .git
├── .gitignore
├── Cargo.toml
└── src
    └── main.rs

root@shunzi-ca65:/shunzi/os/rust-learn# cd world_hello/
root@shunzi-ca65:/shunzi/os/rust-learn/world_hello# cargo run
   Compiling world_hello v0.1.0 (/shunzi/os/rust-learn/world_hello)
    Finished dev [unoptimized + debuginfo] target(s) in 1.20s
     Running `target/debug/world_hello`
Hello, world!
root@shunzi-ca65:/shunzi/os/rust-learn/world_hello# cargo clean
root@shunzi-ca65:/shunzi/os/rust-learn/world_hello# cargo build
   Compiling world_hello v0.1.0 (/shunzi/os/rust-learn/world_hello)
    Finished dev [unoptimized + debuginfo] target(s) in 0.93s
root@shunzi-ca65:/shunzi/os/rust-learn/world_hello# ./target/debug/world_hello 
Hello, world!

root@shunzi-ca65:/shunzi/os/rust-learn/world_hello# cargo check
    Checking world_hello v0.1.0 (/shunzi/os/rust-learn/world_hello)
    Finished dev [unoptimized + debuginfo] target(s) in 0.73s
root@shunzi-ca65:/shunzi/os/rust-learn/world_hello# 
```
- 几个关键文件
  - Cargo.toml：是 cargo 特有的项目数据描述文件。它存储了项目的所有元配置信息，如果 Rust 开发者希望 Rust 项目能够按照期望的方式进行构建、测试和运行，那么，必须按照合理的方式构建 Cargo.toml。
    - package：包含了项目信息以及 Rust 大版本
    - dependencies：定义对应的依赖信息
  - Cargo.lock 文件是 cargo 工具根据同一项目的 toml 文件生成的项目依赖详细清单，因此我们一般不用修改它，只需要对着 Cargo.toml 文件撸就行了。
    - 当你的项目是一个可运行的程序时，就上传 Cargo.lock，如果是一个依赖库项目，那么请把它添加到 .gitignore 中

## 不仅仅是 Hello World
### 多国语言
```rust
fn greet_world() {
     let southern_germany = "Grüß Gott!";
     let chinese = "世界，你好";
     let english = "World, hello";
     let regions = [southern_germany, chinese, english];
     for region in regions.iter() {
             println!("{}", &region);
     }
 }

 fn main() {
     greet_world();
 }

```
- Rust 原生支持 UTF-8 编码的字符串，这意味着你可以很容易的使用世界各国文字作为字符串内容。
- 对于 println 来说，我们没有使用其它语言惯用的 %s、%d 来做输出占位符，而是使用 {}，因为 Rust 在底层帮我们做了大量工作，会自动识别输出数据的类型，例如当前例子，会识别为 String 类型。
- Rust 的集合类型不能直接进行循环，需要变成迭代器（这里是通过 .iter() 方法），才能用于迭代循环。
  - 实际上这段代码可以简写，在 2021 edition 及以后，支持直接写 `for region in regions`，原因会在迭代器章节的开头提到，是因为 for 隐式地将 regions 转换成迭代器。
### 另外一个实例
```rust
fn main() {
   let penguin_data = "\
   common name,length (cm)
   Little penguin,33
   Yellow-eyed penguin,65
   Fiordland penguin,60
   Invalid,data
   ";

   let records = penguin_data.lines();

   for (i, record) in records.enumerate() {
     if i == 0 || record.trim().len() == 0 {
       continue;
     }

     // 声明一个 fields 变量，类型是 Vec
     // Vec 是 vector 的缩写，是一个可伸缩的集合类型，可以认为是一个动态数组
     // <_>表示 Vec 中的元素类型由编译器自行推断，在很多场景下，都会帮我们省却不少功夫
     let fields: Vec<_> = record
       .split(',')
       .map(|field| field.trim())
       .collect();
     if cfg!(debug_assertions) {
         // 输出到标准错误输出
       eprintln!("debug: {:?} -> {:?}",
              record, fields);
     }

     let name = fields[0];
     // 1. 尝试把 fields[1] 的值转换为 f32 类型的浮点数，如果成功，则把 f32 值赋给 length 变量
     //
     // 2. if let 是一个匹配表达式，用来从=右边的结果中，匹配出 length 的值：
     //   1）当=右边的表达式执行成功，则会返回一个 Ok(f32) 的类型，若失败，则会返回一个 Err(e) 类型，if let 的作用就是仅匹配 Ok 也就是成功的情况，如果是错误，就直接忽略
     //   2）同时 if let 还会做一次解构匹配，通过 Ok(length) 去匹配右边的 Ok(f32)，最终把相应的 f32 值赋给 length
     //
     // 3. 当然你也可以忽略成功的情况，用 if let Err(e) = fields[1].parse::<f32>() {...}匹配出错误，然后打印出来，但是没啥卵用
     if let Ok(length) = fields[1].parse::<f32>() {
         // 输出到标准输出
         println!("{}, {}cm", name, length);
     }
   }
 }

```
- 几个特性
  - 方法语法：由于 Rust 没有继承，因此 Rust 不是传统意义上的面向对象语言，但是它却从 OO 语言那里偷师了方法的使用 `record.trim()`，`record.split(',')` 等。
  - 高阶函数编程：函数可以作为参数也能作为返回值，例如 .map(|field| field.trim())，这里 map 方法中使用闭包函数作为参数，也可以称呼为 匿名函数、lambda 函数。
  - 类型标注：`if let Ok(length) = fields[1].parse::<f32>()`，通过 ::<f32> 的使用，告诉编译器 length 是一个 f32 类型的浮点数。这种类型标注不是很常用，但是在编译器无法推断出你的数据类型时，就很有用了。
  - 条件编译：`if cfg!(debug_assertions)`，说明紧跟其后的输出（打印）只在 debug 模式下生效。
  - 隐式返回：Rust 提供了 return 关键字用于函数返回，但是在很多时候，我们可以省略它。因为 Rust 是 基于表达式的语言。