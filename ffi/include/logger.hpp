#ifndef FFI_LOGGER_HPP
#define FFI_LOGGER_HPP

#include <libfreenect2/logger.h>

#include "macros.hpp"
#include "rust/cxx.h"

enum class LogLevel : ::std::uint8_t;

namespace libfreenect2_ffi {
  LIBFREENECT2_MAYBE_UNUSED void create_logger(
      rust::Fn<void(LogLevel, const rust::String &)> log_fn);
}  // namespace libfreenect2_ffi

#endif  // FFI_LOGGER_HPP