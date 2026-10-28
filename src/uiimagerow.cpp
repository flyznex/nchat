// uiimagerow.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "uiimagerow.h"

#include <algorithm>
#include <vector>

#include "blockimage.h"
#include "imagecache.h"
#include "kittyencoder.h"
#include "termgfx.h"
#include "uicolorconfig.h"

void UiImageRow::Draw(WINDOW* p_Win, int p_Y, int p_X, int p_Row, int p_Id, int p_Cols)
{
  if (TermGfx::IsBlocks())
  {
    const BlockGrid* grid = ImageCache::GetBlocks(p_Id);
    if ((grid == nullptr) || (p_Row < 0) || (p_Row >= grid->rows)) return;

    const int cols = std::min(p_Cols, grid->cols);
    if (cols <= 0) return;

    std::vector<cchar_t> blocks(cols);
    for (int col = 0; col < cols; ++col)
    {
      const BlockPaint& paint = grid->At(p_Row, col);
      const wchar_t chars[2] = { paint.ch, L'\0' };
      const int pair = UiColorConfig::GetBlockPair(paint.fg, paint.bg);
      setcchar(&blocks[col], chars, A_NORMAL, static_cast<short>(pair), nullptr);
    }

    mvwadd_wchnstr(p_Win, p_Y, p_X, blocks.data(), cols);
    return;
  }

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
