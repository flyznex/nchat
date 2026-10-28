// chatlist.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <set>
#include <string>
#include <vector>

struct ListChat
{
  std::string name;
  std::string group;
  bool unread;
  std::set<std::string> folders; // names of the folders the chat belongs to
};

struct ListFilter
{
  std::string text;
  bool unreadOnly = false;
  std::string folder; // empty = All
};

struct ListRow
{
  int chat; // index into the input vector, or -1 for a group header
  std::string header;
};

class ChatList
{
public:
  static std::vector<ListRow> BuildRows(const std::vector<ListChat>& p_Chats, const ListFilter& p_Filter,
                                        const std::set<std::string>& p_Collapsed);
  static int Move(const std::vector<ListRow>& p_Rows, int p_CurChat, int p_Delta);
  static int Nearest(const std::vector<ListRow>& p_Rows, int p_Chat); // p_Chat if visible, else next, else previous, else -1
  static int Edge(const std::vector<ListRow>& p_Rows, bool p_Last);
  static std::string CycleFolder(const std::vector<std::string>& p_Names, const std::string& p_Current, int p_Delta);
  // first visible row; p_Lines = height of each row, p_Sel = selected row or -1, p_Free = wheel-scrolled, selection may be off screen
  static int ScrollFirst(const std::vector<int>& p_Lines, int p_First, int p_Sel, int p_Height, bool p_Free);
  static int FirstTab(const std::vector<int>& p_Widths, int p_Selected, int p_Width, int p_Gap);
};
