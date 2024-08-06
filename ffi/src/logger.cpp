#include "logger.hpp"

class Logger : public libfreenect2::Logger {
 public:
  explicit Logger(rust::Fn<void(LogLevel, const rust::String &)> log_fn)
      : log_fn(log_fn) {}

  Level level() const override {
    return Level::Debug;
  }

  void log(Level level, const std::string &message) override {
    log_fn(static_cast<LogLevel>(level), message);
  }

 private:
  rust::Fn<void(LogLevel, const rust::String &)> log_fn;
};

LIBFREENECT2_MAYBE_UNUSED void libfreenect2_ffi::create_logger(
    rust::Fn<void(LogLevel, const rust::String &)> log_fn) {
  auto logger = new Logger(log_fn);
  libfreenect2::setGlobalLogger(logger);
}