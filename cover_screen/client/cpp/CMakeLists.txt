cmake_minimum_required(VERSION 3.14.0)
project(cover-screen-client VERSION 0.1.0 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)

file(GLOB_RECURSE UTILS_DIR "utils/*cpp")

set(MOONCAKE_LOG_BUILD_EXAMPLE OFF)
add_subdirectory(dependencies/mooncake_log)

set(LV_CONF_INCLUDE_SIMPLE OFF)
set(LV_BUILD_CONF_PATH ${CMAKE_SOURCE_DIR}/lv_conf.h CACHE PATH "LVGL config path")
set(LV_CONF_PATH ${CMAKE_SOURCE_DIR}/lv_conf.h CACHE PATH "LVGL config path")
add_subdirectory(dependencies/lvgl)

find_package(cppzmq)

set(JSON_BuildTests OFF CACHE INTERNAL "")
add_subdirectory(dependencies/json)

add_executable(cover-screen-client main.cpp ${UTILS_DIR})

target_link_libraries(cover-screen-client PUBLIC 
    mooncake_log
    lvgl
    lvgl_examples
    lvgl_demos
    cppzmq
    nlohmann_json::nlohmann_json
)

