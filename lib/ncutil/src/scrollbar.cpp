// scrollbar.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "scrollbar.h"

#include <algorithm>

bool ScrollThumb(int p_Total, int p_First, int p_Visible, int p_Track, int& p_Pos, int& p_Len)
{
  if ((p_Track <= 0) || (p_Visible <= 0) || (p_Total <= p_Visible)) return false;

  const int first = std::max(0, std::min(p_First, p_Total - p_Visible));
  p_Len = std::max(1, std::min(p_Track, ((p_Track * p_Visible) + (p_Total / 2)) / p_Total));
  p_Pos = std::min(((p_Track * first) + (p_Total / 2)) / p_Total, p_Track - p_Len);
  return true;
}
