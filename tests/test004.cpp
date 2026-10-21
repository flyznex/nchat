// test004.cpp - image kind detection and cell geometry for inline images
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <cstdint>
#include <iostream>
#include <string>

#include "imagegeom.h"

#include "unittest.h"

static const int kNone = static_cast<int>(ImageKind::None);
static const int kPhoto = static_cast<int>(ImageKind::Photo);
static const int kSticker = static_cast<int>(ImageKind::Sticker);

static int Kind(const std::string& p_FileType, const std::string& p_FilePath)
{
  return static_cast<int>(ImageGeom::DetectKind(p_FileType, p_FilePath));
}

static std::string PngHeader(uint32_t p_Width, uint32_t p_Height)
{
  std::string s("\x89PNG\r\n\x1a\n", 8);
  s += std::string("\0\0\0\x0d", 4);
  s += "IHDR";
  for (uint32_t v : { p_Width, p_Height })
  {
    s += static_cast<char>((v >> 24) & 0xFF);
    s += static_cast<char>((v >> 16) & 0xFF);
    s += static_cast<char>((v >> 8) & 0xFF);
    s += static_cast<char>(v & 0xFF);
  }

  return s;
}

static void TestDetectKindByFileType()
{
  unittest::ExpectEqual(int, Kind("image/jpeg", "/x/a.jpg"), kPhoto);
  unittest::ExpectEqual(int, Kind("image/png", "/x/a.png"), kPhoto);
  unittest::ExpectEqual(int, Kind("image/webp", "/x/a.webp"), kSticker);
  unittest::ExpectEqual(int, Kind("application/x-tgsticker", "/x/a.tgs"), kNone);
  unittest::ExpectEqual(int, Kind("video/webm", "[Sticker]"), kNone);
  unittest::ExpectEqual(int, Kind("application/pdf", "/x/a.jpg"), kNone);
}

static void TestDetectKindFallback()
{
  unittest::ExpectEqual(int, Kind("", "/x/a.JPG"), kPhoto);
  unittest::ExpectEqual(int, Kind("", "/x/a.jpeg"), kPhoto);
  unittest::ExpectEqual(int, Kind("", "/x/a.png"), kPhoto);
  unittest::ExpectEqual(int, Kind("", "/x/a.webp"), kSticker);
  unittest::ExpectEqual(int, Kind("", "/x/a.tgs"), kNone);
  unittest::ExpectEqual(int, Kind("", "/x/a.webm"), kNone);
  unittest::ExpectEqual(int, Kind("", "[Photo]"), kPhoto);
  unittest::ExpectEqual(int, Kind("", "[Sticker]"), kSticker);
  unittest::ExpectEqual(int, Kind("", "[File]"), kNone);
  unittest::ExpectEqual(int, Kind("", ""), kNone);
  unittest::ExpectEqual(int, Kind("", "/x/noext"), kNone);
  unittest::ExpectEqual(int, Kind("", "/x.d/noext"), kNone);
}

static void TestParsePngSize()
{
  int w = 0;
  int h = 0;
  unittest::ExpectTrue(ImageGeom::ParsePngSize(PngHeader(640, 480), w, h));
  unittest::ExpectEqual(int, w, 640);
  unittest::ExpectEqual(int, h, 480);

  const std::string good = PngHeader(512, 256);
  unittest::ExpectFalse(ImageGeom::ParsePngSize(good.substr(0, 23), w, h));
  unittest::ExpectFalse(ImageGeom::ParsePngSize("", w, h));

  std::string badSig = good;
  badSig[1] = 'X';
  unittest::ExpectFalse(ImageGeom::ParsePngSize(badSig, w, h));

  unittest::ExpectFalse(ImageGeom::ParsePngSize(PngHeader(0, 10), w, h));
  unittest::ExpectFalse(ImageGeom::ParsePngSize(PngHeader(10, 0), w, h));
}

static void TestFitToCells()
{
  // square image in 1:2 cells: 20 cols x 10 rows is square on screen
  CellGrid g = ImageGeom::FitToCells(1000, 1000, 10, 20, 80, 10);
  unittest::ExpectEqual(int, g.cols, 20);
  unittest::ExpectEqual(int, g.rows, 10);

  // wide image fills the width up to the row limit
  g = ImageGeom::FitToCells(2000, 500, 10, 20, 80, 10);
  unittest::ExpectEqual(int, g.cols, 80);
  unittest::ExpectEqual(int, g.rows, 10);

  // narrow history area limits the columns
  g = ImageGeom::FitToCells(2000, 500, 10, 20, 30, 10);
  unittest::ExpectEqual(int, g.cols, 30);
  unittest::ExpectEqual(int, g.rows, 4);

  // unknown cell pixel size assumes 1:2
  g = ImageGeom::FitToCells(1000, 1000, 0, 0, 80, 10);
  unittest::ExpectEqual(int, g.cols, 20);
  unittest::ExpectEqual(int, g.rows, 10);

  // small images are scaled up to the limit
  g = ImageGeom::FitToCells(100, 100, 10, 20, 80, 10);
  unittest::ExpectEqual(int, g.cols, 20);
  unittest::ExpectEqual(int, g.rows, 10);

  // never below one cell
  g = ImageGeom::FitToCells(1, 1000, 10, 20, 80, 10);
  unittest::ExpectEqual(int, g.cols, 1);
  unittest::ExpectEqual(int, g.rows, 10);

  // invalid input
  g = ImageGeom::FitToCells(0, 100, 10, 20, 80, 10);
  unittest::ExpectEqual(int, g.cols, 0);
  unittest::ExpectEqual(int, g.rows, 0);
  g = ImageGeom::FitToCells(100, 100, 10, 20, 80, 0);
  unittest::ExpectEqual(int, g.cols, 0);
  unittest::ExpectEqual(int, g.rows, 0);
}

int main()
{
  int rv = 0;

  try
  {
    TestDetectKindByFileType();
    TestDetectKindFallback();
    TestParsePngSize();
    TestFitToCells();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
