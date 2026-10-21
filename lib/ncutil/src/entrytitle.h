// entrytitle.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <string>

enum class EntryMode
{
  Compose,
  Reply,
  Edit,
};

// Title line of the entry box, single line and exactly p_MaxW display columns wide (padded or truncated, wide
// characters are never split). p_Who is the reply target name (newlines are stripped, empty falls back to
// "message"), p_Hint the send/newline key hint, p_CancelKey the key shown for cancel. A long p_Who is shortened
// first so the key hints stay visible.
std::wstring BuildEntryTitle(EntryMode p_Mode, const std::string& p_Who, const std::string& p_Hint,
                             const std::string& p_CancelKey, int p_MaxW);
