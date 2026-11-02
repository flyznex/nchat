// eventjson.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <memory>
#include <string>

#include "protocol.h"

std::string EventToJson(const std::shared_ptr<ServiceMessage>& p_Message);
