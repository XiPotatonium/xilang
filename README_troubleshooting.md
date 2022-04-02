## Troubleshooting

### Update Conan
Many problems that users have can be resolved by updating Conan, so if you are
having any trouble with this project, you should start by doing that.

To update conan:

    pip install --user --upgrade conan

You may need to use `pip3` instead of `pip` in this command, depending on your
platform.

### Clear Conan cache
If you continue to have trouble with your Conan dependencies, you can try
clearing your Conan cache:

    conan remove -f '*'

The next time you run `cmake` or `cmake --build`, your Conan dependencies will
be rebuilt. If you aren't using your system's default compiler, don't forget to
set the CC, CXX, CMAKE_C_COMPILER, and CMAKE_CXX_COMPILER variables, as
described in the 'Build using an alternate compiler' section above.

### Identifying misconfiguration of Conan dependencies

If you have a dependency 'A' that requires a specific version of another
dependency 'B', and your project is trying to use the wrong version of
dependency 'B', Conan will produce warnings about this configuration error
when you run CMake. These warnings can easily get lost between a couple
hundred or thousand lines of output, depending on the size of your project.

If your project has a Conan configuration error, you can use `conan info` to
find it. `conan info` displays information about the dependency graph of your
project, with colorized output in some terminals.

    cd build
    conan info .

In my terminal, the first couple lines of `conan info`'s output show all of the
project's configuration warnings in a bright yellow font.

For example, the package `spdlog/1.5.0` depends on the package `fmt/6.1.2`.
If you were to modify the file `conanfile.py` so that it requires an
earlier version of `fmt`, such as `fmt/6.0.0`, and then run:

```bash
conan remove -f '*'       # clear Conan cache
rm -rf build              # clear previous CMake build
cmake -S . -B ./build     # rebuild Conan dependencies
conan info ./build
```

...the first line of output would be a warning that `spdlog` needs a more recent
version of `fmt`.

## Manage Clang version

[shell](https://gist.github.com/junkdog/70231d6953592cd6f27def59fe19e50d)

```bash
#!/usr/bin/env bash

#     --slave /usr/bin/$1 $1 /usr/bin/$1-\${version} \\

function register_clang_version {
    local version=$1
    local priority=$2

    update-alternatives \
        --install /usr/bin/llvm-config       llvm-config      /usr/bin/llvm-config-${version} ${priority} \
        --slave   /usr/bin/llvm-ar           llvm-ar          /usr/bin/llvm-ar-${version} \
        --slave   /usr/bin/llvm-as           llvm-as          /usr/bin/llvm-as-${version} \
        --slave   /usr/bin/llvm-bcanalyzer   llvm-bcanalyzer  /usr/bin/llvm-bcanalyzer-${version} \
        --slave   /usr/bin/llvm-cov          llvm-cov         /usr/bin/llvm-cov-${version} \
        --slave   /usr/bin/llvm-diff         llvm-diff        /usr/bin/llvm-diff-${version} \
        --slave   /usr/bin/llvm-dis          llvm-dis         /usr/bin/llvm-dis-${version} \
        --slave   /usr/bin/llvm-dwarfdump    llvm-dwarfdump   /usr/bin/llvm-dwarfdump-${version} \
        --slave   /usr/bin/llvm-extract      llvm-extract     /usr/bin/llvm-extract-${version} \
        --slave   /usr/bin/llvm-link         llvm-link        /usr/bin/llvm-link-${version} \
        --slave   /usr/bin/llvm-mc           llvm-mc          /usr/bin/llvm-mc-${version} \
        --slave   /usr/bin/llvm-mcmarkup     llvm-mcmarkup    /usr/bin/llvm-mcmarkup-${version} \
        --slave   /usr/bin/llvm-nm           llvm-nm          /usr/bin/llvm-nm-${version} \
        --slave   /usr/bin/llvm-objdump      llvm-objdump     /usr/bin/llvm-objdump-${version} \
        --slave   /usr/bin/llvm-ranlib       llvm-ranlib      /usr/bin/llvm-ranlib-${version} \
        --slave   /usr/bin/llvm-readobj      llvm-readobj     /usr/bin/llvm-readobj-${version} \
        --slave   /usr/bin/llvm-rtdyld       llvm-rtdyld      /usr/bin/llvm-rtdyld-${version} \
        --slave   /usr/bin/llvm-size         llvm-size        /usr/bin/llvm-size-${version} \
        --slave   /usr/bin/llvm-stress       llvm-stress      /usr/bin/llvm-stress-${version} \
        --slave   /usr/bin/llvm-symbolizer   llvm-symbolizer  /usr/bin/llvm-symbolizer-${version} \
        --slave   /usr/bin/llvm-tblgen       llvm-tblgen      /usr/bin/llvm-tblgen-${version}

    update-alternatives \
        --install /usr/bin/clang                 clang                 /usr/bin/clang-${version} ${priority} \
        --slave   /usr/bin/clang++               clang++               /usr/bin/clang++-${version}  \
        --slave   /usr/bin/asan_symbolize        asan_symbolize        /usr/bin/asan_symbolize-${version} \
        --slave   /usr/bin/c-index-test          c-index-test          /usr/bin/c-index-test-${version} \
        --slave   /usr/bin/clang-check           clang-check           /usr/bin/clang-check-${version} \
        --slave   /usr/bin/clang-cl              clang-cl              /usr/bin/clang-cl-${version} \
        --slave   /usr/bin/clang-cpp             clang-cpp             /usr/bin/clang-cpp-${version} \
        --slave   /usr/bin/clang-format          clang-format          /usr/bin/clang-format.exe \
        --slave   /usr/bin/clang-format-diff     clang-format-diff     /usr/bin/clang-format-diff-${version} \
        --slave   /usr/bin/clang-import-test     clang-import-test     /usr/bin/clang-import-test-${version} \
        --slave   /usr/bin/clang-include-fixer   clang-include-fixer   /usr/bin/clang-include-fixer-${version} \
        --slave   /usr/bin/clang-offload-bundler clang-offload-bundler /usr/bin/clang-offload-bundler-${version} \
        --slave   /usr/bin/clang-query           clang-query           /usr/bin/clang-query-${version} \
        --slave   /usr/bin/clang-rename          clang-rename          /usr/bin/clang-rename-${version} \
        --slave   /usr/bin/clang-reorder-fields  clang-reorder-fields  /usr/bin/clang-reorder-fields-${version} \
        --slave   /usr/bin/clang-tidy            clang-tidy            /usr/bin/clang-tidy.exe \
        --slave   /usr/bin/lldb                  lldb                  /usr/bin/lldb-${version} \
        --slave   /usr/bin/lldb-server           lldb-server           /usr/bin/lldb-server-${version}

    update-alternatives --install /usr/bin/clangd clangd /usr/bin/clangd-${version} ${priority}
}

register_clang_version $1 $2
```

