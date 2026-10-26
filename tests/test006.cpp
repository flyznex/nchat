// test006.cpp - key to action resolver for list/history focus
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <iostream>

#include "navkeys.h"

#include "unittest.h"

static const NavKeys keys = { 1001, 1002, 1003, 1004, 10, 9, 1005, 27, 23 };

#define R(focus, key, vim) r.Resolve(NavFocus::focus, key, keys, vim)

static void TestEntryFocus()
{
  NavKeyResolver r;
  unittest::ExpectTrue(R(Entry, L'j', true) == NavAction::None);
  unittest::ExpectTrue(R(Entry, 1001, true) == NavAction::None);
  unittest::ExpectTrue(R(Entry, 9, true) == NavAction::FocusNext);
  unittest::ExpectTrue(R(Entry, 1005, true) == NavAction::FocusPrev);
  unittest::ExpectTrue(R(Entry, 27, true) == NavAction::FocusEntry);
}

static void TestConfiguredKeys()
{
  NavKeyResolver r;
  unittest::ExpectTrue(R(List, 1001, false) == NavAction::Up);
  unittest::ExpectTrue(R(List, 1002, false) == NavAction::Down);
  unittest::ExpectTrue(R(List, 1003, false) == NavAction::First);
  unittest::ExpectTrue(R(List, 1004, false) == NavAction::Last);
  unittest::ExpectTrue(R(List, 10, false) == NavAction::Open);
  unittest::ExpectTrue(R(History, 10, false) == NavAction::Reply);
  unittest::ExpectTrue(R(List, L'j', false) == NavAction::None);
  unittest::ExpectTrue(R(History, L'r', false) == NavAction::None);
}

static void TestVimList()
{
  NavKeyResolver r;
  unittest::ExpectTrue(R(List, L'j', true) == NavAction::Down);
  unittest::ExpectTrue(R(List, L'k', true) == NavAction::Up);
  unittest::ExpectTrue(R(List, L'G', true) == NavAction::Last);
  unittest::ExpectTrue(R(List, 4, true) == NavAction::HalfDown);
  unittest::ExpectTrue(R(List, 21, true) == NavAction::HalfUp);
  unittest::ExpectTrue(R(List, L'l', true) == NavAction::Open);
  unittest::ExpectTrue(R(List, L'h', true) == NavAction::None);
  unittest::ExpectTrue(R(List, L'i', true) == NavAction::FocusEntry);
  unittest::ExpectTrue(R(List, L'/', true) == NavAction::Search);
  unittest::ExpectTrue(R(List, L'u', true) == NavAction::UnreadOnly);
  unittest::ExpectTrue(R(List, L'z', true) == NavAction::ToggleGroup);
  unittest::ExpectTrue(R(List, L'r', true) == NavAction::None);
  unittest::ExpectTrue(R(List, L'n', true) == NavAction::FolderNext);
  unittest::ExpectTrue(R(List, L'b', true) == NavAction::FolderPrev);
  unittest::ExpectTrue(R(List, L'n', false) == NavAction::None);
  unittest::ExpectTrue(R(History, L'b', true) == NavAction::None);
}

static void TestVimHistory()
{
  NavKeyResolver r;
  unittest::ExpectTrue(R(History, L'h', true) == NavAction::FocusList);
  unittest::ExpectTrue(R(History, L'l', true) == NavAction::None);
  unittest::ExpectTrue(R(History, L'r', true) == NavAction::Reply);
  unittest::ExpectTrue(R(History, L'e', true) == NavAction::Edit);
  unittest::ExpectTrue(R(History, L'y', true) == NavAction::Copy);
  unittest::ExpectTrue(R(History, L'x', true) == NavAction::Delete);
  unittest::ExpectTrue(R(History, L'n', true) == NavAction::FindNext);
  unittest::ExpectTrue(R(History, L'/', true) == NavAction::Search);
  unittest::ExpectTrue(R(History, L'u', true) == NavAction::None);
  unittest::ExpectTrue(R(History, L'z', true) == NavAction::None);
}

