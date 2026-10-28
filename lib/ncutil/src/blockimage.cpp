// blockimage.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "blockimage.h"

#include <algorithm>
#include <cstdlib>

#include "stb_image.h"

namespace
{
  const int s_CubeLevels[6] = { 0, 95, 135, 175, 215, 255 };

  int Sq(int p_V)
  {
    return p_V * p_V;
  }

  int NearestCubeIndex(int p_V)
  {
    int best = 0;
    for (int i = 1; i < 6; ++i)
    {
      if (std::abs(p_V - s_CubeLevels[i]) < std::abs(p_V - s_CubeLevels[best])) best = i;
    }

    return best;
  }
}

int BlockImage::NearestPalette(int p_R, int p_G, int p_B)
{
  const int r = std::max(0, std::min(255, p_R));
  const int g = std::max(0, std::min(255, p_G));
  const int b = std::max(0, std::min(255, p_B));

  const int ri = NearestCubeIndex(r);
  const int gi = NearestCubeIndex(g);
  const int bi = NearestCubeIndex(b);
  const int cubeDist = Sq(r - s_CubeLevels[ri]) + Sq(g - s_CubeLevels[gi]) + Sq(b - s_CubeLevels[bi]);

  const int greyIdx = std::max(0, std::min(23, ((r + g + b) / 3 - 3) / 10));
  const int grey = 8 + 10 * greyIdx;
  const int greyDist = Sq(r - grey) + Sq(g - grey) + Sq(b - grey);

  return (greyDist < cubeDist) ? (232 + greyIdx) : (16 + 36 * ri + 6 * gi + bi);
}

BlockGrid BlockImage::Render(const unsigned char* p_Rgba, int p_Width, int p_Height, int p_Cols, int p_Rows)
{
  BlockGrid grid;
  if ((p_Rgba == nullptr) || (p_Width <= 0) || (p_Height <= 0) || (p_Cols <= 0) || (p_Rows <= 0)) return grid;

  grid.cols = p_Cols;
  grid.rows = p_Rows;
  grid.cells.resize(static_cast<size_t>(p_Cols * p_Rows));

  const int pxRows = p_Rows * 2;
  auto sample = [&](int p_Ox, int p_Oy) -> int
  {
    const int x0 = static_cast<int>(static_cast<long long>(p_Ox) * p_Width / p_Cols);
    const int x1 = std::min(p_Width, std::max(x0 + 1, static_cast<int>(static_cast<long long>(p_Ox + 1) * p_Width / p_Cols)));
    const int y0 = static_cast<int>(static_cast<long long>(p_Oy) * p_Height / pxRows);
    const int y1 = std::min(p_Height, std::max(y0 + 1, static_cast<int>(static_cast<long long>(p_Oy + 1) * p_Height / pxRows)));

    long long sumA = 0;
    long long sumR = 0;
    long long sumG = 0;
    long long sumB = 0;
    long long count = 0;
    for (int y = y0; y < y1; ++y)
    {
      for (int x = x0; x < x1; ++x)
      {
        const unsigned char* px = p_Rgba + (static_cast<size_t>(y) * static_cast<size_t>(p_Width) + static_cast<size_t>(x)) * 4;
        sumA += px[3];
        sumR += px[0] * px[3];
        sumG += px[1] * px[3];
        sumB += px[2] * px[3];
        ++count;
      }
    }

    if ((count == 0) || (sumA < 128 * count)) return -1;

    return NearestPalette(static_cast<int>(sumR / sumA), static_cast<int>(sumG / sumA),
                          static_cast<int>(sumB / sumA));
  };

  for (int row = 0; row < p_Rows; ++row)
  {
    for (int col = 0; col < p_Cols; ++col)
    {
      BlockCell& cell = grid.cells[static_cast<size_t>(row * p_Cols + col)];
      cell.top = sample(col, row * 2);
      cell.bottom = sample(col, row * 2 + 1);
    }
  }

  return grid;
}

BlockPaint BlockImage::Paint(const BlockCell& p_Cell)
{
  BlockPaint paint;
  if ((p_Cell.top < 0) && (p_Cell.bottom < 0))
  {
    paint.ch = L' ';
  }
  else if (p_Cell.top < 0)
  {
    paint.ch = L'▄';
    paint.fg = p_Cell.bottom;
  }
  else if (p_Cell.bottom < 0)
  {
    paint.ch = L'▀';
    paint.fg = p_Cell.top;
  }
  else if (p_Cell.top == p_Cell.bottom)
  {
    paint.ch = L'█';
    paint.fg = p_Cell.top;
  }
  else
  {
    paint.ch = L'▀';
    paint.fg = p_Cell.top;
    paint.bg = p_Cell.bottom;
  }

  return paint;
}

bool BlockImage::RenderPng(const std::string& p_Path, int p_Cols, int p_Rows, BlockGrid& p_Grid)
{
  int w = 0;
  int h = 0;
  int n = 0;
  unsigned char* data = stbi_load(p_Path.c_str(), &w, &h, &n, 4);
  if (data == nullptr) return false;

  p_Grid = Render(data, w, h, p_Cols, p_Rows);
  stbi_image_free(data);
  return !p_Grid.cells.empty();
}
