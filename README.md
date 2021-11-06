# xilang

A toy rust-like programming language that runs on a JVM.

[Development Diary开发日志](https://xipotatonium.github.io/2021/04/04/XilangDev0/)

## 1 FIX

## 2 TODO

* Class type:
  * static/non-static method/field
  * priv/pub flag
* Built-in type:
  * i32
  * string
  * array
  * bool
  * u8
  * char
  * f32/f64
  * isize/usize
* expr/stmt:
  * return
  * if
  * loop
    * loop loop
    * for loop
    * break
    * continue
  * numerical expr: `+ - * / %`
  * cmp: `> < == != >= <=`
  * logical: `&& || !`
  * cast
  * match expr
* Interface
* Generic
* enum
* stdlib
  * collections
  * Iterator interface
* MyJVM

## 3 Usage

See [demo.sh](demo.sh) for demo.

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

## 4 Grammar

See [PEGs file](src/lang/parser/grammar.pest)
