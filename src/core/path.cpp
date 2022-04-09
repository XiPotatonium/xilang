#include "core/path.hpp"
#include <range/v3/view.hpp>
#include <range/v3/view/enumerate.hpp>
#include <sstream>
#include <utility>


namespace core {
[[nodiscard]] Path IPath::range(std::size_t end) const { return this->range(0, end); }
[[nodiscard]] Path IPath::range() const { return this->range(0, this->len()); }


[[nodiscard]] const PathSeg &Path::self() const { return this->buf_.at(this->end_ - 1); }
[[nodiscard]] std::string Path::string() const {
    std::stringstream sstream {};
    bool is_super_prefix = true;
    for (const auto &seg : *this) {
        if (seg.id == ".") {
            // WARNING: Hard coding super
            if (is_super_prefix) {
                // no "::"
                sstream << '.';
                continue;
            }
        }
        is_super_prefix = false;
        sstream << "::" << seg.id;
        if (!seg.generics->empty()) {
            sstream << '<';
            for (const auto &[g_i, g] : *seg.generics | ranges::views::enumerate) {
                if (g_i != 0) { sstream << ", "; }
                sstream << g.string();
            }
            sstream << '>';
        }
    }
    return sstream.str();
}

[[nodiscard]] Path Path::range(std::size_t begin, std::size_t end) const {
    if (begin >= end || this->begin_ + end > this->len()) { throw std::out_of_range("Invalid range for Path"); }
    return Path {this->buf_, this->begin_ + begin, this->begin_ + end};
}

[[nodiscard]] IPath::const_iterator Path::begin() const { return buf_.begin() + static_cast<long>(begin_); }
[[nodiscard]] IPath::const_iterator Path::end() const { return buf_.begin() + static_cast<long>(end_); }


[[nodiscard]] std::string PathBuf::string() const { return this->IPath::range().string(); }

[[nodiscard]] Path PathBuf::range(std::size_t begin, std::size_t end) const {
    if (begin >= end || end > this->len()) { throw std::out_of_range("Invalid range for Path"); }
    return Path {*this, begin, end};
}

}// namespace core