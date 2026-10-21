// chatid.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <cstdint>
#include <cstdlib>
#include <string>

#include "strutil.h"

struct ChatRef
{
  int64_t chatId;
  int32_t topicId; // 0 = not a topic chat (plain chat, or General via the parent id)
};

inline ChatRef ParseChatId(const std::string& p_Str)
{
  ChatRef ref;
  const size_t sep = p_Str.find(':');
  ref.chatId = StrUtil::NumFromHex<int64_t>(p_Str.substr(0, sep));
  ref.topicId = (sep == std::string::npos)
    ? 0 : static_cast<int32_t>(std::strtol(p_Str.c_str() + sep + 1, nullptr, 10));
  return ref;
}

inline std::string MakeChatId(int64_t p_ChatId, int32_t p_TopicId)
{
  std::string str = StrUtil::NumToHex(p_ChatId);
  if (p_TopicId > 1)
  {
    str += ":" + std::to_string(p_TopicId);
  }

  return str;
}
