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
    std::string name;
    std::string created_at;

    int width = 0;
    int height = 0;
    int depth = 0;

    int frame_buffer_port = -1;
    int command_port = -1;

    std::unique_ptr<zmq::socket_t> socket;
};

void connect(std::string infoDir = "/tmp/cover_screen");
const std::vector<ScreenInfo_t>& getScreens();
bool pushFrame(const std::string& screenName, uint8_t* data, size_t size);
void stop();

} // namespace cover_screen
