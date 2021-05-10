# xilang

A toy OOP programming language that runs on a simple CLR-like virtual machine.

[Development Diary开发日志](https://xipotatonium.github.io/2021/04/04/XilangDev0/)


* OOP:
  * class
  * static/non-static method/field
  * cctor
  * ctor
  * Virtual method
  * overload
  * inheritance:
    * all classes except std::Object are derived from std::Object
    * accessing instance fields and methods of base classes
    * ctor of derived class call default ctor of base class automatically
* Built-in type:
  * i32
  * bool
  * single-dim zero-based array: only reference type array
* expr/stmt:
  * return
  * if
  * loop
    * loop loop
    * break
    * continue
  * numerical expr: `+ - * / %`
  * cmp: `> < == != >= <=`
  * logical: `&& || !`
* Built-in attribute:
  * dllimport
* xilang project:
  * mod keyword
  * extern keyword

* stdlib
  * io

## 1 TODO

### 1.1 FIX

* module存在环形依赖的时候估计是会出现错误的，加载的时候需要检查是否已经加载了
* 出现泛型之后，类型也是允许环形依赖的，目前的type loader算法有误
* 研究一下Native部分使用LLVM工具来实现JIT的可能性，CLR-like的Native交互相比JNI-like的Native交互更加舒服
* IrFile在读写时候需要检查一些内容（如attribute）是否是compliant的

### 1.2 RoadMap

#### Ver 0.4.0

* String
* Attribute
  * Dllimport attribute
  * InternalCall
* builtin type
  * usize
  * char

#### Ver 0.5.0

* Generic
* Refactor lang
  * DRY
  * make incremental compilation possible
  * lazy loading external class
* Refactor vm
  * lazy type loading

#### Ver 0.5.1

* More builtin type
  * bool
  * u8
  * f32/f64
  * isize
* pub use
* priv/pub flag

#### Ver 0.5.2

* struct
* Array for struct type
* stdlib
  * collections

#### Ver 0.6.0

* Interface
* for loop
* stdlib
  * Iterator interface

#### Ver 0.6.1

* enum
* union

#### Ver 0.6.2

* match expr

#### Ver 0.1.0

* GC

## 2 Usage

For examples see [demo.ps1](demo.ps1)

```
USAGE:
    xic.exe [FLAGS] [OPTIONS] <root>

FLAGS:
    -h, --help       Prints help information
    -v, --verbose    Level of verbosity. Level1: Display project tree; Level2: Dump .ast.json
    -V, --version    Prints version information

OPTIONS:
    -i, --import <ext>       External module paths
    -O <optim>               Optimization level: 0 | 1
    -o, --output <output>    Output directory. Default to be <root> if not specified

ARGS:
    <root>    Root path
```


```
USAGE:
    xix.exe [FLAGS] [OPTIONS] <entry>

FLAGS:
    -d, --diagnose    Show diagnose info or not
    -h, --help        Prints help information
    -V, --version     Prints version information

OPTIONS:
    -i, --import <ext>    External module paths

ARGS:
    <entry>    Entry module of executable
```

## 3 Grammar

See [PEGs file](src/lang/parser/grammar.pest)