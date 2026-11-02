# Bridge library: C ABI between the Rust UI and the protocol core
add_library(nchat-bridge STATIC
  src/eventjson.cpp
  src/eventjson.h
  src/requestjson.cpp
  src/requestjson.h
)

target_common_config(nchat-bridge)

target_include_directories(nchat-bridge PUBLIC "src")
target_include_directories(nchat-bridge PRIVATE "../common/src")
target_include_directories(nchat-bridge PRIVATE "../ncutil/src")
target_include_directories(nchat-bridge PRIVATE "../ncutil/ext/cereal/include")

set_target_properties(nchat-bridge PROPERTIES COMPILE_FLAGS
                      "-Wall -Wextra -Wpedantic -Wshadow -Wpointer-arith \
                       -Wcast-qual -Wno-missing-braces -Wswitch-default \
                       -Wunreachable-code -Wundef -Wuninitialized \
                       -Wcast-align")

target_link_libraries(nchat-bridge PUBLIC ncutil pthread ${CMAKE_DL_LIBS})
