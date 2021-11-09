cargo fmt
cargo build

Write-Output "================== Compiling examples ======================"

./target/debug/xic.exe ./examples/main.xi -vv -O0 -o ./examples-build
