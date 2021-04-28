# xilang

A toy OOP programming language that runs on a simple CLR-like virtual machine.

[Development Diary开发日志](https://xipotatonium.github.io/2021/04/04/XilangDev0/)


* OOP:
  * class
  * static/non-static method/field
  * cctor
  * ctor (only one constructor)
  * inheritance:
    * all classes except std::Object is derived from std::Object
    * accessing instance field and method of base class
* Built-in type:
  * i32
  * bool
* expr/stmt:
  * return
  * if
  * loop
    * loop loop
    * break
    * continue
  * numerical expr: `+ - * /`
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

* Local var. 
  * ldloc.n的n不是slot的下标，而是第几个局部变量。因为存在用户定义struct，因此栈不能使用定长的slot
* bool占用1byte内存
* module存在环形依赖的时候估计是会出现错误的

### 1.2 RoadMap

#### Ver 0.3.1

* Array
* String
* OOP:
  * Virtual method
  * overload
  * ctor of derived class call default ctor of base class automatically

#### Ver 0.3.2

* Refactor lang
  * DRY
  * make incremental compilation possible
* xilang project structure
  * crate

#### Ver 0.4.0

* Generic
* stdlib
  * collections

#### Ver 0.4.1

* More builtin type
  * char
  * bool
  * i8/u8
  * i16/u16
  * u32
  * f32/f64

#### Ver 0.5.0

* Interface
* for loop
* stdlib
  * Iterator interface

#### Ver 0.5.1

* match expr

#### Ver 0.5.2

* enum

#### Ver 0.5.3

* union

#### Ver 0.6.0

* struct
* stdlib
  * tuple

#### Ver 0.7.0

* pub use
* priv/pub flag
* Attribute
* stdlib
  * Dllimport attribute

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