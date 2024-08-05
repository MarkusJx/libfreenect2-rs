#include "registration.hpp"

using namespace libfreenect2_ffi;

Registration::Registration(libfreenect2::Freenect2Device *device)
    : registration(device->getIrCameraParams(),
                   device->getColorCameraParams()) {}

LIBFREENECT2_MAYBE_UNUSED void Registration::map_depth_to_color(
    const libfreenect2_ffi::Frame &depth, const libfreenect2_ffi::Frame &color,
    Frame &undistorted_depth, Frame &color_depth_image,
    bool enable_filter) const {
  color_depth_image.frame->format = color.frame->format;
  registration.apply(color.frame, depth.frame, undistorted_depth.frame,
                     color_depth_image.frame, enable_filter);
}

LIBFREENECT2_MAYBE_UNUSED void Registration::map_depth_to_full_color(
    const libfreenect2_ffi::Frame &depth, const libfreenect2_ffi::Frame &color,
    libfreenect2_ffi::Frame &undistorted_depth,
    libfreenect2_ffi::Frame &color_depth_image, bool enable_filter,
    libfreenect2_ffi::Frame &big_depth) const {
  color_depth_image.frame->format = color.frame->format;

  registration.apply(color.frame, depth.frame, undistorted_depth.frame,
                     color_depth_image.frame, enable_filter, big_depth.frame);
}

LIBFREENECT2_MAYBE_UNUSED void Registration::undistort_depth(
    const libfreenect2_ffi::Frame &depth,
    libfreenect2_ffi::Frame &undistorted_depth) const {
  registration.undistortDepth(depth.frame, undistorted_depth.frame);
}
