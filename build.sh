cmake -DCMAKE_C_COMPILER=clang \
    -DCMAKE_CXX_COMPILER=clang++ \
    -DCMAKE_BUILD_TYPE=Debug \
    -DCMAKE_INSTALL_PREFIX=/home/xi/xilang/out/install/linux-clang-debug \
    -S/home/xi/xilang \
    -B/home/xi/xilang/out/build/linux-clang-debug \
    -G Ninja

cmake --build out/build/linux-clang-debug --target xilang

./out/build/linux-clang-debug/src/xilang examples/hello.xi