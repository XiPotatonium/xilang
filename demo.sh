# 1. make native code

mkdir -p ./native/out
cd ./native/out
# rm -rf *
cmake ..
make all
cp libxtd.so ../../target/std/libxtd.so
cd ../..

# 2. Build xilang

cargo fmt
cargo build

# 3. Run

echo "================== Compiling stdlib ======================"

# Set Dllimport attribute in xtd source code with value "libxtd.so" or xix cannot find libxtd.so

./target/debug/xic std/lib.xi -vv -O0 -o target/std/

echo "================== Compiling examples ======================"

./target/debug/xic ./examples/main.xi -vv -O0 -o ./examples-build

echo "==================    Run examples    ======================="

./target/debug/xix ./examples-build/examples.xibc -d
