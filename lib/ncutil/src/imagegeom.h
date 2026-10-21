// imagegeom.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <string>

enum class ImageKind
{
  None,
  Photo,
  Sticker,
};

struct CellGrid
{
  int cols = 0;
  int rows = 0;
};

class ImageGeom
{
public:
  static ImageKind DetectKind(const std::string& p_FileType, const std::string& p_FilePath);
  static bool ParsePngSize(const std::string& p_Bytes, int& p_Width, int& p_Height);
  static CellGrid FitToCells(int p_PxWidth, int p_PxHeight, int p_CellPxWidth, int p_CellPxHeight,
                             int p_MaxCols, int p_MaxRows);
};
