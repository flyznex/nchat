// imagegeom.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "imagegeom.h"

#include <algorithm>
#include <cmath>
#include <cstdint>

#include "strutil.h"

ImageKind ImageGeom::DetectKind(const std::string& p_FileType, const std::string& p_FilePath)
{
  if (!p_FileType.empty())
  {
    if ((p_FileType == "image/jpeg") || (p_FileType == "image/png")) return ImageKind::Photo;
    if (p_FileType == "image/webp") return ImageKind::Sticker;
    return ImageKind::None;
  }

  const size_t slash = p_FilePath.rfind('/');
  const std::string name = (slash == std::string::npos) ? p_FilePath : p_FilePath.substr(slash + 1);
  if (name == "[Photo]") return ImageKind::Photo;
  if (name == "[Sticker]") return ImageKind::Sticker;

  const size_t dot = name.rfind('.');
  if (dot == std::string::npos) return ImageKind::None;

  const std::string ext = StrUtil::ToLower(name.substr(dot + 1));
  // only trust the extension for files TDLib stored as photo/sticker, not for arbitrary documents
  if (((ext == "jpg") || (ext == "jpeg") || (ext == "png")) && (p_FilePath.find("/photos/") != std::string::npos))
    return ImageKind::Photo;
  if ((ext == "webp") && (p_FilePath.find("/stickers/") != std::string::npos)) return ImageKind::Sticker;
  return ImageKind::None;
}

bool ImageGeom::ParsePngSize(const std::string& p_Bytes, int& p_Width, int& p_Height)
{
  static const std::string signature("\x89PNG\r\n\x1a\n", 8);
  if (p_Bytes.size() < 24) return false;
  if (p_Bytes.compare(0, 8, signature) != 0) return false;
  if (p_Bytes.compare(12, 4, "IHDR") != 0) return false;

  auto be32 = [&](size_t p_Pos) -> uint32_t
  {
    return (static_cast<uint32_t>(static_cast<unsigned char>(p_Bytes[p_Pos])) << 24) |
           (static_cast<uint32_t>(static_cast<unsigned char>(p_Bytes[p_Pos + 1])) << 16) |
           (static_cast<uint32_t>(static_cast<unsigned char>(p_Bytes[p_Pos + 2])) << 8) |
           static_cast<uint32_t>(static_cast<unsigned char>(p_Bytes[p_Pos + 3]));
  };

  const uint32_t w = be32(16);
  const uint32_t h = be32(20);
  if ((w == 0) || (h == 0) || (w > 100000) || (h > 100000)) return false;

  p_Width = static_cast<int>(w);
  p_Height = static_cast<int>(h);
  return true;
}

CellGrid ImageGeom::FitToCells(int p_PxWidth, int p_PxHeight, int p_CellPxWidth, int p_CellPxHeight,
                               int p_MaxCols, int p_MaxRows)
{
  CellGrid grid;
  if ((p_PxWidth <= 0) || (p_PxHeight <= 0) || (p_MaxCols <= 0) || (p_MaxRows <= 0)) return grid;

  if ((p_CellPxWidth <= 0) || (p_CellPxHeight <= 0))
  {
    p_CellPxWidth = 1;
    p_CellPxHeight = 2;
  }

  const double naturalCols = static_cast<double>(p_PxWidth) / p_CellPxWidth;
  const double naturalRows = static_cast<double>(p_PxHeight) / p_CellPxHeight;
  const double scale = std::min(p_MaxCols / naturalCols, p_MaxRows / naturalRows);

  grid.cols = std::max(1, std::min(p_MaxCols, static_cast<int>(std::lround(naturalCols * scale))));
  grid.rows = std::max(1, std::min(p_MaxRows, static_cast<int>(std::lround(naturalRows * scale))));
  return grid;
}
