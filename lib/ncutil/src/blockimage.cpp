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

  // quadrant glyph by mask of set pixels: 1 = top left, 2 = top right, 4 = bottom left, 8 = bottom right
  const wchar_t s_Glyphs[16] = { L' ',      L'▘', L'▝', L'▀', L'▖', L'▌', L'▞', L'▛',
                                 L'▗', L'▚', L'▐', L'▜', L'▄', L'▙', L'▟', L'█' };

  struct Pixel
  {
    bool opaque = false;
    double rgb[3] = { 0, 0, 0 };
  };

  struct Group
  {
    double mean[3] = { 0, 0, 0 };
    double err = 0;
    int count = 0;
  };

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

  Group Summarize(const Pixel* p_Px, int p_Mask)
  {
    Group g;
    for (int i = 0; i < 4; ++i)
    {
      if ((p_Mask & (1 << i)) == 0) continue;
      for (int c = 0; c < 3; ++c) g.mean[c] += p_Px[i].rgb[c];
      ++g.count;
    }

    if (g.count == 0) return g;

    for (int c = 0; c < 3; ++c) g.mean[c] /= g.count;
    for (int i = 0; i < 4; ++i)
    {
      if ((p_Mask & (1 << i)) == 0) continue;
      for (int c = 0; c < 3; ++c) g.err += (p_Px[i].rgb[c] - g.mean[c]) * (p_Px[i].rgb[c] - g.mean[c]);
    }

    return g;
  }

  int ToPalette(const Group& p_Group)
  {
    return BlockImage::NearestPalette(static_cast<int>(p_Group.mean[0] + 0.5), static_cast<int>(p_Group.mean[1] + 0.5),
                                      static_cast<int>(p_Group.mean[2] + 0.5));
  }

  BlockPaint PaintCell(const Pixel* p_Px)
  {
    BlockPaint paint;
    int opaqueMask = 0;
    for (int i = 0; i < 4; ++i)
    {
      if (p_Px[i].opaque) opaqueMask |= (1 << i);
    }

    if (opaqueMask == 0) return paint;

    if (opaqueMask != 15)
    {
      paint.ch = s_Glyphs[opaqueMask];
      paint.fg = ToPalette(Summarize(p_Px, opaqueMask));
      return paint;
    }

    // all opaque: split into two colour groups, bottom right always in the background group
    int bestMask = 1;
    double bestErr = -1;
    for (int mask = 1; mask <= 7; ++mask)
    {
      const double err = Summarize(p_Px, mask).err + Summarize(p_Px, 15 & ~mask).err;
      if ((bestErr < 0) || (err < bestErr))
      {
        bestErr = err;
        bestMask = mask;
      }
    }

    const int fg = ToPalette(Summarize(p_Px, bestMask));
    const int bg = ToPalette(Summarize(p_Px, 15 & ~bestMask));
    if (fg == bg)
    {
      paint.ch = s_Glyphs[15];
      paint.fg = fg;
      return paint;
    }

    paint.ch = s_Glyphs[bestMask];
    paint.fg = fg;
    paint.bg = bg;
    return paint;
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

  const int pxCols = p_Cols * 2;
  const int pxRows = p_Rows * 2;
  auto sample = [&](int p_Ox, int p_Oy) -> Pixel
  {
    const int x0 = static_cast<int>(static_cast<long long>(p_Ox) * p_Width / pxCols);
    const int x1 = std::min(p_Width, std::max(x0 + 1, static_cast<int>(static_cast<long long>(p_Ox + 1) * p_Width / pxCols)));
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

    Pixel pixel;
    if ((count == 0) || (sumA < 128 * count)) return pixel;

    pixel.opaque = true;
    pixel.rgb[0] = static_cast<double>(sumR) / static_cast<double>(sumA);
    pixel.rgb[1] = static_cast<double>(sumG) / static_cast<double>(sumA);
    pixel.rgb[2] = static_cast<double>(sumB) / static_cast<double>(sumA);
    return pixel;
  };

  for (int row = 0; row < p_Rows; ++row)
  {
    for (int col = 0; col < p_Cols; ++col)
    {
      const Pixel px[4] = { sample(col * 2, row * 2), sample(col * 2 + 1, row * 2),
                            sample(col * 2, row * 2 + 1), sample(col * 2 + 1, row * 2 + 1) };
      grid.cells[static_cast<size_t>(row * p_Cols + col)] = PaintCell(px);
    }
  }

  return grid;
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
