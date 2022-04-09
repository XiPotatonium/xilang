/**
 * @file path.hpp
 * @author shwu (xipotatonium@lookout.com)
 * @brief Only ASCII path is supported
 * @version 0.1
 * @date 2022-04-04
 *
 * @copyright Copyright (c) 2022
 *
 */
#ifndef XILANG_CORE_PATH_HPP
#define XILANG_CORE_PATH_HPP

#include <cstddef>
#include <memory>
#include <optional>
#include <stdexcept>
#include <string_view>
#include <vector>

namespace core {

class Path;
class PathBuf;

struct PathSeg {
    std::string id;
    std::optional<std::vector<PathBuf>> generics;

    explicit PathSeg(const std::string_view &id) : id(id) {}
    PathSeg(const std::string_view &id, std::vector<PathBuf> &&generics)
        : id(id), generics(generics) {}
};

class IPath {
public:
    virtual ~IPath() = default;

    [[nodiscard]] virtual std::size_t len() const = 0;
    [[nodiscard]] bool empty() const { return this->len() == 0; }
    [[nodiscard]] virtual const PathSeg &self() const = 0;
    [[nodiscard]] virtual std::string string() const = 0;
    [[nodiscard]] virtual Path range(std::size_t start, std::size_t end) const = 0;
    [[nodiscard]] virtual Path range(std::size_t end) const;
    [[nodiscard]] virtual Path range() const;

    using const_iterator = typename std::vector<PathSeg>::const_iterator;

    [[nodiscard]] virtual const_iterator begin() const = 0;
    [[nodiscard]] virtual const_iterator end() const = 0;
};

class Path : public IPath {
    const PathBuf &buf_;
    std::size_t begin_;
    std::size_t end_;

public:
    Path(const PathBuf &buf, std::size_t begin, std::size_t end) : buf_(buf), begin_(begin), end_(end) {}

    [[nodiscard]] std::size_t len() const override { return end_ - begin_; }
    [[nodiscard]] const PathSeg &self() const override;
    [[nodiscard]] std::string string() const override;
    [[nodiscard]] Path range(std::size_t begin, std::size_t end) const override;

    [[nodiscard]] const_iterator begin() const override;
    [[nodiscard]] const_iterator end() const override;
};

class PathBuf : public IPath {

    std::vector<PathSeg> segs_;

public:
    [[nodiscard]] const PathSeg &at(std::size_t n) const { return segs_.at(n); }

    [[nodiscard]] std::size_t len() const override { return segs_.size(); }
    [[nodiscard]] const PathSeg &self() const override { return segs_.back(); }
    [[nodiscard]] std::string string() const override;
    [[nodiscard]] Path range(std::size_t begin, std::size_t end) const override;

    [[nodiscard]] const_iterator begin() const override { return segs_.cbegin(); }
    [[nodiscard]] const_iterator end() const override { return segs_.cend(); }

    void push(PathSeg &&seg) { this->segs_.push_back(seg); }
};

}// namespace core

#endif
