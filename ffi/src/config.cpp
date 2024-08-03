#include "config.hpp"

using namespace libfreenect2_ffi;

LIBFREENECT2_MAYBE_UNUSED float Config::get_min_depth() const noexcept {
  return config.MinDepth;
}

LIBFREENECT2_MAYBE_UNUSED float Config::get_max_depth() const noexcept {
  return config.MaxDepth;
}

LIBFREENECT2_MAYBE_UNUSED bool Config::get_enable_bilateral_filter()
    const noexcept {
  return config.EnableBilateralFilter;
}

LIBFREENECT2_MAYBE_UNUSED bool Config::get_enable_edge_aware_filter()
    const noexcept {
  return config.EnableEdgeAwareFilter;
}

LIBFREENECT2_MAYBE_UNUSED void Config::set_min_depth(float min_depth) noexcept {
  config.MinDepth = min_depth;
}

LIBFREENECT2_MAYBE_UNUSED void Config::set_max_depth(float max_depth) noexcept {
  config.MaxDepth = max_depth;
}

LIBFREENECT2_MAYBE_UNUSED void Config::set_enable_bilateral_filter(
    bool enable_bilateral_filter) noexcept {
  config.EnableBilateralFilter = enable_bilateral_filter;
}

LIBFREENECT2_MAYBE_UNUSED void Config::set_enable_edge_aware_filter(
    bool enable_edge_aware_filter) noexcept {
  config.EnableEdgeAwareFilter = enable_edge_aware_filter;
}

LIBFREENECT2_MAYBE_UNUSED std::unique_ptr<Config>
libfreenect2_ffi::create_config() {
  return std::make_unique<Config>();
}
