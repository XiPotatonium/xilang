cargo fmt
cargo build

# echo "================== Compiling stdlib ======================"

# ./target/debug/xic std/lib.xi -vv -O0 -o target/std/

echo "================== Compiling examples ======================"

./target/debug/xic ./examples/main.xi -vv -O0 -o ./examples-build

