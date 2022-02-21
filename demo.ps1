cargo fmt
cargo build

echo "================== Compiling examples ======================"

./target/debug/xi.exe ./examples/main.xi --ast