// navkeys.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <cwchar>
#include <string>
#include <vector>

enum class NavFocus { List, History, Entry };

enum class NavAction
{
  None, Pending,
  FocusNext, FocusPrev, FocusEntry, FocusList, FocusHistory,
  Up, Down, First, Last, HalfUp, HalfDown,
  Open, Reply, Edit, Copy, Delete,
  Search, FindNext, UnreadOnly, ToggleGroup, FolderNext, FolderPrev, SelectContact
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

struct NavBinding
{
  unsigned focusMask;
  std::vector<wint_t> keys;
  NavAction action;
  bool vimOnly;
};

class NavKeyMap
{
public:
  static const wint_t Pane = 0x7fffff00; // <Pane> token, matches NavKeys::panePrefix

  NavKeyMap();
  // Parses nav.conf text, returns one message per rejected line.
  std::vector<std::string> Load(const std::string& p_Text, bool p_VimOnly = false);
  const NavBinding* Find(NavFocus p_Focus, const std::vector<wint_t>& p_Seq, const NavKeys& p_Keys, bool p_Vim,
                         bool& p_Prefix) const;

private:
  std::vector<NavBinding> m_Bindings;
};

class NavKeyResolver
{
public:
  NavAction Resolve(NavFocus p_Focus, wint_t p_Key, const NavKeys& p_Keys, bool p_Vim);
  NavKeyMap& Map() { return m_Map; }

private:
  NavKeyMap m_Map;
  std::vector<wint_t> m_Seq;
};
