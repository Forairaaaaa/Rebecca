/**
 * @file lvgl_port.cpp
 * @author Forairaaaaa
 * @brief
 * @version 0.1
 * @date 2025-08-02
 *
 * @copyright Copyright (c) 2025
 *
 */
#include "lvgl_port.h"
#include "cover_screen.h"
#include <mooncake_log.h>
#include <chrono>
#include <lvgl.h>

namespace lvgl_port {

static constexpr int _color_depth = LV_COLOR_DEPTH / 8;

static std::string _screen_name;

static const auto _app_start_time = std::chrono::steady_clock::now();

static uint8_t* _convert_buf = nullptr;

static uint32_t my_get_ticks()
{
    auto now = std::chrono::steady_clock::now();
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(now - _app_start_time).count();
    return static_cast<uint32_t>(ms);
}

// RGBA
// static void my_flush_cb(lv_display_t* display, const lv_area_t* area, uint8_t* px_map)
// {
//     int width = area->x2 - area->x1 + 1;
//     int height = area->y2 - area->y1 + 1;
//     size_t total_pixels = width * height;
//     size_t required_size = total_pixels * 4;

//     const uint8_t* src = px_map;
//     uint8_t* dst = _convert_buf;
//     uint8_t* end = dst + required_size;

//     while (dst < end) {
//         dst[0] = src[2]; // R ← B
//         dst[1] = src[1]; // G
//         dst[2] = src[0]; // B ← R
//         dst[3] = 255;    // A 固定值

//         src += 4;
//         dst += 4;
//     }

//     cover_screen::pushFrame(_screen_name, _convert_buf, required_size);
//     lv_display_flush_ready(display);
// }

// RGB565
static void my_flush_cb(lv_display_t* display, const lv_area_t* area, uint8_t* px_map)
{
    int width = area->x2 - area->x1 + 1;
    int height = area->y2 - area->y1 + 1;
    size_t total_pixels = width * height;

    cover_screen::pushFrame(_screen_name, px_map, total_pixels * _color_depth);
    lv_display_flush_ready(display);
}

bool init(std::string screenName)
{
    mclog::info("Initializing LVGL for screen: {}", screenName);

    if (!cover_screen::exists(screenName)) {
        mclog::error("Screen {} does not exist", screenName);
        return false;
    }

    _screen_name = screenName;

    auto& screen = cover_screen::getScreen(screenName);

    lv_init();
    lv_tick_set_cb(my_get_ticks);

    auto buf_size = screen.width * screen.height * _color_depth;
    auto buf1 = new uint8_t[buf_size];
    // auto buf2 = new uint8_t[buf_size];
    // _convert_buf = new uint8_t[buf_size];

    lv_display_t* display = lv_display_create(screen.width, screen.height);
    lv_display_set_buffers(display, buf1, NULL, buf_size, LV_DISPLAY_RENDER_MODE_FULL);
    lv_display_set_flush_cb(display, my_flush_cb);

    mclog::info("LVGL initialized");

    return true;
}

void update()
{
    lv_timer_handler();
}

} // namespace lvgl_port
