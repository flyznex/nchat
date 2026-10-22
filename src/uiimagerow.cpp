// uiimagerow.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "uiimagerow.h"

#include <vector>

#include "kittyencoder.h"
#include "uicolorconfig.h"

void UiImageRow::Draw(WINDOW* p_Win, int p_Y, int p_X, int p_Row, int p_Id, int p_Cols)
{
  const int pair = UiColorConfig::GetImageIdPair(p_Id);
  std::vector<cchar_t> cells(p_Cols);
  for (int col = 0; col < p_Cols; ++col)
  {
    wchar_t chars[4];
    KittyEncoder::CellChars(p_Row, col, chars);
    setcchar(&cells[col], chars, A_NORMAL, static_cast<short>(pair), nullptr);
  }

  mvwadd_wchnstr(p_Win, p_Y, p_X, cells.data(), p_Cols);
}
