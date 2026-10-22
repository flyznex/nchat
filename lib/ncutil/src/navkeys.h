// navkeys.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <cwchar>

enum class NavFocus { List, History, Entry };

enum class NavAction
{
  None, Pending,
  FocusNext, FocusPrev, FocusEntry, FocusList, FocusHistory,
  Up, Down, First, Last, HalfUp, HalfDown,
  Open, Reply, Edit, Copy, Delete,
  Search, FindNext, UnreadOnly, ToggleGroup
};

struct NavKeys
{
  wint_t up;
  wint_t down;
  wint_t home;
  wint_t end;
  wint_t ok;
  wint_t focusNext;
  wint_t focusPrev;
  wint_t focusEntry;
  wint_t panePrefix;
};

class NavKeyResolver
{
public:
  NavAction Resolve(NavFocus p_Focus, wint_t p_Key, const NavKeys& p_Keys, bool p_Vim);

private:
  bool m_PendingG = false;
  bool m_PendingPane = false;
};
