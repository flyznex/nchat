// test017.cpp - block character image rendering
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <iostream>
#include <vector>

#include "blockimage.h"

#include "unittest.h"

static void Put(std::vector<unsigned char>& p_Img, int p_W, int p_X, int p_Y,
                int p_R, int p_G, int p_B, int p_A = 255)
{
  unsigned char* p = &p_Img[static_cast<size_t>((p_Y * p_W + p_X) * 4)];
  p[0] = static_cast<unsigned char>(p_R);
  p[1] = static_cast<unsigned char>(p_G);
  p[2] = static_cast<unsigned char>(p_B);
  p[3] = static_cast<unsigned char>(p_A);
}

static std::vector<unsigned char> Solid(int p_W, int p_H, int p_R, int p_G, int p_B, int p_A = 255)
{
  std::vector<unsigned char> img(static_cast<size_t>(p_W * p_H * 4));
  for (int y = 0; y < p_H; ++y)
  {
    for (int x = 0; x < p_W; ++x) Put(img, p_W, x, y, p_R, p_G, p_B, p_A);
  }

  return img;
}

// one cell from a 2x2 image, pixels in order top left, top right, bottom left, bottom right
static BlockPaint Cell(const int p_Rgba[4][4])
{
  std::vector<unsigned char> img = Solid(2, 2, 0, 0, 0);
  for (int i = 0; i < 4; ++i) Put(img, 2, i % 2, i / 2, p_Rgba[i][0], p_Rgba[i][1], p_Rgba[i][2], p_Rgba[i][3]);

  BlockGrid g = BlockImage::Render(img.data(), 2, 2, 1, 1);
  return g.At(0, 0);
}

static void TestNearestPalette()
{
  unittest::ExpectEqual(int, BlockImage::NearestPalette(0, 0, 0), 16);
  unittest::ExpectEqual(int, BlockImage::NearestPalette(255, 0, 0), 196);
  unittest::ExpectEqual(int, BlockImage::NearestPalette(0, 0, 255), 21);
  unittest::ExpectEqual(int, BlockImage::NearestPalette(255, 255, 255), 231);
  unittest::ExpectEqual(int, BlockImage::NearestPalette(128, 128, 128), 244); // grey ramp beats the cube
}

static void TestRenderSolid()
{
  std::vector<unsigned char> img = Solid(4, 4, 255, 0, 0);
  BlockGrid g = BlockImage::Render(img.data(), 4, 4, 1, 1);
  unittest::ExpectEqual(int, g.cols, 1);
  unittest::ExpectEqual(int, g.rows, 1);
  unittest::ExpectEqual(int, static_cast<int>(g.At(0, 0).ch), 0x2588);
  unittest::ExpectEqual(int, g.At(0, 0).fg, 196);
  unittest::ExpectEqual(int, g.At(0, 0).bg, -1);
}

static void TestRenderSplits()
{
  const int red[4] = { 255, 0, 0, 255 };
  const int blue[4] = { 0, 0, 255, 255 };

  // top red, bottom blue: upper half block
  const int topBottom[4][4] = { { red[0], red[1], red[2], red[3] }, { red[0], red[1], red[2], red[3] },
                                { blue[0], blue[1], blue[2], blue[3] }, { blue[0], blue[1], blue[2], blue[3] } };
  BlockPaint p = Cell(topBottom);
  unittest::ExpectEqual(int, static_cast<int>(p.ch), 0x2580);
  unittest::ExpectEqual(int, p.fg, 196);
  unittest::ExpectEqual(int, p.bg, 21);

  // left red, right blue: left half block
  const int leftRight[4][4] = { { red[0], red[1], red[2], red[3] }, { blue[0], blue[1], blue[2], blue[3] },
                                { red[0], red[1], red[2], red[3] }, { blue[0], blue[1], blue[2], blue[3] } };
  p = Cell(leftRight);
  unittest::ExpectEqual(int, static_cast<int>(p.ch), 0x258C);
  unittest::ExpectEqual(int, p.fg, 196);
  unittest::ExpectEqual(int, p.bg, 21);

  // red on the diagonal, blue on the other one: the glyph covers the group without the bottom right pixel
  const int diagonal[4][4] = { { red[0], red[1], red[2], red[3] }, { blue[0], blue[1], blue[2], blue[3] },
                               { blue[0], blue[1], blue[2], blue[3] }, { red[0], red[1], red[2], red[3] } };
  p = Cell(diagonal);
  unittest::ExpectEqual(int, static_cast<int>(p.ch), 0x259E);
  unittest::ExpectEqual(int, p.fg, 21);
  unittest::ExpectEqual(int, p.bg, 196);

  // one white pixel on black: quadrant glyph
  const int white[4] = { 255, 255, 255, 255 };
  const int black[4] = { 0, 0, 0, 255 };
  const int corner[4][4] = { { white[0], white[1], white[2], white[3] }, { black[0], black[1], black[2], black[3] },
                             { black[0], black[1], black[2], black[3] }, { black[0], black[1], black[2], black[3] } };
  p = Cell(corner);
  unittest::ExpectEqual(int, static_cast<int>(p.ch), 0x2598);
  unittest::ExpectEqual(int, p.fg, 231);
  unittest::ExpectEqual(int, p.bg, 16);
}