static void TestGg()
{
  NavKeyResolver r;
  unittest::ExpectTrue(R(List, L'g', true) == NavAction::Pending);
  unittest::ExpectTrue(R(List, L'g', true) == NavAction::First);
  // pending expires on the next key
  unittest::ExpectTrue(R(List, L'g', true) == NavAction::Pending);
  unittest::ExpectTrue(R(List, L'j', true) == NavAction::Down);
  unittest::ExpectTrue(R(List, L'g', true) == NavAction::Pending);
  unittest::ExpectTrue(R(List, L'x', true) == NavAction::None);
  unittest::ExpectTrue(R(List, L'g', true) == NavAction::Pending);
  // pending also expires when focus becomes entry
  unittest::ExpectTrue(R(Entry, L'a', true) == NavAction::None);
  unittest::ExpectTrue(R(List, L'g', true) == NavAction::Pending);
}

static void TestPaneKeys()
{
  NavKeyResolver r;
  // Ctrl-W prefix works from every focus, then h/j/k/l move on the list | history / entry layout
  unittest::ExpectTrue(R(Entry, 23, true) == NavAction::Pending);
  unittest::ExpectTrue(R(Entry, L'h', true) == NavAction::FocusList);
  unittest::ExpectTrue(R(Entry, 23, true) == NavAction::Pending);
  unittest::ExpectTrue(R(Entry, L'k', true) == NavAction::FocusHistory);
  unittest::ExpectTrue(R(History, 23, true) == NavAction::Pending);
  unittest::ExpectTrue(R(History, L'j', true) == NavAction::FocusEntry);
  unittest::ExpectTrue(R(History, 23, true) == NavAction::Pending);
  unittest::ExpectTrue(R(History, L'h', true) == NavAction::FocusList);
  unittest::ExpectTrue(R(List, 23, true) == NavAction::Pending);
  unittest::ExpectTrue(R(List, L'l', true) == NavAction::FocusHistory);
  // moves with no target are swallowed, not typed
  unittest::ExpectTrue(R(List, 23, true) == NavAction::Pending);
  unittest::ExpectTrue(R(List, L'h', true) == NavAction::Pending);
  unittest::ExpectTrue(R(Entry, 23, true) == NavAction::Pending);
  unittest::ExpectTrue(R(Entry, L'j', true) == NavAction::Pending);
  // unknown key and Esc after the prefix are swallowed, and the prefix expires
  unittest::ExpectTrue(R(Entry, 23, true) == NavAction::Pending);
  unittest::ExpectTrue(R(Entry, L'z', true) == NavAction::Pending);
  unittest::ExpectTrue(R(Entry, L'z', true) == NavAction::None);
  // Ctrl-W a opens the address book from every focus
  unittest::ExpectTrue(R(Entry, 23, true) == NavAction::Pending);
  unittest::ExpectTrue(R(Entry, L'a', true) == NavAction::SelectContact);
  unittest::ExpectTrue(R(List, 23, true) == NavAction::Pending);
  unittest::ExpectTrue(R(List, L'a', true) == NavAction::SelectContact);
  unittest::ExpectTrue(R(Entry, 23, true) == NavAction::Pending);
  unittest::ExpectTrue(R(Entry, 27, true) == NavAction::Pending);
  unittest::ExpectTrue(R(Entry, 27, true) == NavAction::FocusEntry);
  // without vim_keys Ctrl-W is not a prefix
  unittest::ExpectTrue(R(Entry, 23, false) == NavAction::None);
  unittest::ExpectTrue(R(Entry, L'h', false) == NavAction::None);
}

