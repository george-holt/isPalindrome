cmake_minimum_required(VERSION 3.16)
project(is_palindrome_c C)

set(CMAKE_C_STANDARD 99)

add_library(is_palindrome STATIC src/is_palindrome.c)
target_include_directories(is_palindrome PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/include")

set(CJSON_BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
set(ENABLE_CJSON_TEST OFF CACHE BOOL "" FORCE)
set(ENABLE_CJSON_UTILS OFF CACHE BOOL "" FORCE)
include(FetchContent)
FetchContent_Declare(
    cjson
    GIT_REPOSITORY https://github.com/DaveGamble/cJSON.git
    GIT_TAG v1.7.18
)
FetchContent_MakeAvailable(cjson)

get_filename_component(IS_PAL_REPO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/.." ABSOLUTE)
set(MANIFEST_JSON "${IS_PAL_REPO_ROOT}/fixtures/acceptance_manifest.json")
string(REPLACE "\\" "/" MANIFEST_JSON "${MANIFEST_JSON}")

add_executable(acceptance_test tests/acceptance_manifest.c)
target_link_libraries(acceptance_test PRIVATE is_palindrome cjson)
target_include_directories(acceptance_test PRIVATE "${cjson_SOURCE_DIR}")
target_compile_definitions(acceptance_test PRIVATE "PAL_MANIFEST_PATH=\"${MANIFEST_JSON}\"")

add_executable(pal_stdin src/pal_stdin.c)
target_link_libraries(pal_stdin PRIVATE is_palindrome cjson)
target_include_directories(pal_stdin PRIVATE "${cjson_SOURCE_DIR}")

enable_testing()
add_test(NAME acceptance_manifest COMMAND acceptance_test)
