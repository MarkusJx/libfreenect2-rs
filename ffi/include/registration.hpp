#ifndef FFI_REGISTRATION_HPP
#define FFI_REGISTRATION_HPP

#include <libfreenect2/registration.h>

#include "frame.hpp"
#include "macros.hpp"

namespace libfreenect2_ffi {
  class Registration {
   public:
    explicit Registration(libfreenect2::Freenect2Device* device);

    LIBFREENECT2_MAYBE_UNUSED void map_depth_to_color(const Frame& depth,
                                                      const Frame& color,
                                                      Frame& undistorted_depth,
                                                      Frame& color_depth_image,
                                                      bool enable_filter) const;

    LIBFREENECT2_MAYBE_UNUSED void map_depth_to_full_color(
        const Frame& depth, const Frame& color, Frame& undistorted_depth,
        Frame& color_depth_image, bool enable_filter, Frame& big_depth) const;

    LIBFREENECT2_MAYBE_UNUSED void undistort_depth(
        const Frame& depth, Frame& undistorted_depth) const;

   private:
    libfreenect2::Registration registration;
  };
}  // namespace libfreenect2_ffi

#endif  // FFI_REGISTRATION_HPP
