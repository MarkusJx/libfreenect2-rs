#include "frame.hpp"

using namespace libfreenect2_ffi;

Frame::Frame(libfreenect2::Frame *frame) : frame(frame) {}

LIBFREENECT2_MAYBE_UNUSED uint64_t Frame::width() const {
  return frame->width;
}

LIBFREENECT2_MAYBE_UNUSED uint64_t Frame::height() const {
  return frame->height;
}

LIBFREENECT2_MAYBE_UNUSED uint64_t Frame::bytes_per_pixel() const {
  return frame->bytes_per_pixel;
}

LIBFREENECT2_MAYBE_UNUSED unsigned char *Frame::data() const {
  return frame->data;
}

LIBFREENECT2_MAYBE_UNUSED uint32_t Frame::timestamp() const {
  return frame->timestamp;
}

LIBFREENECT2_MAYBE_UNUSED uint32_t Frame::sequence() const {
  return frame->sequence;
}

LIBFREENECT2_MAYBE_UNUSED float Frame::exposure() const {
  return frame->exposure;
}

LIBFREENECT2_MAYBE_UNUSED float Frame::gain() const {
  return frame->gain;
}

LIBFREENECT2_MAYBE_UNUSED float Frame::gamma() const {
  return frame->gamma;
}

LIBFREENECT2_MAYBE_UNUSED uint32_t Frame::status() const {
  return frame->status;
}

LIBFREENECT2_MAYBE_UNUSED FrameFormat Frame::format() const {
  return static_cast<FrameFormat>(frame->format);
}

Frame::~Frame() {
  delete frame;
}

class FrameListenerImpl : public libfreenect2::FrameListener {
 public:
  explicit FrameListenerImpl(
      rust::cxxbridge1::Box<CallContext> &&ctx,
      rust::Fn<rust::String(FrameType, std::unique_ptr<Frame>,
                            const rust::cxxbridge1::Box<CallContext> &)>
          on_new_frame)
      : on_new_frame(on_new_frame), ctx(std::move(ctx)) {}

  bool onNewFrame(libfreenect2::Frame::Type type,
                  libfreenect2::Frame *frame) override {
    rust::String res = on_new_frame(static_cast<FrameType>(type),
                                    std::make_unique<Frame>(frame), ctx);

    if (!res.empty()) {
      throw std::runtime_error(res.operator std::string());
    }

    return true;
  }

 private:
  rust::Fn<rust::String(FrameType, std::unique_ptr<Frame>,
                        const rust::cxxbridge1::Box<CallContext> &)>
      on_new_frame;
  const rust::cxxbridge1::Box<CallContext> ctx;
};

LIBFREENECT2_RS_FUNC std::unique_ptr<Frame> libfreenect2_ffi::create_frame(
    uint64_t width, uint64_t height, uint64_t bytes_per_pixel,
    unsigned char *data, uint32_t timestamp, uint32_t sequence, float exposure,
    float gain, float gamma, uint32_t status, FrameFormat format) {
  auto *frame = new libfreenect2::Frame(width, height, bytes_per_pixel, data);

  frame->timestamp = timestamp;
  frame->sequence = sequence;
  frame->exposure = exposure;
  frame->gain = gain;
  frame->gamma = gamma;
  frame->status = status;
  frame->format = static_cast<libfreenect2::Frame::Format>(format);

  return std::make_unique<Frame>(frame);
}

LIBFREENECT2_MAYBE_UNUSED std::unique_ptr<libfreenect2::FrameListener>
libfreenect2_ffi::create_frame_listener(
    rust::cxxbridge1::Box<CallContext> ctx,
    rust::Fn<rust::String(FrameType, std::unique_ptr<Frame>,
                          const rust::cxxbridge1::Box<CallContext> &)>
        on_new_frame) {
  return std::make_unique<FrameListenerImpl>(std::move(ctx), on_new_frame);
}

#ifndef NDEBUG
namespace libfreenect2_ffi {
  namespace test {
    LIBFREENECT2_MAYBE_UNUSED void call_frame_listener(
        std::unique_ptr<libfreenect2::FrameListener> &listener, FrameType type,
        uint64_t width, uint64_t height, uint64_t bytes_per_pixel,
        unsigned char *data) {
      listener->onNewFrame(
          static_cast<libfreenect2::Frame::Type>(type),
          new libfreenect2::Frame(width, height, bytes_per_pixel, data));
    }
  }  // namespace test
}  // namespace libfreenect2_ffi
#endif
