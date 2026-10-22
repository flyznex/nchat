// uilistview.cpp
//
// Copyright (c) 2019-2025 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "uilistview.h"

#include <algorithm>
#include <ctime>

#include "strutil.h"
#include "uicolorconfig.h"
#include "uiconfig.h"
#include "uimodel.h"

UiListView::UiListView(const UiViewParams& p_Params)
  : UiViewBase(p_Params)
{
  if (m_Enabled)
  {
    int paddedY = m_Y + 1;
    int paddedX = m_X + 1;
    m_PaddedH = m_H - 2;
    m_PaddedW = m_W - 2;
    m_PaddedWin = newwin(m_PaddedH, m_PaddedW, paddedY, paddedX);

    static int colorPair = UiColorConfig::GetColorPair("list_color");
    static int attribute = UiColorConfig::GetAttribute("list_attr");

    werase(m_Win);
    wbkgd(m_Win, attribute | colorPair | ' ');
    wrefresh(m_Win);
  }
}

UiListView::~UiListView()
{
  if (m_PaddedWin != nullptr)
  {
    delwin(m_PaddedWin);
    m_PaddedWin = nullptr;
  }
}

static std::string FormatListTime(int64_t p_TimeSent)
{
  if (p_TimeSent <= 0) return "";
  time_t t = (time_t)(p_TimeSent / 1000);
  time_t now = time(NULL);
  struct tm tmSent;
  struct tm tmNow;
  localtime_r(&t, &tmSent);
  localtime_r(&now, &tmNow);
  char buf[16] = { 0 };
  const bool today = (tmSent.tm_year == tmNow.tm_year) && (tmSent.tm_yday == tmNow.tm_yday);
  strftime(buf, sizeof(buf), today ? "%H:%M" : "%m-%d", &tmSent);
  return buf;
}

// fit to exactly p_Width columns (wide-character aware), marking cut text with "..." like the history view
static std::wstring FitWidth(const std::wstring& p_Str, int p_Width)
{
  if ((StrUtil::WStringWidth(p_Str) > p_Width) && (p_Width > 3))
  {
    return StrUtil::TrimPadWString(StrUtil::TrimPadWString(p_Str, p_Width - 3) + L"...", p_Width);
  }

  return StrUtil::TrimPadWString(p_Str, p_Width);
}