static void TestNavConf()
{
  NavKeyResolver r;
  const std::vector<std::string> errs = r.Map().Load(
    "# comment\n"
    "map list x delete\n"            // new key
    "map list,history <Space> reply\n"
    "map list j none\n"              // disable a default
    "unmap history r\n"              // remove a default
    "map list <Pane>x folderprev\n"  // custom pane sequence
    "unmap list z\n"                 // an exact single key wins over a longer sequence
    "map list zz first\n"
    "map entry <C-g> search\n");
  unittest::ExpectEqual(size_t, errs.size(), size_t(0));
  unittest::ExpectTrue(R(List, L'x', true) == NavAction::Delete);
  unittest::ExpectTrue(R(List, L' ', true) == NavAction::Reply);
  unittest::ExpectTrue(R(History, L' ', true) == NavAction::Reply);
  unittest::ExpectTrue(R(List, L'j', true) == NavAction::None);
  unittest::ExpectTrue(R(History, L'j', true) == NavAction::Down);
  unittest::ExpectTrue(R(History, L'r', true) == NavAction::None);
  unittest::ExpectTrue(R(List, 23, true) == NavAction::Pending);
  unittest::ExpectTrue(R(List, L'x', true) == NavAction::FolderPrev);
  unittest::ExpectTrue(R(List, L'z', true) == NavAction::Pending);
  unittest::ExpectTrue(R(List, L'z', true) == NavAction::First);
  unittest::ExpectTrue(R(Entry, 7, true) == NavAction::Search);
  // user maps work without vim_keys, defaults do not
  unittest::ExpectTrue(R(List, L'x', false) == NavAction::Delete);
  unittest::ExpectTrue(R(List, L'k', false) == NavAction::None);
}

static void TestFirstKeys()
{
  NavKeyResolver r;
  std::vector<wint_t> k;
  unittest::ExpectTrue(r.Map().FirstKeys(NavFocus::List, NavAction::Down, true, k));
  unittest::ExpectTrue((k == std::vector<wint_t>{ L'j' }));
  unittest::ExpectTrue(!r.Map().FirstKeys(NavFocus::List, NavAction::Down, false, k));
  unittest::ExpectTrue(!r.Map().FirstKeys(NavFocus::History, NavAction::Open, true, k));
  unittest::ExpectTrue(r.Map().FirstKeys(NavFocus::Entry, NavAction::SelectContact, true, k));
  unittest::ExpectTrue((k == std::vector<wint_t>{ NavKeyMap::Pane, L'a' }));
  // the last-defined binding wins, and user maps show without vim_keys
  r.Map().Load("map list <C-n> down\n");
  unittest::ExpectTrue(r.Map().FirstKeys(NavFocus::List, NavAction::Down, false, k));
  unittest::ExpectTrue((k == std::vector<wint_t>{ 14 }));
  r.Map().Load("unmap list <C-n>\nunmap list j\n");
  unittest::ExpectTrue(!r.Map().FirstKeys(NavFocus::List, NavAction::Down, true, k));
}

static void TestNavConfErrors()
{
  NavKeyResolver r;
  const std::vector<std::string> errs = r.Map().Load(
    "map list x\n"
    "map nowhere x delete\n"
    "map list <Bogus> delete\n"
    "map list x explode\n"
    "map list <lt> delete\n");
  unittest::ExpectEqual(size_t, errs.size(), size_t(4));
  unittest::ExpectTrue(R(List, L'<', false) == NavAction::Delete);
  // mapclear drops the defaults but keeps the configured keys
  r.Map().Load("mapclear\nmap list q open\n");
  unittest::ExpectTrue(R(List, L'j', true) == NavAction::None);
  unittest::ExpectTrue(R(List, L'q', true) == NavAction::Open);
  unittest::ExpectTrue(R(List, 1001, true) == NavAction::Up);
}

int main()
{
  int rv = 0;
  try
  {
    TestEntryFocus();
    TestConfiguredKeys();
    TestVimList();
    TestVimHistory();
    TestGg();
    TestPaneKeys();
    TestNavConf();
    TestFirstKeys();
    TestNavConfErrors();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
