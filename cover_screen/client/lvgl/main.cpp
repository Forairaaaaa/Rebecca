/**
 * @file main.cpp
 * @author Forairaaaaa
 * @brief
 * @version 0.1
 * @date 2025-08-02
 *
 * @copyright Copyright (c) 2025
 *
 */
#include "utils/cover_screen.h"
#include "utils/lvgl_port.h"
#include <benchmark/lv_demo_benchmark.h>
#include <mooncake_log.h>
#include <lvgl.h>
#include <stress/lv_demo_stress.h>
#include <thread>
#include <signal.h>

bool stop_requested = false;

void handle_signal(int signal)
{
    if (signal == SIGINT) {
        mclog::info("Caught SIGINT (Ctrl+C), shutting down...");
        stop_requested = true;
    }
}

int main(int, char**)
{
    signal(SIGINT, handle_signal);

    cover_screen::connect();

    if (!lvgl_port::init("screen1")) {
        mclog::error("Failed to initialize LVGL");
        cover_screen::stop();
        return 1;
    }

    // App shit
    lv_demo_stress();
    // lv_demo_benchmark();

    while (!stop_requested) {
        lvgl_port::update();
        std::this_thread::sleep_for(std::chrono::milliseconds(5));
    }

    cover_screen::stop();

    return 0;
}
