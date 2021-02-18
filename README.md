# xilang

A toy programming language that runs on a simple virtual machine.

## 1 TODO

* more expressions
* xivm

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
    -o, --output <output>    Output directory. Default to be <root> if not specified

ARGS:
    <root>    Input root directory
```

## 3 Grammar

See [PEGs file](src/lang/parser/grammar.pest)