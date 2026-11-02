// requestjson.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <memory>
#include <string>

#include "protocol.h"

struct ParsedRequest
{
  std::string profileId;
  std::shared_ptr<RequestMessage> request;
};

bool RequestFromJson(const std::string& p_Json, ParsedRequest& p_Out, std::string& p_Error);
