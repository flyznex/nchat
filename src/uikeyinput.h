// uikeyinput.h
//
// Copyright (c) 2023 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <ncurses.h>

// BUTTON5_PRESSED (wheel down) only exists with NCURSES_MOUSE_VERSION > 1, 0 when unsupported.
static const unsigned long kMouseClick = BUTTON1_PRESSED;
static const unsigned long kMouseWheelUp = BUTTON4_PRESSED;
#if defined(NCURSES_MOUSE_VERSION) && (NCURSES_MOUSE_VERSION > 1)
static const unsigned long kMouseWheelDown = BUTTON5_PRESSED;
#else
static const unsigned long kMouseWheelDown = 0;
#endif

class UiKeyInput
{
public:
  static int GetWch(wint_t* p_Wch);
  static bool GetLastMouse(int& p_X, int& p_Y, unsigned long& p_State);
};
