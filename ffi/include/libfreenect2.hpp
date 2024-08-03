#ifndef FFI_LIBFREENECT2_HPP
#define FFI_LIBFREENECT2_HPP

#include <libfreenect2/libfreenect2.hpp>
#include <memory>
#include "freenect2_device.hpp"
#include "rust/cxx.h"
#include "macros.hpp"

namespace libfreenect2_ffi {
    class Freenect2 {
    public:
        Freenect2();

        ~Freenect2();

        LIBFREENECT2_RS_FUNC int32_t enumerate_devices();

        LIBFREENECT2_RS_FUNC rust::String get_device_serial_number(int32_t idx);

        LIBFREENECT2_RS_FUNC rust::String get_default_device_serial_number();

        LIBFREENECT2_RS_FUNC std::unique_ptr<Freenect2Device> open_device_by_id(int32_t idx);

        LIBFREENECT2_RS_FUNC std::unique_ptr<Freenect2Device> open_device_by_serial(rust::Str serial);

        LIBFREENECT2_RS_FUNC std::unique_ptr<Freenect2Device> open_default_device();

    private:
        libfreenect2::Freenect2 freenect2;
    };

    LIBFREENECT2_RS_FUNC std::unique_ptr<Freenect2> create_freenect2();
}

#endif //FFI_LIBFREENECT2_HPP
