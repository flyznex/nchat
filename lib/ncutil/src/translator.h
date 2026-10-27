// translator.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <functional>
#include <string>

class Translator
{
public:
  enum class State { Pending, Ready, Unchanged, Failed };

  static void Init(const std::string& p_Cmd, const std::string& p_Target, std::function<void()> p_OnUpdate);
  static void Cleanup();
  static State Get(const std::string& p_Text, std::string& p_Translated);
  static bool TranslateNow(const std::string& p_Text, std::string& p_Translated, std::string& p_Error);
  static bool IsValidLang(const std::string& p_Lang);
};
