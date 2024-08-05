#include "freenect2_device.hpp"

using namespace libfreenect2_ffi;

Freenect2Device::Freenect2Device(libfreenect2::Freenect2Device* device)
    : device(device) {
  if (device == nullptr) {
    throw std::runtime_error("Failed to open device");
  }
}

LIBFREENECT2_MAYBE_UNUSED rust::String Freenect2Device::get_serial_number() {
  return device->getSerialNumber();
}

LIBFREENECT2_MAYBE_UNUSED rust::String Freenect2Device::get_firmware_version() {
  return device->getFirmwareVersion();
}

LIBFREENECT2_MAYBE_UNUSED bool Freenect2Device::start() {
  return device->start();
}

LIBFREENECT2_MAYBE_UNUSED bool Freenect2Device::start_streams(bool rgb,
                                                              bool depth) {
  return device->startStreams(rgb, depth);
}

LIBFREENECT2_MAYBE_UNUSED bool Freenect2Device::stop() {
  return device->stop();
}

LIBFREENECT2_MAYBE_UNUSED bool Freenect2Device::close() {
  return device->close();
}

LIBFREENECT2_MAYBE_UNUSED void Freenect2Device::set_color_frame_listener(
    const std::unique_ptr<libfreenect2::FrameListener>& listener) {
  device->setColorFrameListener(listener.get());
}

LIBFREENECT2_MAYBE_UNUSED void Freenect2Device::set_ir_and_depth_frame_listener(
    const std::unique_ptr<libfreenect2::FrameListener>& listener) {
  device->setIrAndDepthFrameListener(listener.get());
}

LIBFREENECT2_MAYBE_UNUSED void Freenect2Device::set_config(
    const std::unique_ptr<Config>& config) {
  device->setConfiguration(config->config);
}

LIBFREENECT2_MAYBE_UNUSED std::unique_ptr<Registration>
Freenect2Device::get_registration() {
  return std::make_unique<Registration>(device);
}

Freenect2Device::~Freenect2Device() {
  delete device;
}
