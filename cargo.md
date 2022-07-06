## Cargo
- https://course.rs/cargo/intro.html
- Cargo 是包管理工具，可以用于依赖包的下载、编译、更新、分发等，与 Cargo 一样有名的还有 crates.io，它是社区提供的包注册中心：用户可以将自己的包发布到该注册中心，然后其它用户通过注册中心引入该包。
- Cargo 会在安装 Rust 的时候一并进行安装，无需我们手动的操作执行
### 基础
- package: 工程、项目
- Cargo.toml 被称之为清单( manifest )，包含了 Cargo 编译程序所需的所有元数据。
- 基础命令
  - cargo new hello_world
  - cargo build
  - cargo build --release
  - ./target/debug/hello_world
  - cargo run
  - cargo run --release

### 添加依赖
- crates.io 是 Rust 社区维护的中心化注册服务，用户可以在其中寻找和下载所需的包。对于 cargo 来说，默认就是从这里下载依赖。
- 如下所示添加两个依赖：
```
[package]
name = "hello_world"
version = "0.1.0"
edition = "2021"

[dependencies]
time = "0.1.12"
regex = "0.1.41"
```
- 运行 cargo build 来重新构建，首先 Cargo 会获取新的依赖以及依赖的依赖, 接着对它们进行编译并更新 Cargo.lock:
  - 依赖下载太慢 https://course.rs/first-try/slowly-downloading.html
  - 首先 Cargo 会获取新的依赖以及依赖的依赖, 接着对它们进行编译并更新 Cargo.lock:
```
root@shunzi-ca65:/shunzi/os/rust-learn/hello_world# cargo build
    Updating `ustc` index
  Downloaded aho-corasick v0.5.3 (registry `ustc`)
  Downloaded thread-id v2.0.0 (registry `ustc`)
  Downloaded thread_local v0.2.7 (registry `ustc`)
  Downloaded utf8-ranges v0.1.3 (registry `ustc`)
  Downloaded time v0.1.44 (registry `ustc`)
  Downloaded regex-syntax v0.3.9 (registry `ustc`)
  Downloaded winapi-build v0.1.1 (registry `ustc`)
  Downloaded regex v0.1.80 (registry `ustc`)
  Downloaded kernel32-sys v0.2.2 (registry `ustc`)
  Downloaded memchr v0.1.11 (registry `ustc`)
  Downloaded winapi v0.2.8 (registry `ustc`)
  Downloaded libc v0.2.126 (registry `ustc`)
  Downloaded 12 crates (1.7 MB) in 1.82s
   Compiling libc v0.2.126
   Compiling winapi-build v0.1.1
   Compiling winapi v0.2.8
   Compiling regex-syntax v0.3.9
   Compiling utf8-ranges v0.1.3
   Compiling kernel32-sys v0.2.2
   Compiling thread-id v2.0.0
   Compiling memchr v0.1.11
   Compiling time v0.1.44
   Compiling thread_local v0.2.7
   Compiling aho-corasick v0.5.3
   Compiling regex v0.1.80
   Compiling hello_world v0.1.0 (/shunzi/os/rust-learn/hello_world)
    Finished dev [unoptimized + debuginfo] target(s) in 1m 36s
```
- Cargo.lock 中包含了我们项目使用的所有依赖的准确版本信息。这个非常重要，未来就算 regexp 的作者升级了该包，我们依然会下载 Cargo.lock 中的版本，而不是最新的版本，只有这样，才能保证项目依赖包不会莫名其妙的因为更新升级导致无法编译。 当然，你还可以使用 cargo update 来手动更新包的版本。
  - cargo update -p regex   # 只更新 “regex”
- 此时，就可以在 src/main.rs 中使用新引入的 regexp 包:
```rust
use regex::Regex;

fn main() {
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    println!("Did our date match? {}", re.is_match("2014-01-01"));
}
```

### Package 结构
- 标准结构
  - Cargo.toml 和 Cargo.lock 保存在 package 根目录下
    - Cargo.lock 会详尽描述上一次成功构建的各种信息：环境状态、依赖、版本等等，Cargo 可以使用它提供确定性的构建环境和流程，无论何时何地。这种特性对于终端服务是非常重要的：能确定、稳定的在用户环境中运行起来是终端服务最重要的特性之一。
    - 
  - 源代码放在 src 目录下
  - 默认的 lib 包根是 src/lib.rs
  - 默认的二进制包根是 src/main.rs
    - 其它二进制包根放在 src/bin/ 目录下
  - 基准测试 benchmark 放在 benches 目录下
  - 示例代码放在 examples 目录下
  - 集成测试代码放在 tests 目录下
