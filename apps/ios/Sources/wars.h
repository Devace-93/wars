// C ABI of the Rust engine (crates/ffi/src/lib.rs).
#ifndef WARS_H
#define WARS_H
#include <stdbool.h>
#include <stdint.h>

void wars_create(uint32_t width, uint32_t height, float scale, const char *data_dir);
bool wars_attach_metal_layer(void *layer, uint32_t width, uint32_t height);
void wars_detach_surface(void);
void wars_resize(uint32_t width, uint32_t height);
bool wars_frame(void);
void wars_touch(int32_t kind, float x, float y);
void wars_long_press(float x, float y);
void wars_double_tap(float x, float y);
void wars_pinch(float scale, float x, float y);
void wars_pan(float dx, float dy);
void wars_key(int32_t code, bool down);
void wars_text(const char *s);
bool wars_wants_keyboard(void);
bool wars_back(void);

#endif
