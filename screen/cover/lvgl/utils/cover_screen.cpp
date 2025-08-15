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
#include <zmq.hpp>
#include <vector>
#include <memory>
#include <cstdlib>
#include <stdexcept>

using json = nlohmann::json;

namespace cover_screen {

static const int HTTP_PORT = 12580;
static zmq::context_t _context(1);
static std::vector<ScreenInfo_t> _screens;
static std::unordered_map<std::string, int> _screen_index_map;

// 执行HTTP GET请求（使用系统curl命令）
static std::string httpGet(const std::string& url)
{
    std::string command = "curl -s --connect-timeout 10 \"" + url + "\"";

    FILE* pipe = popen(command.c_str(), "r");
    if (!pipe) {
        throw std::runtime_error("无法执行curl命令");
    }

    std::string result;
    char buffer[128];
    while (fgets(buffer, sizeof(buffer), pipe) != nullptr) {
        result += buffer;
    }

    int returnCode = pclose(pipe);
    if (returnCode != 0) {
        throw std::runtime_error("HTTP请求失败，curl返回码: " + std::to_string(returnCode));
    }

    return result;
}

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
            throw std::runtime_error("Failed to receive reply from " + id);
        }

        return true;
    } catch (const std::exception& e) {
        mclog::error("Failed to push frame: {}", e.what());
        return false;
    }
}

void connect(const std::string& apiUrl)
{
    mclog::info("Connecting to cover screen API at {}", apiUrl);

    stop();

    try {
        // 获取所有设备列表
        std::string url = apiUrl + "/devices";
        std::string response = httpGet(url);

        json deviceList = json::parse(response);

        for (const auto& deviceId : deviceList) {
            try {
                // 获取每个设备的详细信息
                std::string infoUrl = apiUrl + "/" + deviceId.get<std::string>() + "/info";
                std::string infoResponse = httpGet(infoUrl);

                json deviceInfo = json::parse(infoResponse);

                // 只处理屏幕设备（跳过 imu0 等非屏幕设备）
                if (deviceInfo.contains("screen_size") && deviceInfo.contains("frame_buffer_port")) {
                    ScreenInfo_t screen;
                    screen.id = deviceId.get<std::string>();

                    screen.description = deviceInfo.value("description", "");
                    screen.device_type = deviceInfo.value("device_type", "");

                    auto screen_size = deviceInfo.value("screen_size", std::vector<int>{0, 0});
                    screen.width = screen_size[0];
                    screen.height = screen_size[1];
                    screen.bits_per_pixel = deviceInfo.value("bits_per_pixel", 0);
                    screen.frame_buffer_port = deviceInfo.value("frame_buffer_port", -1);

                    std::string addr = "tcp://127.0.0.1:" + std::to_string(screen.frame_buffer_port);
                    screen.socket = std::make_unique<zmq::socket_t>(_context, ZMQ_REQ);
                    screen.socket->connect(addr);

                    mclog::info(
                        "Connected to {} for screen:\n  id: {}\n  size: {} x {}\n  bpp: {}\n  frame_buffer_port: {}\n  "
                        "device_type: {}",
                        addr,
                        screen.id,
                        screen.width,
                        screen.height,
                        screen.bits_per_pixel,
                        screen.frame_buffer_port,
                        screen.device_type);

                    auto id = screen.id;
                    _screens.push_back(std::move(screen));
                    _screen_index_map[id] = _screens.size() - 1;
                }

            } catch (const std::exception& e) {
                mclog::error("Failed to process device {}: {}", deviceId.get<std::string>(), e.what());
            }
        }

    } catch (const std::exception& e) {
        mclog::error("Failed to connect to API: {}", e.what());
    }
}

const std::vector<ScreenInfo_t>& getScreens()
{
    return _screens;
}

bool exists(const std::string& screenName)
{
    for (const auto& screen : _screens) {
        if (screen.id == screenName) {
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
