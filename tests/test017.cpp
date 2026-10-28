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
  std::vector<unsigned char> img = Solid(2, 2, 255, 0, 0);
  BlockGrid g = BlockImage::Render(img.data(), 2, 2, 1, 1);
  unittest::ExpectEqual(int, g.cols, 1);
  unittest::ExpectEqual(int, g.rows, 1);
  unittest::ExpectEqual(int, g.At(0, 0).top, 196);
  unittest::ExpectEqual(int, g.At(0, 0).bottom, 196);
}

static void TestRenderTwoPixels()
{
  std::vector<unsigned char> img = Solid(1, 2, 0, 0, 0);
  Put(img, 1, 0, 0, 255, 0, 0);
  Put(img, 1, 0, 1, 0, 0, 255);
  BlockGrid g = BlockImage::Render(img.data(), 1, 2, 1, 1);
  unittest::ExpectEqual(int, g.At(0, 0).top, 196);
  unittest::ExpectEqual(int, g.At(0, 0).bottom, 21);
}

static void TestRenderTransparent()
{
  std::vector<unsigned char> none = Solid(2, 2, 255, 0, 0, 0);
  BlockGrid g = BlockImage::Render(none.data(), 2, 2, 1, 1);
  unittest::ExpectEqual(int, g.At(0, 0).top, -1);
  unittest::ExpectEqual(int, g.At(0, 0).bottom, -1);

  std::vector<unsigned char> half = Solid(1, 2, 255, 0, 0);
  Put(half, 1, 0, 1, 255, 0, 0, 0);
  g = BlockImage::Render(half.data(), 1, 2, 1, 1);
  unittest::ExpectEqual(int, g.At(0, 0).top, 196);
  unittest::ExpectEqual(int, g.At(0, 0).bottom, -1);
}

static void TestRenderAverage()
{
  // 4x4 into one cell (4x2 sample): top half is left white / right black, bottom half black
  std::vector<unsigned char> img = Solid(4, 4, 0, 0, 0);
  for (int y = 0; y < 2; ++y)
  {
    for (int x = 0; x < 2; ++x) Put(img, 4, x, y, 255, 255, 255);
  }

  BlockGrid g = BlockImage::Render(img.data(), 4, 4, 1, 1);
  unittest::ExpectEqual(int, g.At(0, 0).top, 244);   // average 127 is a mid grey
  unittest::ExpectEqual(int, g.At(0, 0).bottom, 16); // black
}

static void TestRenderInvalid()
{
  BlockGrid g = BlockImage::Render(nullptr, 2, 2, 1, 1);
  unittest::ExpectEqual(int, g.cols, 0);
  std::vector<unsigned char> img = Solid(2, 2, 0, 0, 0);
  g = BlockImage::Render(img.data(), 2, 2, 0, 1);
  unittest::ExpectEqual(int, g.cols, 0);
}

static void TestPaint()
{
  BlockPaint p = BlockImage::Paint(BlockCell{ 196, 21 });
  unittest::ExpectEqual(int, static_cast<int>(p.ch), 0x2580);
  unittest::ExpectEqual(int, p.fg, 196);
  unittest::ExpectEqual(int, p.bg, 21);

  p = BlockImage::Paint(BlockCell{ 196, 196 });
  unittest::ExpectEqual(int, static_cast<int>(p.ch), 0x2588);
  unittest::ExpectEqual(int, p.fg, 196);
  unittest::ExpectEqual(int, p.bg, -1);

  p = BlockImage::Paint(BlockCell{ -1, 21 });
  unittest::ExpectEqual(int, static_cast<int>(p.ch), 0x2584);
  unittest::ExpectEqual(int, p.fg, 21);
  unittest::ExpectEqual(int, p.bg, -1);

  p = BlockImage::Paint(BlockCell{ 196, -1 });
  unittest::ExpectEqual(int, static_cast<int>(p.ch), 0x2580);
  unittest::ExpectEqual(int, p.fg, 196);
  unittest::ExpectEqual(int, p.bg, -1);

  p = BlockImage::Paint(BlockCell{ -1, -1 });
  unittest::ExpectEqual(int, static_cast<int>(p.ch), 0x20);
  unittest::ExpectEqual(int, p.bg, -1);
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
    TestRenderTwoPixels();
    TestRenderTransparent();
    TestRenderAverage();
    TestRenderInvalid();
    TestPaint();
    TestRenderPngMissing();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
