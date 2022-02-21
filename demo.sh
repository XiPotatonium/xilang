cargo fmt
cargo build

echo "================== Compiling examples ======================"

./target/debug/xi ./examples/main.xi --ast
