cargo fmt
cargo build

Write-Output "================== Compiling stdlib ======================"

./target/debug/xic.exe std/lib.xi -vv -O0 -o target/std/

Write-Output "================== Compiling examples ======================"

./target/debug/xic.exe ./examples/main.xi -vv -O0 -o ./examples-build

Write-Output "==================    Run examples    ======================="

<# 
If stdlib is loaded, ./target/std/std.dll must present for native functions
Build project ./std-dll for std.dll
#>

./target/debug/xix.exe ./examples-build/examples.xibc -d
