#ifndef XILANG_LANG_CFG_HPP
#define XILANG_LANG_CFG_HPP

#include <string>

namespace lang {

struct Config {
    bool no_sys;
    bool compile;
    bool dump_ast;
    std::string entry;
};

}

#endif