#ifndef FFI_FRAME_HPP
#define FFI_FRAME_HPP

#include <libfreenect2/frame_listener.hpp>
#include <memory>

#include "macros.hpp"
#include "rust/cxx.h"

enum class FrameType : ::std::uint8_t;
enum class FrameFormat : ::std::uint8_t;

struct CallContext;

namespace libfreenect2_ffi {
  class Frame {
   public:
    explicit Frame(libfreenect2::Frame* frame);

    LIBFREENECT2_RS_FUNC uint64_t width() const;
    LIBFREENECT2_RS_FUNC uint64_t height() const;
    LIBFREENECT2_RS_FUNC uint64_t bytes_per_pixel() const;
    LIBFREENECT2_RS_FUNC unsigned char* data() const;
    LIBFREENECT2_RS_FUNC uint32_t timestamp() const;
    LIBFREENECT2_RS_FUNC uint32_t sequence() const;
    LIBFREENECT2_RS_FUNC float exposure() const;
    LIBFREENECT2_RS_FUNC float gain() const;
    LIBFREENECT2_RS_FUNC float gamma() const;
    LIBFREENECT2_RS_FUNC uint32_t status() const;
    LIBFREENECT2_RS_FUNC FrameFormat format() const;

    ~Frame();

   private:
    libfreenect2::Frame* frame;
  };

  LIBFREENECT2_RS_FUNC std::unique_ptr<libfreenect2::FrameListener>
  create_frame_listener(
      rust::cxxbridge1::Box<CallContext> ctx,
      rust::Fn<void(FrameType, std::unique_ptr<Frame>,
                    const rust::cxxbridge1::Box<CallContext>&)>
          on_new_frame);

#ifndef NDEBUG
  namespace test {
    LIBFREENECT2_MAYBE_UNUSED void call_frame_listener(
        std::unique_ptr<libfreenect2::FrameListener>& listener, FrameType type,
        uint64_t width, uint64_t height, uint64_t bytes_per_pixel,
        unsigned char* data);
  }
#endif
}  // namespace libfreenect2_ffi

#endif  // FFI_FRAME_HPP
