// uikeyinput.cpp
//
// Copyright (c) 2023 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "uikeyinput.h"

#include <ncurses.h>

#include "uikeyconfig.h"

static MEVENT s_Mouse;
static bool s_MouseValid = false;

int UiKeyInput::GetWch(wint_t* p_Wch)
{
  int rv = get_wch(p_Wch);
  if (rv == KEY_CODE_YES)
  {
    if (*p_Wch == KEY_MOUSE)
    {
      // always drain the ncurses mouse queue, one getmouse() per KEY_MOUSE
      MEVENT ev;
      s_MouseValid = (getmouse(&ev) == OK);
      if (s_MouseValid)
      {
        s_Mouse = ev;
      }
    }

    *p_Wch = UiKeyConfig::GetOffsettedKeyCode(*p_Wch, true);
  }

  return rv;
}

bool UiKeyInput::GetLastMouse(int& p_X, int& p_Y, unsigned long& p_State)
{
  if (!s_MouseValid) return false;

  p_X = s_Mouse.x;
  p_Y = s_Mouse.y;
  p_State = s_Mouse.bstate;
  return true;
}
