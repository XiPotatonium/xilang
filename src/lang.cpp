#include "CLI/App.hpp"
#include "CLI/Config.hpp"
#include "CLI/Formatter.hpp"
#include <exception>
#include <fmt/core.h>
#include <functional>
#include <iostream>

// This file will be generated automatically when you run the CMake configuration step.
// It creates a namespace called `myproject`.
// You can modify the source template at `configured_files/config.hpp.in`.
#include <internal_use_only/config.hpp>

#include <lang/cfg.hpp>

int main(int argc, const char **argv) noexcept {
    lang::Config cfg {};
    try {
        CLI::App app {fmt::format("xilang compiler and its interpreter. Ver {}", myproject::cmake::project_version),
            fmt::format("{}", myproject::cmake::project_name)};

        app.add_option("entry", cfg.entry, "Entry file");
        app.add_flag("-c,--compile", cfg.compile, "Do not run. Only generate byte code in cache.");
        app.add_flag("--ast", cfg.dump_ast, "Whether to dump .ast.json in cache");
        app.add_flag("--no-sys", cfg.no_sys, "Not to load syslib");

        try {
            app.parse(argc, argv);
        } catch (const CLI::ParseError &e) { return app.exit(e); }
    } catch (...) {
        std::unexpected();
        return -1;
    }

    fmt::print(
        "Entry = {}, compile = {}, dump-ast = {}, no-sys = {}\n", cfg.entry, cfg.compile, cfg.dump_ast, cfg.no_sys);

    return 0;
}
