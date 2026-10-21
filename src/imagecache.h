// imagecache.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <string>

struct ImageInfo
{
  std::string pngPath;
  int pxWidth = 0;
  int pxHeight = 0;
};

class ImageCache
{
public:
  enum class State
  {
    Pending,
    Ready,
    Failed,
  };

  static State Request(const std::string& p_SrcPath, ImageInfo* p_Info);
  static bool TakeCompleted();
  static void BeginDrawPass();
  static int Acquire(const std::string& p_SrcPath, const ImageInfo& p_Info, int p_Cols, int p_Rows);
  static void InvalidateAll();
  static void Shutdown();
};
