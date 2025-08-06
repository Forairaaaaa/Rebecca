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
#include <cstdint>
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

// basically for python screen service
static uint8_t* _converted_data = nullptr;
static void convert_to_bpp32(const ScreenInfo_t& screen, uint8_t* data, size_t size)
{
    if (!_converted_data) {
        _converted_data = new uint8_t[screen.width * screen.height * sizeof(uint32_t)];
    }

    uint32_t* dst = reinterpret_cast<uint32_t*>(_converted_data);
    uint16_t* src = reinterpret_cast<uint16_t*>(data);

    for (size_t i = 0; i < screen.width * screen.height; ++i) {
        uint16_t pixel = src[i];

        // 提取 RGB565 分量
        uint8_t r5 = (pixel >> 11) & 0x1F;
        uint8_t g6 = (pixel >> 5) & 0x3F;
        uint8_t b5 = pixel & 0x1F;

        // 转换成 8-bit 分量（扩展位数）
        uint8_t r8 = (r5 << 3) | (r5 >> 2); // 5-bit -> 8-bit
        uint8_t g8 = (g6 << 2) | (g6 >> 4); // 6-bit -> 8-bit
        uint8_t b8 = (b5 << 3) | (b5 >> 2); // 5-bit -> 8-bit

        // 组装成 RGBA（A通道设为255）
        dst[i] = (255 << 24) | (b8 << 16) | (g8 << 8) | r8;
    }
}

bool ScreenInfo_t::pushFrame(uint8_t* data, size_t size)
{
    try {
        if (bits_per_pixel == 32) {
            convert_to_bpp32(*this, data, size);
            data = _converted_data;
            size = width * height * sizeof(uint32_t);
        }

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
                    "Connected to {} for screen:\n  name: {}\n  size: {} x {}\n  bbp: {}\n  frame_buffer_port: {}\n  "
                    "command_port: {}",
                    addr,
                    screen.name,
                    screen.width,
                    screen.height,
                    screen.bits_per_pixel,
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
