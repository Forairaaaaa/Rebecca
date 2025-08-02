/**
 * @file cover_screen.cpp
 * @author Forairaaaaa
 * @brief
 * @version 0.1
 * @date 2025-08-02
 *
 * @copyright Copyright (c) 2025
 *
 */
#include "cover_screen.h"
#include <nlohmann/json.hpp>
#include <mooncake_log.h>
#include <filesystem>
#include <zmq.hpp>
#include <fstream>
#include <iostream>
#include <vector>
#include <memory>
#include <mutex>

namespace fs = std::filesystem;
using json = nlohmann::json;

namespace cover_screen {

static zmq::context_t _context(1);
static std::vector<ScreenInfo_t> _screens;
static std::recursive_mutex _screen_mutex;

void connect(std::string infoDir)
{
    mclog::info("Connecting to cover screen at {}", infoDir);

    std::lock_guard<std::recursive_mutex> lock(_screen_mutex);
    stop();

    for (const auto& entry : fs::directory_iterator(infoDir)) {
        if (entry.path().extension() == ".json") {
            try {
                std::ifstream file(entry.path());
                json j;
                file >> j;

                ScreenInfo_t screen;
                screen.name = j.value("name", "");
                screen.created_at = j.value("created_at", "");
                screen.width = j.value("width", 0);
                screen.height = j.value("height", 0);
                screen.depth = j.value("depth", 0);
                screen.frame_buffer_port = j.value("frame_buffer_port", -1);
                screen.command_port = j.value("command_port", -1);

                std::string addr = "tcp://127.0.0.1:" + std::to_string(screen.frame_buffer_port);
                screen.socket = std::make_unique<zmq::socket_t>(_context, ZMQ_REQ);
                screen.socket->connect(addr);

                _screens.push_back(std::move(screen));
                mclog::info("Connected to {} for screen: {}", addr, screen.name);

                // Pretty print the screen info
                mclog::info("\n{}", j.dump(4));

            } catch (const std::exception& e) {
                mclog::error("Failed to load {}: {}", entry.path().filename().string(), e.what());
            }
        }
    }
}

const std::vector<ScreenInfo_t>& getScreens()
{
    std::lock_guard<std::recursive_mutex> lock(_screen_mutex);
    return _screens;
}

bool pushFrame(const std::string& screenName, uint8_t* data, size_t size)
{
    std::lock_guard<std::recursive_mutex> lock(_screen_mutex);
    for (auto& screen : _screens) {
        if (screen.name == screenName && screen.socket) {
            try {
                // Zero-copy send
                zmq::message_t msg(data, size, nullptr, nullptr);
                screen.socket->send(msg, zmq::send_flags::none);

                zmq::message_t reply;
                auto ret = screen.socket->recv(reply, zmq::recv_flags::none);
                if (!ret) {
                    throw std::runtime_error("Failed to receive reply from " + screenName);
                }

                return true;
            } catch (const std::exception& e) {
                mclog::error("Failed to push frame: {}", e.what());
                return false;
            }
        }
    }
    return false;
}

void stop()
{
    std::lock_guard<std::recursive_mutex> lock(_screen_mutex);
    for (auto& screen : _screens) {
        if (screen.socket) {
            screen.socket->close();
        }
    }
    _screens.clear();
}

} // namespace cover_screen
