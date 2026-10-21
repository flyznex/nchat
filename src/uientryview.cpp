// uientryview.cpp
//
// Copyright (c) 2019-2025 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "uientryview.h"

#include <algorithm>

#include "strutil.h"
#include "uicolorconfig.h"
#include "uimodel.h"

UiEntryView::UiEntryView(const UiViewParams& p_Params)
  : UiViewBase(p_Params)
{
}

void UiEntryView::Draw()
{
  if (!m_Enabled) return;

  if (!m_Dirty)
  {
    wmove(m_Win, m_CursY, m_CursX);
    wrefresh(m_Win);
    return;
  }

  m_Dirty = false;

  curs_set(0);

  std::wstring input = m_Model->GetEntryStrLocked();
  const int inputPos = m_Model->GetEntryPosLocked();
  std::vector<std::wstring> lines;
  int cx = 0;
  int cy = 0;
  lines = StrUtil::WordWrap(input, m_W, false, false, false, 2, inputPos, cy, cx);

  static int colorPair = UiColorConfig::GetColorPair("entry_color");
  static int attribute = UiColorConfig::GetAttribute("entry_attr");
  static int focusColorPair = UiColorConfig::GetColorPair("focus_color");
  static int focusAttr = UiColorConfig::GetAttribute("focus_attr");
  static int dimAttr = UiColorConfig::GetAttribute("dim_attr");

  werase(m_Win);
  wbkgd(m_Win, attribute | colorPair | ' ');

  // row 0 is the title line, content starts at row 1
  const bool focused = (m_Model->GetFocusLocked() == NavFocus::Entry);
  const std::wstring title = m_Model->GetEntryTitleLocked(m_W);
  const int titleAttr = focused ? (focusAttr | focusColorPair) : (attribute | colorPair | dimAttr);
  wattron(m_Win, titleAttr);
  mvwaddwstr(m_Win, 0, 0, title.c_str());
  wattroff(m_Win, titleAttr);

  wattron(m_Win, attribute | colorPair);
  const int contentH = m_H - 1;
  int yoffs = (cy < (contentH - 1)) ? 0 : (cy - (contentH - 1));

  for (int i = 0; i < contentH; ++i)
  {
    if ((i + yoffs) < (int)lines.size())
    {
      std::wstring line = lines.at(i + yoffs);
      line.erase(std::remove(line.begin(), line.end(), EMOJI_PAD), line.end());
      mvwaddwstr(m_Win, i + 1, 0, line.c_str());
    }
  }

  wattroff(m_Win, attribute | colorPair);

  m_CursX = cx;
  m_CursY = (cy - yoffs) + 1;

  wmove(m_Win, m_CursY, m_CursX);
  wrefresh(m_Win);
}
