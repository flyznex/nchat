// mdstyle.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <cstdint>
#include <string>
#include <vector>

enum MdStyle : uint8_t
{
  MdStyleNone = 0,
  MdStyleBold = 1,
  MdStyleItalic = 2,
  MdStyleCode = 4,
};

class MdStyleUtil
{
public:
  // Strips inline markdown (*bold*, _italic_, `code`, ```pre```, backslash escapes) from p_In.
  // p_Styles gets one MdStyle bit mask per char in p_Out.
  static void Parse(const std::wstring& p_In, std::wstring& p_Out, std::vector<uint8_t>& p_Styles);

  // Maps p_Styles of p_Text onto word wrapped p_Lines (which only drop spaces/newlines or expand tabs).
  static std::vector<std::vector<uint8_t>> WrapStyles(const std::wstring& p_Text,
                                                      const std::vector<uint8_t>& p_Styles,
                                                      const std::vector<std::wstring>& p_Lines);
};
