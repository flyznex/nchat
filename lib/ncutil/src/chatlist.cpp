// chatlist.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "chatlist.h"

#include <algorithm>
#include <cstdlib>

#include "strutil.h"

std::vector<ListRow> ChatList::BuildRows(const std::vector<ListChat>& p_Chats, const ListFilter& p_Filter,
                                         const std::set<std::string>& p_Collapsed)
{
  const bool hasText = !p_Filter.text.empty();
  const std::wstring needle = StrUtil::ToLower(StrUtil::ToWString(p_Filter.text));

  std::vector<int> visible;
  std::set<std::string> groups;
  for (int i = 0; i < static_cast<int>(p_Chats.size()); ++i)
  {
    const ListChat& chat = p_Chats[i];
    if (!p_Filter.folder.empty() && !chat.folders.count(p_Filter.folder)) continue;
    if (p_Filter.unreadOnly && !chat.unread) continue;
    if (hasText && (StrUtil::ToLower(StrUtil::ToWString(chat.name)).find(needle) == std::wstring::npos)) continue;
    visible.push_back(i);
    groups.insert(chat.group);
  }

  const bool grouped = !hasText && (groups.size() >= 2);
  std::vector<ListRow> rows;
  std::string lastGroup;
  bool first = true;
  for (int i : visible)
  {
    const std::string& group = p_Chats[i].group;
    if (grouped)
    {
      if (first || (group != lastGroup))
      {
        rows.push_back(ListRow{ -1, group });
        lastGroup = group;
        first = false;
      }

      if (p_Collapsed.count(group) > 0) continue;
    }

    rows.push_back(ListRow{ i, std::string() });
  }

  return rows;
}

static int Step(const std::vector<ListRow>& p_Rows, int p_Cur, int p_Dir)
{
  int found = p_Cur;
  if (p_Dir > 0)
  {
    for (const ListRow& row : p_Rows)
    {
      if ((row.chat >= 0) && (row.chat > p_Cur)) return row.chat;
    }
  }
  else
  {
    for (const ListRow& row : p_Rows)
    {
      if ((row.chat >= 0) && (row.chat < p_Cur)) found = row.chat;
    }
  }

  return found;
}

int ChatList::Move(const std::vector<ListRow>& p_Rows, int p_CurChat, int p_Delta)
{
  int cur = p_CurChat;
  const int dir = (p_Delta > 0) ? 1 : -1;
  for (int n = std::abs(p_Delta); n > 0; --n)
  {
    cur = Step(p_Rows, cur, dir);
  }

  return cur;
}

int ChatList::Nearest(const std::vector<ListRow>& p_Rows, int p_Chat)
{
  int prev = -1;
  for (const ListRow& row : p_Rows)
  {
    if (row.chat < 0) continue;
    if (row.chat >= p_Chat) return row.chat;
    prev = row.chat;
  }

  return prev;
}

int ChatList::Edge(const std::vector<ListRow>& p_Rows, bool p_Last)
{
  int edge = -1;
  for (const ListRow& row : p_Rows)
  {
    if (row.chat < 0) continue;
    edge = row.chat;
    if (!p_Last) break;
  }

  return edge;
}

std::string ChatList::CycleFolder(const std::vector<std::string>& p_Names, const std::string& p_Current, int p_Delta)
{
  if (p_Names.empty()) return "";

  const int count = static_cast<int>(p_Names.size()) + 1; // slot 0 is All
  int index = 0;
  for (int i = 0; i < static_cast<int>(p_Names.size()); ++i)
  {
    if (p_Names[i] == p_Current) index = i + 1;
  }

  index = (((index + p_Delta) % count) + count) % count;
  return (index == 0) ? std::string() : p_Names[index - 1];
}

int ChatList::FirstTab(const std::vector<int>& p_Widths, int p_Selected, int p_Width, int p_Gap)
{
  if (p_Widths.empty()) return 0;

  const int last = static_cast<int>(p_Widths.size()) - 1;
  int first = std::max(0, std::min(p_Selected, last));
  int used = p_Widths[first];
  while ((first > 0) && ((used + p_Gap + p_Widths[first - 1]) <= p_Width))
  {
    --first;
    used += p_Gap + p_Widths[first];
  }

  return first;
}
