// test006.cpp - key to action resolver for list/history focus
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <iostream>

#include "navkeys.h"

#include "unittest.h"

static const NavKeys keys = { 1001, 1002, 1003, 1004, 10, 9, 1005, 27 };

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
  unittest::ExpectTrue(R(List, L'n', true) == NavAction::None);
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
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
