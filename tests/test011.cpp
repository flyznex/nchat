// test011.cpp - chat folder filter, folder cycling and tab layout
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <iostream>
#include <set>
#include <string>
#include <vector>

#include "chatlist.h"
#include "scrollbar.h"

#include "unittest.h"

static ListChat C(const std::string& p_Name, const std::set<std::string>& p_Folders, bool p_Unread = false)
{
  ListChat chat;
  chat.name = p_Name;
  chat.group = "tg";
  chat.unread = p_Unread;
  chat.folders = p_Folders;
  return chat;
}

static std::vector<ListChat> Chats()
{
  std::vector<ListChat> v;
  v.push_back(C("Alice", { "Work" }));
  v.push_back(C("Bob", { "Work", "Family" }));
  v.push_back(C("Carol", { }));
  v.push_back(C("Dave", { "Family" }, true));
  return v;
}

static void ExpectIds(const std::vector<ListRow>& p_Rows, const std::vector<int>& p_Expected)
{
  unittest::ExpectEqual(int, static_cast<int>(p_Rows.size()), static_cast<int>(p_Expected.size()));
  for (size_t i = 0; (i < p_Rows.size()) && (i < p_Expected.size()); ++i)
  {
    unittest::ExpectEqual(int, p_Rows[i].chat, p_Expected[i]);
  }
}

static void TestFolderFilter()
{
  const std::set<std::string> none;
  ListFilter f;
  ExpectIds(ChatList::BuildRows(Chats(), f, none), { 0, 1, 2, 3 });

  f.folder = "Work";
  ExpectIds(ChatList::BuildRows(Chats(), f, none), { 0, 1 });

  f.folder = "Family";
  ExpectIds(ChatList::BuildRows(Chats(), f, none), { 1, 3 });

  f.folder = "Nope";
  ExpectIds(ChatList::BuildRows(Chats(), f, none), { });

  f.folder = "Work";
  f.unreadOnly = true;
  ExpectIds(ChatList::BuildRows(Chats(), f, none), { });

  f.folder = "Family";
  ExpectIds(ChatList::BuildRows(Chats(), f, none), { 3 });

  f.folder = "Work";
  f.unreadOnly = false;
  f.text = "bo";
  ExpectIds(ChatList::BuildRows(Chats(), f, none), { 1 });
}

static void TestCycleFolder()
{
  const std::vector<std::string> names = { "Work", "Family" };
  unittest::ExpectEqual(std::string, ChatList::CycleFolder(names, "", 1), std::string("Work"));
  unittest::ExpectEqual(std::string, ChatList::CycleFolder(names, "Work", 1), std::string("Family"));
  unittest::ExpectEqual(std::string, ChatList::CycleFolder(names, "Family", 1), std::string(""));
  unittest::ExpectEqual(std::string, ChatList::CycleFolder(names, "", -1), std::string("Family"));
  unittest::ExpectEqual(std::string, ChatList::CycleFolder(names, "Work", -1), std::string(""));
  unittest::ExpectEqual(std::string, ChatList::CycleFolder(names, "Gone", 1), std::string("Work"));
  unittest::ExpectEqual(std::string, ChatList::CycleFolder({ }, "Work", 1), std::string(""));
}

static void TestFirstTab()
{
  const std::vector<int> widths = { 3, 4, 6 }; // All, Work, Family
  unittest::ExpectEqual(int, ChatList::FirstTab(widths, 2, 20, 2), 0);
  unittest::ExpectEqual(int, ChatList::FirstTab(widths, 2, 12, 2), 1);
  unittest::ExpectEqual(int, ChatList::FirstTab(widths, 2, 5, 2), 2);
  unittest::ExpectEqual(int, ChatList::FirstTab(widths, 0, 5, 2), 0);
  unittest::ExpectEqual(int, ChatList::FirstTab({ }, 0, 5, 2), 0);
}

static void TestScrollFirst()
{
  const std::vector<int> ten(10, 1); // ten one-line rows, window of 4
  // selection follows: below the window scrolls down, above scrolls up, inside leaves it
  unittest::ExpectEqual(int, ChatList::ScrollFirst(ten, 0, 2, 4, false), 0);
  unittest::ExpectEqual(int, ChatList::ScrollFirst(ten, 0, 7, 4, false), 4);
  unittest::ExpectEqual(int, ChatList::ScrollFirst(ten, 6, 2, 4, false), 2);
  // free scroll ignores the selection, but never leaves a gap below the last row
  unittest::ExpectEqual(int, ChatList::ScrollFirst(ten, 5, 0, 4, true), 5);
  unittest::ExpectEqual(int, ChatList::ScrollFirst(ten, 9, 0, 4, true), 6);
  unittest::ExpectEqual(int, ChatList::ScrollFirst(ten, -3, 9, 4, true), 0);
  // a stale first past the end of a shrunken list is clamped
  unittest::ExpectEqual(int, ChatList::ScrollFirst({ 1, 1, 1 }, 40, -1, 4, false), 2);
  unittest::ExpectEqual(int, ChatList::ScrollFirst({ }, 5, -1, 4, false), 0);
  // two-line rows: window of 4 lines holds two rows
  const std::vector<int> two(6, 2);
  unittest::ExpectEqual(int, ChatList::ScrollFirst(two, 0, 3, 4, false), 2);
  unittest::ExpectEqual(int, ChatList::ScrollFirst(two, 5, 0, 4, true), 4);
}

static void TestScrollThumb()
{
  int pos = -1;
  int len = -1;
  // everything fits, or there is no track or window: no bar
  unittest::ExpectFalse(ScrollThumb(4, 0, 4, 4, pos, len));
  unittest::ExpectFalse(ScrollThumb(3, 0, 4, 4, pos, len));
  unittest::ExpectFalse(ScrollThumb(10, 0, 4, 0, pos, len));
  unittest::ExpectFalse(ScrollThumb(10, 0, 0, 4, pos, len));
  // 10 rows, window 4, track 4: thumb of 2 from top to bottom
  unittest::ExpectTrue(ScrollThumb(10, 0, 4, 4, pos, len));
  unittest::ExpectEqual(int, len, 2);
  unittest::ExpectEqual(int, pos, 0);
  unittest::ExpectTrue(ScrollThumb(10, 3, 4, 4, pos, len));
  unittest::ExpectEqual(int, pos, 1);
  unittest::ExpectTrue(ScrollThumb(10, 6, 4, 4, pos, len));
  unittest::ExpectEqual(int, pos, 2);
  // a stale first past the end stays inside the track
  unittest::ExpectTrue(ScrollThumb(10, 50, 4, 4, pos, len));
  unittest::ExpectEqual(int, pos, 2);
  unittest::ExpectTrue(ScrollThumb(10, -5, 4, 4, pos, len));
  unittest::ExpectEqual(int, pos, 0);
  // thumb is at least one cell, and never leaves the track
  unittest::ExpectTrue(ScrollThumb(1000, 999, 1, 5, pos, len));
  unittest::ExpectEqual(int, len, 1);
  unittest::ExpectEqual(int, pos, 4);
  unittest::ExpectTrue(ScrollThumb(10, 6, 4, 1, pos, len));
  unittest::ExpectEqual(int, len, 1);
  unittest::ExpectEqual(int, pos, 0);
}

int main()
{
  int rv = 0;

  try
  {
    TestFolderFilter();
    TestCycleFolder();
    TestFirstTab();
    TestScrollFirst();
    TestScrollThumb();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
