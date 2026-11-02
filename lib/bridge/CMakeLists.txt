# Bridge library: C ABI between the Rust UI and the protocol core
add_library(nchat-bridge STATIC
  src/bridge.cpp
  src/bridge.h
  src/eventjson.cpp
  src/eventjson.h
  src/protocolfactory.h
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

target_compile_definitions(nchat-bridge PRIVATE
                           CMAKE_INSTALL_LIBDIR="${CMAKE_INSTALL_LIBDIR}")

# Same feature switches as the nchat executable
if(HAS_MULTIPROTOCOL)
  target_compile_definitions(nchat-bridge PRIVATE HAS_MULTIPROTOCOL="${HAS_MULTIPROTOCOL}")
endif()

if(HAS_DYNAMICLOAD)
  target_compile_definitions(nchat-bridge PRIVATE HAS_DYNAMICLOAD="${HAS_DYNAMICLOAD}")
endif()

# Protocols: header + define always, link directly only when not dlopen'ed
macro(bridge_protocol flag lib)
  if(${flag})
    target_include_directories(nchat-bridge PRIVATE "../${lib}/src")
    target_compile_definitions(nchat-bridge PRIVATE ${flag}="${${flag}}")
    if(NOT HAS_DYNAMICLOAD)
      target_link_libraries(nchat-bridge PUBLIC ${lib})
    endif()
  endif()
endmacro()

bridge_protocol(HAS_DUMMY duchat)
bridge_protocol(HAS_TELEGRAM tgchat)
bridge_protocol(HAS_WHATSAPP wmchat)
bridge_protocol(HAS_SIGNAL sgchat)

# Trivial executable whose generated link.txt rust/nchat-core-sys/build.rs parses
add_executable(nchat-bridge-probe src/probe.cpp)
target_common_config(nchat-bridge-probe)
target_link_libraries(nchat-bridge-probe PRIVATE nchat-bridge)
