// termgfx.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <string>

class TermGfx
{
public:
  static void Init();
  static bool IsEnabled();
  static void GetCellPixelSize(int& p_Width, int& p_Height);
  static bool Upload(int p_Id, int p_Cols, int p_Rows, const std::string& p_PngPath);
  static void Delete(int p_Id);

private:
  static void Write(const std::string& p_Seq);

  static bool m_Enabled;
  static bool m_InTmux;
};