- bin、tests、examples 等目录路径都可以通过配置文件进行配置，它们被统一称之为 Cargo Target。
```Bash
.
├── Cargo.lock
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── main.rs
│   └── bin/
│       ├── named-executable.rs
│       ├── another-executable.rs
│       └── multi-file-executable/
│           ├── main.rs
│           └── some_module.rs
├── benches/
│   ├── large-input.rs
│   └── multi-file-bench/
│       ├── main.rs
│       └── bench_module.rs
├── examples/
│   ├── simple.rs
│   └── multi-file-example/
│       ├── main.rs
│       └── ex_module.rs
└── tests/
    ├── some-integration-tests.rs
    └── multi-file-test/
        ├── main.rs
        └── test_module.rs

```

### 测试和 CI
- Cargo 可以通过 cargo test 命令运行项目中的测试文件：它会在 src/ 底下的文件寻找单元测试，也会在 tests/ 目录下寻找集成测试。
  - 事实上，除了单元测试、集成测试，cargo test 还会编译 examples/ 下的示例文件以及文档中的示例。
```bash
root@shunzi-ca65:/shunzi/os/rust-learn/hello_world# cargo test
   Compiling hello_world v0.1.0 (/shunzi/os/rust-learn/hello_world)
    Finished test [unoptimized + debuginfo] target(s) in 0.68s
     Running unittests src/main.rs (target/debug/deps/hello_world-c62c56b0577820c0)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

#### CI - Github Actions
- https://course.rs/test/ci.html

### Cargo 缓存
- Cargo 使用了缓存的方式提升构建效率，当构建时，Cargo 会将已下载的依赖包放在 CARGO_HOME 目录下
  - 默认情况下，Cargo Home 所在的目录是 $HOME/.cargo/
- 几个文件的含义
  - config.toml 是 Cargo 的全局配置文件
  - credentials.toml 为 cargo login 提供私有化登录证书，用于登录 package 注册中心，例如 crates.io
  - .crates.toml, .crates2.json 这两个是隐藏文件，包含了通过 cargo install 安装的包的 package 信息，请不要手动修改！
- 目录的含义：
  - bin 目录包含了通过 cargo install 或 rustup 下载的包编译出的可执行文件。你可以将该目录加入到 $PATH 环境变量中，以实现对这些可执行文件的直接访问
  - git 中存储了 Git 的资源文件:
    - git/db，当一个包依赖某个 git 仓库时，Cargo 会将该仓库克隆到 git/db 目录下，如果未来需要还会对其进行更新
    - git/checkouts，若指定了 git 源和 commit，那相应的仓库就会从 git/db 中 checkout 到该目录下，因此同一个仓库的不同 checkout 共存成为了可能性
  - registry 包含了注册中心( 例如 crates.io )的元数据 和 packages
    - registry/index 是一个 git 仓库，包含了注册中心中所有可用包的元数据( 版本、依赖等 )
    - registry/cache 中保存了已下载的依赖，这些依赖包以 gzip 的压缩档案形式保存，后缀名为 .crate
    - registry/src，若一个已下载的 .crate 档案被一个 package 所需要，该档案会被解压缩到 registry/src 文件夹下，最终 rustc 可以在其中找到所需的 .rs 文件

### Build 缓存
- cargo build 的结果会被放入项目根目录下的 target 文件夹中，当然，这个位置可以三种方式更改：
  - 设置 CARGO_TARGET_DIR 环境变量
  - build.target-dir 配置项
  - --target-dir 命令行参数。
- Target 目录结构
  - profile: debug/release/test/foo
  - 当使用 --target XXX 为特定的平台编译后，输出会放在 target/XXX/ 目录下:
- 依赖信息文件
  - 一个编译成果的旁边，都有一个依赖信息文件，文件后缀是 .d。该文件的语法类似于 Makefile，用于说明构建编译成果所需的所有依赖包。
  - 该文件往往用于提供给外部的构建系统，这样它们就可以判断 Cargo 命令是否需要再次被执行。
  - 文件中的路径默认是绝对路径，你可以通过 build.dep-info-basedir 配置项来修改为相对路径。




