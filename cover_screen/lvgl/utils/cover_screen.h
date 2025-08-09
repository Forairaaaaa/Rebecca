/**
 * @file cover_screen.h
 * @author Forairaaaaa
 * @brief
 * @version 0.1
 * @date 2025-08-02
 *
 * @copyright Copyright (c) 2025
 *
 */
#pragma once
#include <string>
#include <vector>
#include <memory>
#include <zmq.hpp>

namespace cover_screen {

struct ScreenInfo_t {
    std::string id;
    std::string description;
    std::string device_type;

    int width = 0;
    int height = 0;
    int bits_per_pixel = 0;

    int frame_buffer_port = -1;

    std::unique_ptr<zmq::socket_t> socket;
    bool pushFrame(uint8_t* data, size_t size);
};

void connect(const std::string& apiUrl = "http://127.0.0.1:12580");
const std::vector<ScreenInfo_t>& getScreens();
bool exists(const std::string& screenName);
const ScreenInfo_t& getScreen(const std::string& screenName);
bool pushFrame(const std::string& screenName, uint8_t* data, size_t size);
void stop();

} // namespace cover_screen
