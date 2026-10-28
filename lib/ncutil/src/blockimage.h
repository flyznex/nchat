// blockimage.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <string>
#include <vector>

struct BlockCell
{
  int top = -1;    // xterm 256 palette index, -1 = transparent
  int bottom = -1;
};

struct BlockGrid
{
  int cols = 0;
  int rows = 0;
  std::vector<BlockCell> cells; // row-major

  const BlockCell& At(int p_Row, int p_Col) const { return cells[static_cast<size_t>(p_Row * cols + p_Col)]; }
};

struct BlockPaint
{
  wchar_t ch = L' ';
  int fg = 0;
  int bg = -1; // -1 = history background
};

class BlockImage
{
public:
  static int NearestPalette(int p_R, int p_G, int p_B);
  static BlockGrid Render(const unsigned char* p_Rgba, int p_Width, int p_Height, int p_Cols, int p_Rows);
  static BlockPaint Paint(const BlockCell& p_Cell);
  static bool RenderPng(const std::string& p_Path, int p_Cols, int p_Rows, BlockGrid& p_Grid);
};
