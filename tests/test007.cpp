// test007.cpp - chat list filter, grouping and navigation
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

#include "unittest.h"

static std::vector<ListChat> Chats()
{
  std::vector<ListChat> v;
  v.push_back(ListChat{ "Alice", "Pinned", false });
  v.push_back(ListChat{ "Bob", "Pinned", true });
  v.push_back(ListChat{ "Carol", "tg", false });
  v.push_back(ListChat{ "Dave", "tg", true });
  return v;
}

static std::vector<int> Ids(const std::vector<ListRow>& p_Rows)
{
  std::vector<int> ids;
  for (const ListRow& row : p_Rows) ids.push_back(row.chat);
  return ids;
}

static void ExpectIds(const std::vector<ListRow>& p_Rows, const std::vector<int>& p_Expected)
{
  unittest::ExpectEqual(int, static_cast<int>(Ids(p_Rows).size()), static_cast<int>(p_Expected.size()));
  for (size_t i = 0; (i < p_Rows.size()) && (i < p_Expected.size()); ++i)
  {
    unittest::ExpectEqual(int, p_Rows[i].chat, p_Expected[i]);
  }
}

static void TestGrouping()
{
  const std::set<std::string> none;
  ListFilter f;
  std::vector<ListRow> rows = ChatList::BuildRows(Chats(), f, none);
  ExpectIds(rows, { -1, 0, 1, -1, 2, 3 });
  unittest::ExpectEqual(std::string, rows[0].header, std::string("Pinned"));
  unittest::ExpectEqual(std::string, rows[3].header, std::string("tg"));

  std::vector<ListChat> one = Chats();
  for (ListChat& c : one) c.group = "tg";
  ExpectIds(ChatList::BuildRows(one, f, none), { 0, 1, 2, 3 });
}

static void TestFilters()
{
  const std::set<std::string> none;
  ListFilter f;
  f.text = "AL";
  ExpectIds(ChatList::BuildRows(Chats(), f, none), { 0 });

  ListFilter u;
  u.unreadOnly = true;
  ExpectIds(ChatList::BuildRows(Chats(), u, none), { -1, 1, -1, 3 });

  ListFilter none2;
  ExpectIds(ChatList::BuildRows(std::vector<ListChat>(), none2, none), {});
}

static void TestCollapse()
{
  std::set<std::string> collapsed;
  collapsed.insert("Pinned");
  ListFilter f;
  ExpectIds(ChatList::BuildRows(Chats(), f, collapsed), { -1, -1, 2, 3 });

  f.text = "bob";
  ExpectIds(ChatList::BuildRows(Chats(), f, collapsed), { 1 });
}

static void TestMove()
{
  const std::set<std::string> none;
  ListFilter f;
  std::vector<ListRow> rows = ChatList::BuildRows(Chats(), f, none);
  unittest::ExpectEqual(int, ChatList::Move(rows, 0, 1), 1);
  unittest::ExpectEqual(int, ChatList::Move(rows, 1, 1), 2);
  unittest::ExpectEqual(int, ChatList::Move(rows, 3, 1), 3);
  unittest::ExpectEqual(int, ChatList::Move(rows, 2, -1), 1);
  unittest::ExpectEqual(int, ChatList::Move(rows, 0, -1), 0);
  unittest::ExpectEqual(int, ChatList::Move(rows, 0, 3), 3);
  unittest::ExpectEqual(int, ChatList::Edge(rows, true), 3);
  unittest::ExpectEqual(int, ChatList::Edge(rows, false), 0);

  // current chat hidden by a collapsed group
  std::set<std::string> collapsed;
  collapsed.insert("Pinned");
  rows = ChatList::BuildRows(Chats(), f, collapsed);
  unittest::ExpectEqual(int, ChatList::Move(rows, 0, 1), 2);
  unittest::ExpectEqual(int, ChatList::Move(rows, 1, -1), 1);

  unittest::ExpectEqual(int, ChatList::Move(std::vector<ListRow>(), 5, 1), 5);
  unittest::ExpectEqual(int, ChatList::Edge(std::vector<ListRow>(), true), -1);
}

static void TestNearest()
{
  std::set<std::string> collapsed;
  collapsed.insert("Pinned");
  ListFilter f;
  std::vector<ListRow> rows = ChatList::BuildRows(Chats(), f, collapsed); // visible chats: 2, 3
  unittest::ExpectEqual(int, ChatList::Nearest(rows, 1), 2);  // hidden, next one
  unittest::ExpectEqual(int, ChatList::Nearest(rows, 3), 3);  // visible stays
  unittest::ExpectEqual(int, ChatList::Nearest(rows, 9), 3);  // none after, previous one
  unittest::ExpectEqual(int, ChatList::Nearest(std::vector<ListRow>(), 1), -1);
}

int main()
{
  int rv = 0;
  try
  {
    TestGrouping();
    TestFilters();
    TestCollapse();
    TestMove();
    TestNearest();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
