// uilistborderview.cpp
//
// Copyright (c) 2019-2021 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "uilistborderview.h"

#include <ncurses.h>

#include "uicolorconfig.h"
#include "uimodel.h"

UiListBorderView::UiListBorderView(const UiViewParams& p_Params)
  : UiViewBase(p_Params)
{
}

void UiListBorderView::Draw()
{
  if (!m_Enabled || !m_Dirty) return;
  m_Dirty = false;

  curs_set(0);

  const bool focused = (m_Model->GetFocusLocked() == NavFocus::List);
  static int colorPair = UiColorConfig::GetColorPair("listborder_color");
  static int attribute = UiColorConfig::GetAttribute("listborder_attr");
  static int focusColorPair = UiColorConfig::GetColorPair("focus_color");
  static int focusAttribute = UiColorConfig::GetAttribute("focus_attr");
  const int drawColor = focused ? focusColorPair : colorPair;
  const int drawAttr = focused ? focusAttribute : attribute;

  werase(m_Win);
  wbkgd(m_Win, drawAttr | drawColor | ' ');
  wattron(m_Win, drawAttr | drawColor);

  mvwvline(m_Win, 0, 0, ACS_VLINE, m_H);

  wattroff(m_Win, drawAttr | drawColor);
  wrefresh(m_Win);
}
