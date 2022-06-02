# xilang

A toy programming language and its interpreter.

* 非基本类型不能为`null`，标准库中使用`opt<T>`来表示nullable，这会需要rust式的`enum`
* 函数不能重载
* 要有模板，不然强类型情况下容器没法做
