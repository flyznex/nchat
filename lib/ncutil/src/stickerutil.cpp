// stickerutil.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "stickerutil.h"

#include <algorithm>
#include <set>

#include "numutil.h"

std::vector<size_t> StickerUtil::DedupeCap(const std::vector<int64_t>& p_Ids, size_t p_Max)
{
  std::vector<size_t> keep;
  std::set<int64_t> seen;
  for (size_t i = 0; (i < p_Ids.size()) && (keep.size() < p_Max); ++i)
  {
    if (seen.insert(p_Ids[i]).second) keep.push_back(i);
  }

  return keep;
}

int StickerUtil::Columns(int p_Width, int p_SlotWidth)
{
  if (p_SlotWidth <= 0) return 1;
  return std::max(1, p_Width / p_SlotWidth);
}

int StickerUtil::Move(int p_Index, int p_Dx, int p_Dy, int p_Cols, int p_Count)
{
  if ((p_Count <= 0) || (p_Cols <= 0)) return 0;

  const int index = NumUtil::Bound(0, p_Index, p_Count - 1);
  if (p_Dx != 0) return NumUtil::Bound(0, index + p_Dx, p_Count - 1);

  if (p_Dy > 0)
  {
    if ((index + p_Cols) < p_Count) return index + p_Cols;
    const int lastRow = (p_Count - 1) / p_Cols;
    return ((index / p_Cols) < lastRow) ? (p_Count - 1) : index;
  }

  if ((p_Dy < 0) && (index >= p_Cols)) return index - p_Cols;
  return index;
}

int StickerUtil::ScrollRow(int p_SelectedRow, int p_VisibleRows, int p_TotalRows, int p_CurrentRow)
{
  const int visible = std::max(1, p_VisibleRows);
  int top = p_CurrentRow;
  if (p_SelectedRow < top) top = p_SelectedRow;
  else if (p_SelectedRow >= (top + visible)) top = p_SelectedRow - visible + 1;
  return NumUtil::Bound(0, top, std::max(0, p_TotalRows - visible));
}

int StickerUtil::CycleTab(int p_Index, int p_Delta, int p_Count)
{
  if (p_Count <= 0) return 0;
  return (((p_Index + p_Delta) % p_Count) + p_Count) % p_Count;
}