void UiListView::Draw()
{
  if (!m_Enabled || !m_Dirty) return;
  m_Dirty = false;

  curs_set(0);

  if ((m_PaddedWin == nullptr) || (m_PaddedH <= 0) || (m_PaddedW <= 0)) return; // tiny terminal

  static int colorPair = UiColorConfig::GetColorPair("list_color");
  static int attribute = UiColorConfig::GetAttribute("list_attr");
  static int attributeSelected = UiColorConfig::GetAttribute("list_attr_selected");
  static int colorPairUnread = UiColorConfig::GetColorPair("list_color_unread");
  static int nameAttr = UiColorConfig::GetAttribute("list_name_attr");
  static int previewAttr = UiColorConfig::GetAttribute("list_preview_attr");
  static int previewColorPair = UiColorConfig::GetColorPair("list_preview_color");
  static int dimAttr = UiColorConfig::GetAttribute("dim_attr");
  static int focusColorPair = UiColorConfig::GetColorPair("focus_color");
  static int focusAttr = UiColorConfig::GetAttribute("focus_attr");
  static const std::string unreadIndicator = UiConfig::GetStr("unread_indicator");
  // ponytail: two-line rows need room; narrower lists or terminals fall back to the old one-line rows
  const bool twoLines = (m_W >= 20) && (COLS >= 60) && (m_PaddedH >= 4);
  const int rowLines = twoLines ? 2 : 1;

  const bool focused = (m_Model->GetFocusLocked() == NavFocus::List);
  const int base = attribute | (focused ? 0 : dimAttr);
  const bool emojiEnabled = m_Model->GetEmojiEnabledLocked();
  const std::vector<std::pair<std::string, std::string>>& chatVec = m_Model->GetChatVecLocked();
  const std::vector<ListRow> rows = m_Model->GetListRowsLocked();
  const int index = m_Model->GetCurrentChatIndexLocked();
  const ListFilter& filter = m_Model->GetListFilterLocked();
  const bool editing = m_Model->GetListFilterEditingLocked();

  werase(m_PaddedWin);
  wbkgd(m_PaddedWin, base | colorPair | ' ');
  wattron(m_PaddedWin, base | colorPair);

  const bool bar = (editing || !filter.text.empty() || filter.unreadOnly) && (m_PaddedH > 1);
  const int top = bar ? 1 : 0;
  const int height = m_PaddedH - top;
  if (bar)
  {
    std::string text = "/" + filter.text + (editing ? "_" : "") + (filter.unreadOnly ? " [unread]" : "");
    // keep the tail visible while typing a long filter
    std::wstring wtext = StrUtil::ToWString(text);
    while ((StrUtil::WStringWidth(wtext) > m_PaddedW) && (wtext.size() > 1)) wtext.erase(0, 1);
    std::wstring wbar = StrUtil::TrimPadWString(wtext, m_PaddedW);
    wattron(m_PaddedWin, focusAttr | focusColorPair);
    mvwaddnwstr(m_PaddedWin, 0, 0, wbar.c_str(), wbar.size());
    wattroff(m_PaddedWin, focusAttr | focusColorPair);
  }

  auto linesOf = [&](const ListRow& row) { return (row.chat < 0) ? 1 : rowLines; };
  int sel = -1;
  for (int i = 0; i < (int)rows.size(); ++i)
  {
    if (rows[i].chat == index) sel = i;
  }

  m_First = std::min(m_First, std::max(0, (int)rows.size() - 1));
  if (sel >= 0)
  {
    if (sel < m_First) m_First = sel;
    auto span = [&](int a, int b) { int n = 0; for (int i = a; i <= b; ++i) n += linesOf(rows[i]); return n; };
    while ((m_First < sel) && (span(m_First, sel) > height)) ++m_First;
  }

  int y = top;
  for (int i = m_First; (i < (int)rows.size()) && (y < m_PaddedH); ++i)
  {
    const ListRow& row = rows[i];
    if (row.chat < 0)
    {
      std::string header = m_Model->GetProfileDisplayNameLocked(row.header);
      if (header.empty()) header = row.header; // "Pinned" is not a profile
      std::wstring whead = FitWidth(StrUtil::ToWString(header), m_PaddedW);
      wattron(m_PaddedWin, A_BOLD);
      mvwaddnwstr(m_PaddedWin, y, 0, whead.c_str(), whead.size());
      wattroff(m_PaddedWin, A_BOLD);
      ++y;
      continue;
    }

    const std::string& profileId = chatVec.at(row.chat).first;
    const std::string& chatId = chatVec.at(row.chat).second;
    const bool selected = (row.chat == index);
    const bool unread = m_Model->GetChatIsUnreadLocked(profileId, chatId);

    std::string name = m_Model->GetContactListNameLocked(profileId, chatId, true /*p_AllowId*/, true /*p_AllowAlias*/);
    if (!emojiEnabled) name = StrUtil::Textize(name);

    std::string marks;
    if (m_Model->GetChatIsPinnedLocked(profileId, chatId)) marks += "^";
    if (m_Model->GetChatIsMutedLocked(profileId, chatId)) marks += "~";
    if (unread) marks += unreadIndicator;
    std::string right = twoLines ? FormatListTime(m_Model->GetLastMessageTimeLocked(profileId, chatId)) : "";
    if (!marks.empty()) right += (right.empty() ? "" : " ") + marks;
    // leading space doubles as the gap, so the selected bar has no hole between name and right part
    std::wstring wright = right.empty() ? L"" : (L" " + StrUtil::ToWString(right));
    if (StrUtil::WStringWidth(wright) >= m_PaddedW) wright = StrUtil::TrimPadWString(wright, m_PaddedW - 1);
    const int rightW = StrUtil::WStringWidth(wright);
    const int nameW = std::max(1, m_PaddedW - rightW);

    if (selected)
    {
      wattroff(m_PaddedWin, base);
      wattron(m_PaddedWin, attributeSelected | (focused ? 0 : dimAttr));
    }

    if (unread) wattron(m_PaddedWin, colorPairUnread);

    std::wstring wname = FitWidth(StrUtil::ToWString(name), nameW);
    wattron(m_PaddedWin, nameAttr);
    mvwaddnwstr(m_PaddedWin, y, 0, wname.c_str(), wname.size());
    wattroff(m_PaddedWin, nameAttr);
    if (rightW > 0)
    {
      mvwaddnwstr(m_PaddedWin, y, m_PaddedW - rightW, wright.c_str(), wright.size());
    }

    if (unread) wattron(m_PaddedWin, colorPair);

    if (twoLines && ((y + 1) < m_PaddedH))
    {
      std::string preview = m_Model->GetChatPreviewLocked(profileId, chatId);
      if (!emojiEnabled) preview = StrUtil::Textize(preview);
      std::wstring wpreview = FitWidth(StrUtil::ToWString(" " + preview), m_PaddedW);
      // selected rows keep the reverse bar; unfocused lists are already dim via base
      const int previewStyle = selected ? 0 : (previewAttr | previewColorPair);
      wattron(m_PaddedWin, previewStyle);
      mvwaddnwstr(m_PaddedWin, y + 1, 0, wpreview.c_str(), wpreview.size());
      wattroff(m_PaddedWin, previewStyle);
      if (!selected) wattron(m_PaddedWin, colorPair);
    }

    if (selected)
    {
      wattroff(m_PaddedWin, attributeSelected | (focused ? 0 : dimAttr));
      wattron(m_PaddedWin, base);
    }

    y += rowLines;
  }

  wattroff(m_PaddedWin, base | colorPair);
  wrefresh(m_PaddedWin);
}
