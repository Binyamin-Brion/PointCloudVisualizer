cmake_minimum_required(VERSION 3.20)
project(ClusterDetectionExe)

set(CMAKE_CXX_STANDARD 14)

find_package(Open3D REQUIRED)
include_directories(${Open3D_INCLUDE_DIRECTORIES})

option(STATIC_WINDOWS_RUNTIME "Use static (MT/MTd) Windows runtime" ON)
if(STATIC_WINDOWS_RUNTIME)
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
else()
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>DLL")
endif()


add_executable(ClusterDetectionExe main.cpp)

target_link_libraries(ClusterDetectionExe ${Open3D_LIBRARIES})