static void TestRenderTransparent()
{
  std::vector<unsigned char> none = Solid(2, 2, 255, 0, 0, 0);
  BlockGrid g = BlockImage::Render(none.data(), 2, 2, 1, 1);
  unittest::ExpectEqual(int, static_cast<int>(g.At(0, 0).ch), 0x20);
  unittest::ExpectEqual(int, g.At(0, 0).bg, -1);

  // bottom row transparent: upper half block on the history background
  std::vector<unsigned char> bottom = Solid(2, 2, 255, 0, 0);
  Put(bottom, 2, 0, 1, 255, 0, 0, 0);
  Put(bottom, 2, 1, 1, 255, 0, 0, 0);
  g = BlockImage::Render(bottom.data(), 2, 2, 1, 1);
  unittest::ExpectEqual(int, static_cast<int>(g.At(0, 0).ch), 0x2580);
  unittest::ExpectEqual(int, g.At(0, 0).fg, 196);
  unittest::ExpectEqual(int, g.At(0, 0).bg, -1);

  // left column transparent: right half block
  std::vector<unsigned char> left = Solid(2, 2, 255, 0, 0);
  Put(left, 2, 0, 0, 255, 0, 0, 0);
  Put(left, 2, 0, 1, 255, 0, 0, 0);
  g = BlockImage::Render(left.data(), 2, 2, 1, 1);
  unittest::ExpectEqual(int, static_cast<int>(g.At(0, 0).ch), 0x2590);
  unittest::ExpectEqual(int, g.At(0, 0).fg, 196);
  unittest::ExpectEqual(int, g.At(0, 0).bg, -1);
}

static void TestRenderAverage()
{
  // 4x4 into one cell: the top left 2x2 block is half white, half black, the rest is black
  std::vector<unsigned char> img = Solid(4, 4, 0, 0, 0);
  Put(img, 4, 0, 0, 255, 255, 255);
  Put(img, 4, 0, 1, 255, 255, 255);

  BlockGrid g = BlockImage::Render(img.data(), 4, 4, 1, 1);
  unittest::ExpectEqual(int, static_cast<int>(g.At(0, 0).ch), 0x2598);
  unittest::ExpectEqual(int, g.At(0, 0).fg, 244); // average 127 is a mid grey
  unittest::ExpectEqual(int, g.At(0, 0).bg, 16);
}

static void TestRenderGridSize()
{
  std::vector<unsigned char> img = Solid(8, 6, 0, 255, 0);
  BlockGrid g = BlockImage::Render(img.data(), 8, 6, 4, 3);
  unittest::ExpectEqual(int, g.cols, 4);
  unittest::ExpectEqual(int, g.rows, 3);
  unittest::ExpectEqual(int, static_cast<int>(g.cells.size()), 12);
  unittest::ExpectEqual(int, g.At(2, 3).fg, 46);
}

static void TestRenderInvalid()
{
  BlockGrid g = BlockImage::Render(nullptr, 2, 2, 1, 1);
  unittest::ExpectEqual(int, g.cols, 0);
  std::vector<unsigned char> img = Solid(2, 2, 0, 0, 0);
  g = BlockImage::Render(img.data(), 2, 2, 0, 1);
  unittest::ExpectEqual(int, g.cols, 0);
}

static void TestRenderPngMissing()
{
  BlockGrid g;
  unittest::ExpectFalse(BlockImage::RenderPng("/nonexistent/nchat-test.png", 2, 2, g));
}

int main()
{
  int rv = 0;
  try
  {
    TestNearestPalette();
    TestRenderSolid();
    TestRenderSplits();
    TestRenderTransparent();
    TestRenderAverage();
    TestRenderGridSize();
    TestRenderInvalid();
    TestRenderPngMissing();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
