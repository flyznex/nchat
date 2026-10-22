// uiimagerow.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <ncurses.h>

class UiImageRow
{
public:
  static void Draw(WINDOW* p_Win, int p_Y, int p_X, int p_Row, int p_Id, int p_Cols);
};
