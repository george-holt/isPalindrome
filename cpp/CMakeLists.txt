cmake_minimum_required(VERSION 3.24)
project(is_palindrome_cpp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

add_library(palindrome_core STATIC src/IsPalindrome.cpp)
target_include_directories(palindrome_core PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/include")
if(MSVC)
  target_compile_options(palindrome_core PUBLIC /utf-8)
endif()

include(FetchContent)
FetchContent_Declare(
  Catch2
  GIT_REPOSITORY https://github.com/catchorg/Catch2.git
  GIT_TAG v3.5.4
)
FetchContent_Declare(
  nlohmann_json
  GIT_REPOSITORY https://github.com/nlohmann/json.git
  GIT_TAG v3.11.3
)
FetchContent_MakeAvailable(Catch2 nlohmann_json)

add_executable(acceptance_tests tests/acceptance_tests.cpp)
target_link_libraries(
  acceptance_tests PRIVATE palindrome_core nlohmann_json::nlohmann_json Catch2::Catch2WithMain
)
if(MSVC)
  target_compile_options(acceptance_tests PRIVATE /utf-8)
endif()

add_executable(pal_stdin src/pal_stdin.cpp)
target_link_libraries(pal_stdin PRIVATE palindrome_core nlohmann_json::nlohmann_json)
if(MSVC)
  target_compile_options(pal_stdin PRIVATE /utf-8)
endif()

include(CTest)
add_test(NAME acceptance_manifest COMMAND acceptance_tests)
