// stickerutil.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <cstddef>
#include <cstdint>
#include <vector>

class StickerUtil
{
public:
  static std::vector<size_t> DedupeCap(const std::vector<int64_t>& p_Ids, size_t p_Max);
  static int Columns(int p_Width, int p_SlotWidth);
  static int Move(int p_Index, int p_Dx, int p_Dy, int p_Cols, int p_Count);
  static int ScrollRow(int p_SelectedRow, int p_VisibleRows, int p_TotalRows, int p_CurrentRow);
  static int CycleTab(int p_Index, int p_Delta, int p_Count);
};
