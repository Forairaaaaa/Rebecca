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
#include <unordered_map>
#include <filesystem>
#include <zmq.hpp>
#include <fstream>
#include <vector>
#include <memory>

namespace fs = std::filesystem;
using json = nlohmann::json;

namespace cover_screen {

static zmq::context_t _context(1);
static std::vector<ScreenInfo_t> _screens;
static std::unordered_map<std::string, int> _screen_index_map;

bool ScreenInfo_t::pushFrame(uint8_t* data, size_t size)
{
    try {
        // Zero-copy send
        zmq::message_t msg(data, size, nullptr, nullptr);
        socket->send(msg, zmq::send_flags::none);

        zmq::message_t reply;
        auto ret = socket->recv(reply, zmq::recv_flags::none);
        if (!ret) {
            throw std::runtime_error("Failed to receive reply from " + name);
        }

        return true;
    } catch (const std::exception& e) {
        mclog::error("Failed to push frame: {}", e.what());
        return false;
    }
}

void connect(std::string infoDir)
{
    mclog::info("Connecting to cover screen at {}", infoDir);

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
                auto screen_size = j.value("screen_size", std::vector<int>{0, 0});
                screen.width = screen_size[0];
                screen.height = screen_size[1];
                screen.bits_per_pixel = j.value("bits_per_pixel", 0);
                screen.frame_buffer_port = j.value("frame_buffer_port", -1);
                screen.command_port = j.value("command_port", -1);

                std::string addr = "tcp://127.0.0.1:" + std::to_string(screen.frame_buffer_port);
                screen.socket = std::make_unique<zmq::socket_t>(_context, ZMQ_REQ);
                screen.socket->connect(addr);

                mclog::info(
                    "Connected to {} for screen:\n  name: {}\n  size: {} x {}\n  frame_buffer_port: {}\n  "
                    "command_port: {}",
                    addr,
                    screen.name,
                    screen.width,
                    screen.height,
                    screen.frame_buffer_port,
                    screen.command_port);

                auto name = screen.name;
                _screens.push_back(std::move(screen));
                _screen_index_map[name] = _screens.size() - 1;

            } catch (const std::exception& e) {
                mclog::error("Failed to load {}: {}", entry.path().filename().string(), e.what());
            }
        }
    }
}

const std::vector<ScreenInfo_t>& getScreens()
{
    return _screens;
}

bool exists(const std::string& screenName)
{
    for (const auto& screen : _screens) {
        if (screen.name == screenName) {
            return true;
        }
    }
    return false;
}

const ScreenInfo_t& getScreen(const std::string& screenName)
{
    auto it = _screen_index_map.find(screenName);
    if (it == _screen_index_map.end()) {
        throw std::runtime_error("Screen " + screenName + " does not exist");
    }
    return _screens[it->second];
}

bool pushFrame(const std::string& screenName, uint8_t* data, size_t size)
{
    auto it = _screen_index_map.find(screenName);
    if (it == _screen_index_map.end()) {
        throw std::runtime_error("Screen " + screenName + " does not exist");
    }
    return _screens[it->second].pushFrame(data, size);
}

void stop()
{
    for (auto& screen : _screens) {
        if (screen.socket) {
            screen.socket->close();
        }
    }
    _screens.clear();
}

} // namespace cover_screen
