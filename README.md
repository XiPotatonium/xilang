# xilang

A toy programming language that runs on a simple virtual machine.

## 1 TODO

* more expressions
    * arithmatic: -, *, /, %
    * unary: -
* more statements:
    * if
    * loop
* Constant folding
* GC

## 2 Usage

```
USAGE:
    xic.exe [FLAGS] [OPTIONS] <root>

FLAGS:
    -h, --help       Prints help information
    -v               Level of verbosity. Level1: Display project tree; Level2: Dump .ast.json
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
    xilang.exe [FLAGS] [OPTIONS] <entry>

FLAGS:
    -d, --diagnose    Run diagnose or not
    -h, --help        Prints help information
    -V, --version     Prints version information

OPTIONS:
    -i, --import <ext>    External module paths

ARGS:
    <entry>    Entry of executable
```

## 3 Grammar

See [PEGs file](src/lang/parser/grammar.pest)