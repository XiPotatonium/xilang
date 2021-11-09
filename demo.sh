cargo fmt
cargo build

echo "================== Compiling examples ======================"

./target/debug/xic ./examples/main.xi -vv -O0 -o ./examples-build

