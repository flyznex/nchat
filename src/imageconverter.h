// imageconverter.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <string>

class ImageConverter
{
public:
  static bool ToPng(const std::string& p_SrcPath, const std::string& p_DstPngPath);
};
