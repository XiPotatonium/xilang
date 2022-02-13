# xilang

A toy script language.

[Development Diary开发日志](https://xipotatonium.github.io/2021/04/04/XilangDev0/)

## 1 FIX

## 2 TODO

* Class type:
  * static/non-static method/field
  * priv/pub flag
* Built-in type:
  * string
  * array
  * bool
  * u8
  * char
  * i32
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
* stdlib
  * collections
  * Iterator interface

## 3 Usage

See [demo.sh](demo.sh) for demo.

```
xi --help
```

## 4 Grammar

See [PEGs file](src/lang/parser/grammar.pest)
