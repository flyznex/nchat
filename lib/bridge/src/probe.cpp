// probe.cpp - exists so CMake writes a complete link line (link.txt) for the Rust build.rs
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "bridge.h"

int main()
{
  return (nc_abi_version() == NC_ABI_VERSION) ? 0 : 1;
}
