#ifndef FFI_CONFIG_HPP
#define FFI_CONFIG_HPP

#include <libfreenect2/libfreenect2.hpp>
#include <memory>

#include "macros.hpp"

namespace libfreenect2_ffi {
  class Config {
   public:
    LIBFREENECT2_RS_FUNC float get_min_depth() const noexcept;

    LIBFREENECT2_RS_FUNC float get_max_depth() const noexcept;

    LIBFREENECT2_RS_FUNC bool get_enable_bilateral_filter() const noexcept;

    LIBFREENECT2_RS_FUNC bool get_enable_edge_aware_filter() const noexcept;

    LIBFREENECT2_MAYBE_UNUSED void set_min_depth(float min_depth) noexcept;

    LIBFREENECT2_MAYBE_UNUSED void set_max_depth(float max_depth) noexcept;

    LIBFREENECT2_MAYBE_UNUSED void set_enable_bilateral_filter(
        bool enable_bilateral_filter) noexcept;

    LIBFREENECT2_MAYBE_UNUSED void set_enable_edge_aware_filter(
        bool enable_edge_aware_filter) noexcept;

    libfreenect2::Freenect2Device::Config config;
  };

  LIBFREENECT2_RS_FUNC std::unique_ptr<Config> create_config();
}  // namespace libfreenect2_ffi

#endif  // FFI_CONFIG_HPP
