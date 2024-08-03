#include "libfreenect2.hpp"

using namespace libfreenect2_ffi;

Freenect2::Freenect2() : freenect2() {}

Freenect2::~Freenect2() = default;

LIBFREENECT2_MAYBE_UNUSED int32_t Freenect2::enumerate_devices() {
  return freenect2.enumerateDevices();
}

LIBFREENECT2_MAYBE_UNUSED rust::String Freenect2::get_device_serial_number(
    int32_t idx) {
  return freenect2.getDeviceSerialNumber(idx);
}

LIBFREENECT2_MAYBE_UNUSED rust::String
Freenect2::get_default_device_serial_number() {
  return freenect2.getDefaultDeviceSerialNumber();
}

LIBFREENECT2_MAYBE_UNUSED std::unique_ptr<Freenect2Device>
Freenect2::open_device_by_id(int32_t idx) {
  return std::make_unique<Freenect2Device>(freenect2.openDevice(idx));
}

LIBFREENECT2_MAYBE_UNUSED std::unique_ptr<Freenect2Device>
Freenect2::open_device_by_serial(rust::Str serial) {
  return std::make_unique<Freenect2Device>(
      freenect2.openDevice(serial.operator std::string()));
}

LIBFREENECT2_MAYBE_UNUSED std::unique_ptr<Freenect2Device>
Freenect2::open_default_device() {
  return std::make_unique<Freenect2Device>(freenect2.openDefaultDevice());
}

LIBFREENECT2_MAYBE_UNUSED std::unique_ptr<Freenect2>
libfreenect2_ffi::create_freenect2() {
  return std::make_unique<Freenect2>();
}
