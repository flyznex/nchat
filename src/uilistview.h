// uilistview.h
//
// Copyright (c) 2019-2021 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <string>
#include <vector>

#include <ncurses.h>

#include "chatlist.h"
#include "uiviewbase.h"

class UiListView : public UiViewBase
{
public:
  UiListView(const UiViewParams& p_Params);
  virtual ~UiListView();

  virtual void Draw();
  bool RowAt(int p_ScreenY, ListRow& p_Row) const;
  void Scroll(int p_Rows);

private:
  std::string m_FilterKey;
  bool m_FreeScroll = false; // view scrolled by wheel, selection need not be visible
  int m_FreeIndex = -1;
  int m_PendingScroll = 0;
  std::vector<ListRow> m_Hits; // per padded-window line, chat == -2 means not a row
  WINDOW* m_PaddedWin = nullptr;
  int m_PaddedH = 0;
  int m_PaddedW = 0;
  int m_First = 0; // first visible row, kept between draws so the list scrolls only when the selection leaves it
};
