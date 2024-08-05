#ifndef FFI_FREENECT2_DEVICE_HPP
#define FFI_FREENECT2_DEVICE_HPP

#include <libfreenect2/libfreenect2.hpp>
#include <memory>

#include "config.hpp"
#include "macros.hpp"
#include "rust/cxx.h"
#include "registration.hpp"

namespace libfreenect2_ffi {
  class Freenect2Device {
   public:
    explicit Freenect2Device(libfreenect2::Freenect2Device* device);

    ~Freenect2Device();

    LIBFREENECT2_RS_FUNC rust::String get_serial_number();

    LIBFREENECT2_RS_FUNC rust::String get_firmware_version();

    LIBFREENECT2_RS_FUNC bool start();

    LIBFREENECT2_RS_FUNC bool start_streams(bool rgb, bool depth);

    LIBFREENECT2_RS_FUNC bool stop();

    LIBFREENECT2_RS_FUNC bool close();

    LIBFREENECT2_MAYBE_UNUSED void set_color_frame_listener(
        const std::unique_ptr<libfreenect2::FrameListener>& listener);

    LIBFREENECT2_MAYBE_UNUSED void set_ir_and_depth_frame_listener(
        const std::unique_ptr<libfreenect2::FrameListener>& listener);

    LIBFREENECT2_MAYBE_UNUSED void set_config(
        const std::unique_ptr<Config>& config);

    LIBFREENECT2_MAYBE_UNUSED std::unique_ptr<Registration> get_registration();

   private:
    libfreenect2::Freenect2Device* device;
  };
}  // namespace libfreenect2_ffi

#endif  // FFI_FREENECT2_DEVICE_HPP
