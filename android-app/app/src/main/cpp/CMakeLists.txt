cmake_minimum_required(VERSION 3.22.1)
project("astrashell-native" C CXX)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find the Rust shared library
add_library(astrashell SHARED IMPORTED)
set_target_properties(astrashell PROPERTIES
    IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/../../../runtime/target/aarch64-linux-android/release/libastrashell.so
)

# JNI bridge - links Rust runtime to Android Java/Kotlin
add_library(astrashell_jni SHARED
    jni_bridge.c
    engine_bridge.c
    avf_bridge.c
)

target_link_libraries(astrashell_jni
    astrashell
    log
    android
)

target_include_directories(astrashell_jni PRIVATE
    ${CMAKE_SOURCE_DIR}/../../../runtime/c-api
)

# Build flags for Android NDK
target_compile_options(astrashell_jni PRIVATE
    -Wall -Wextra -Werror
    -fvisibility=hidden
    -ffunction-sections
    -fdata-sections
)

set_target_properties(astrashell_jni PROPERTIES
    LINK_FLAGS "-Wl,--gc-sections -Wl,-z,noexecstack -Wl,-z,relro -Wl,-z,now"
)
