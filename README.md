# PIO-first Rust ESP32-CAM project

* Build and flash with `cargo pio exec -- run -t upload`
* **NOTE**: Need to expose the ESP32-CAM driver C functions manually, with Rust unsafe `extern "C"` declarations
