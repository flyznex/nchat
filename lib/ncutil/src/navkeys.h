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
  FocusNext, FocusPrev, FocusEntry, FocusList, FocusHistory, FocusLeft, FocusRight,
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

struct NavHint
{
  std::vector<wint_t> keys; // keys still to press
  NavAction action; // None when the keys only lead to a longer sequence
};

const char* NavActionName(NavAction p_Action);

class NavKeyMap
{
public:
  static const wint_t Pane = 0x7fffff00; // <Pane> token, matches NavKeys::panePrefix

  NavKeyMap();
  // Parses nav.conf text, returns one message per rejected line.
  std::vector<std::string> Load(const std::string& p_Text, bool p_VimOnly = false);
  const NavBinding* Find(NavFocus p_Focus, const std::vector<wint_t>& p_Seq, const NavKeys& p_Keys, bool p_Vim,
                         bool& p_Prefix) const;
  // What can follow the pending sequence p_Seq in this focus.
  std::vector<NavHint> Continuations(NavFocus p_Focus, const std::vector<wint_t>& p_Seq, const NavKeys& p_Keys,
                                     bool p_Vim) const;
  // Keys of the last-defined active binding for the action, false if unbound in that focus.
  bool FirstKeys(NavFocus p_Focus, NavAction p_Action, bool p_Vim, std::vector<wint_t>& p_Keys) const;

private:
  std::vector<NavBinding> m_Bindings;
};

class NavKeyResolver
{
public:
  NavAction Resolve(NavFocus p_Focus, wint_t p_Key, const NavKeys& p_Keys, bool p_Vim);
  // Hints for the keys pending now; FocusLeft/FocusRight hints without a target from p_Focus are dropped.
  std::vector<NavHint> Continuations(NavFocus p_Focus, const NavKeys& p_Keys, bool p_Vim) const;
  void SetListRight(bool p_ListRight) { m_ListRight = p_ListRight; }
  NavKeyMap& Map() { return m_Map; }
  const std::vector<wint_t>& PendingKeys() const { return m_Seq; }

private:
  NavKeyMap m_Map;
  std::vector<wint_t> m_Seq;
  bool m_ListRight = false;
};
