# xilang

A toy OOP programming language that runs on a simple CLR-like virtual machine.

[Development Diary开发日志](https://xipotatonium.github.io/2021/04/04/XilangDev0/)

## 1 TODO

### 1.1 FIX

IrFile Loader的逻辑存在问题，无论是VM里的还是Lang里的，external module的递归读取还是要再考虑一下
另外在lang load的时候就要检查VM的版本

* Local var. 
  * 目前let声明变量如果有同名的变量会发生覆盖，但是实际上不应该产生覆盖，let仅仅是绑定，新的局部变量一定分配新的空间。
  * ldloc.n的n不是slot的下标，而是第几个局部变量。因为存在用户定义struct，需要考证栈到底是怎么实现的，有没有slot。
  * 局部变量的类型信息需要在blob记录
* 更贴近CLR标准的Blob
  * 除了上面说的局部变量信息，其他Blob设计(例如函数签名)也应当逐渐接近CLR标准，不过不需要采用它那个样子的编码

### 1.2 RoadMap

#### Ver 0.1.0

* OOP:
  * class
  * static/non-static method/field
  * cctor
  * ctor (default constructor)
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
* xilang project:
  * mod

#### Ver 0.2.0 (Present)

* Dllimport
* stdlib
  * io
* xilang project:
  * extern

#### Ver 0.3.0

* Class inheritance
* Param table
* Blob

#### Ver 0.3.1

* Array
* String
* Built-in type:
  * i8/u8
  * i16/u16
  * u32

#### Ver 0.3.2

* Refactor lang
  * make incremental compilation possible
  * Overload
* xilang project structure
  * crate

#### Ver 0.4.0

* Interface

#### Ver 0.5.0

* Generic
* stdlib
  * collections

#### Ver 0.6.0

* pub use
* priv/pub flag
* Attribute
* stdlib
  * Dllimport attribute

#### Ver 0.6.1

* GC
* Refactor vm, use unsafe properly

#### Ver 0.6.2

* Built-in type:
  * f64
* Constant folding

#### Ver 0.6.3

* match expr
* Default value and StructExprEtCetera

#### Ver 0.7.0

* struct
* stdlib
  * tuple

#### Ver 0.8.0

* enum

#### Ver 0.8.1

* union

#### Ver 0.8.2

* for loop
* Iterator interface

## 2 Usage

For examples see [demo.ps1](demo.ps1)

```
USAGE:
    xic.exe [FLAGS] [OPTIONS] <root>

FLAGS:
    -h, --help       Prints help information
        --no_std     Build without stdlib
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