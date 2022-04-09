#ifndef XILANG_CORE_COMMON_HPP
#define XILANG_CORE_COMMON_HPP

#include <exception>
#include <stdexcept>

namespace core {

class NotImplementedException : public std::logic_error {
public:
    NotImplementedException() : std::logic_error {"Function not yet implemented."} {}
};

}// namespace core

#endif