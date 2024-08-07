#include "libfreenect2.hpp"

#include "libfreenect2-rs/src/ffi.rs.h"
#include "libfreenect2/packet_pipeline.h"

using namespace libfreenect2_ffi;

Freenect2::Freenect2() : freenect2() {}

Freenect2::~Freenect2() = default;

libfreenect2::PacketPipeline *get_pipeline(PacketPipeline pipeline) {
  switch (pipeline) {
#if defined(LIBFREENECT2_RS_WITH_OPENCL) && !defined(__linux__)
    case PacketPipeline::OpenCL:
      return new libfreenect2::OpenCLPacketPipeline();
    case PacketPipeline::OpenCLKDE:
      return new libfreenect2::OpenCLKdePacketPipeline();
#endif  // LIBFREENECT2_RS_WITH_OPENCL
#ifdef LIBFREENECT2_RS_WITH_OPENGL
    case PacketPipeline::OpenGL:
      return new libfreenect2::OpenGLPacketPipeline();
#endif  // LIBFREENECT2_RS_WITH_OPENGL
    default:
      return new libfreenect2::CpuPacketPipeline();
  }
}

LIBFREENECT2_MAYBE_UNUSED int32_t Freenect2::enumerate_devices() {
  return freenect2.enumerateDevices();
}

LIBFREENECT2_MAYBE_UNUSED rust::String Freenect2::get_device_serial_number(
    int32_t idx) {
  return freenect2.getDeviceSerialNumber(idx);
}

LIBFREENECT2_MAYBE_UNUSED rust::String
Freenect2::get_default_device_serial_number() {
  freenect2.openDefaultDevice();
  return freenect2.getDefaultDeviceSerialNumber();
}

LIBFREENECT2_MAYBE_UNUSED std::unique_ptr<Freenect2Device>
Freenect2::open_device_by_id(int32_t idx) {
  return std::make_unique<Freenect2Device>(freenect2.openDevice(idx));
}

LIBFREENECT2_RS_FUNC std::unique_ptr<Freenect2Device>
Freenect2::open_device_by_id_with_packet_pipeline(int32_t idx,
                                                  PacketPipeline pipeline) {
  return std::make_unique<Freenect2Device>(
      freenect2.openDevice(idx, get_pipeline(pipeline)));
}

LIBFREENECT2_MAYBE_UNUSED std::unique_ptr<Freenect2Device>
Freenect2::open_device_by_serial(rust::Str serial) {
  return std::make_unique<Freenect2Device>(
      freenect2.openDevice(serial.operator std::string()));
}

LIBFREENECT2_RS_FUNC std::unique_ptr<Freenect2Device>
Freenect2::open_device_by_serial_with_packet_pipeline(rust::Str serial,
                                                      PacketPipeline pipeline) {
  return std::make_unique<Freenect2Device>(freenect2.openDevice(
      serial.operator std::string(), get_pipeline(pipeline)));
}

LIBFREENECT2_MAYBE_UNUSED std::unique_ptr<Freenect2Device>
Freenect2::open_default_device() {
  return std::make_unique<Freenect2Device>(freenect2.openDefaultDevice());
}

LIBFREENECT2_RS_FUNC std::unique_ptr<Freenect2Device>
Freenect2::open_default_device_with_packet_pipeline(PacketPipeline pipeline) {
  return std::make_unique<Freenect2Device>(
      freenect2.openDefaultDevice(get_pipeline(pipeline)));
}

LIBFREENECT2_MAYBE_UNUSED std::unique_ptr<Freenect2>
libfreenect2_ffi::create_freenect2() {
  return std::make_unique<Freenect2>();
}
