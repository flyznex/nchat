// uiview.cpp
//
// Copyright (c) 2019-2023 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "uiview.h"

#include "log.h"
#include "uiconfig.h"
#include "uientryview.h"
#include "uihelpview.h"
#include "uihistoryview.h"
#include "uikeyconfig.h"
#include "uilistborderview.h"
#include "uilistview.h"
#include "uiscreen.h"
#include "uistatusview.h"
#include "uitopview.h"
#include "uiviewbase.h"

UiView::UiView(UiModel* p_UiModel)
  : m_UiModel(p_UiModel)
{
  m_EmojiEnabled = UiConfig::GetBool("emoji_enabled");
  m_HelpEnabled = UiConfig::GetBool("help_enabled");
  m_ListEnabled = UiConfig::GetBool("list_enabled");
  m_StatusEnabled = UiConfig::GetBool("status_enabled");
  m_TopEnabled = UiConfig::GetBool("top_enabled");
  m_ListWidth = UiConfig::GetNum("list_width");
  m_Sidebar = (UiConfig::GetStr("layout_mode") != "classic");
  m_ListRight = (UiConfig::GetStr("list_position") == "right");
  m_EntryHeight = std::max(1, UiConfig::GetNum("entry_height"));
}

UiView::~UiView()
{
  UiConfig::SetBool("emoji_enabled", m_EmojiEnabled);
  UiConfig::SetBool("help_enabled", m_HelpEnabled);
  UiConfig::SetBool("list_enabled", m_ListEnabled);
  UiConfig::SetBool("top_enabled", m_TopEnabled);
  UiConfig::SetNum("list_width", m_ListWidth);
}

void UiView::Init()
{
  m_UiScreen = std::make_shared<UiScreen>();

  LayoutIn in;
  in.termW = m_UiScreen->W();
  in.termH = m_UiScreen->H();
  in.top = m_TopEnabled;
  in.help = m_HelpEnabled;
  in.status = m_StatusEnabled;
  in.list = m_ListEnabled;
  in.history = m_HistoryEnabled;
  in.entry = m_EntryEnabled;
  in.listWidth = m_ListWidth;
  in.sidebar = m_Sidebar;
  in.listRight = m_ListRight;
  in.entryMin = m_EntryHeight;
  in.entryLines = m_EntryLines;
  const LayoutOut lo = ComputeLayout(in);
  m_Layout = lo;

  auto params = [this](const LayoutRect& r, bool enabled) { return UiViewParams(r.x, r.y, r.w, r.h, enabled, m_UiModel); };
  m_UiTopView = std::make_shared<UiTopView>(params(lo.top, m_TopEnabled));
  m_UiHelpView = std::make_shared<UiHelpView>(params(lo.help, m_HelpEnabled));
  m_UiEntryView = std::make_shared<UiEntryView>(params(lo.entry, m_EntryEnabled));
  m_UiStatusView = std::make_shared<UiStatusView>(params(lo.status, m_StatusEnabled));
  m_UiListView = std::make_shared<UiListView>(params(lo.list, m_ListEnabled));
  m_UiListBorderView = std::make_shared<UiListBorderView>(params(lo.border, m_ListEnabled && (m_ListWidth > 0)));
  m_UiHistoryView = std::make_shared<UiHistoryView>(params(lo.history, m_HistoryEnabled));
}

bool UiView::GetListRowAt(int p_ScreenY, ListRow& p_Row)
{
  return m_UiListView && m_UiListView->RowAt(p_ScreenY, p_Row);
}

void UiView::ScrollList(int p_Rows)
{
  if (m_UiListView) m_UiListView->Scroll(p_Rows);
}

int UiView::GetHistoryMsgAt(int p_ScreenY)
{
  return m_UiHistoryView ? m_UiHistoryView->MsgAt(p_ScreenY) : -1;
}

bool UiView::SetEntryLines(int p_Lines)
{
  const int termH = m_UiScreen ? m_UiScreen->H() : 24;
  const int before = EntryContentHeight(m_EntryHeight, m_EntryLines, termH);
  m_EntryLines = p_Lines;
  return EntryContentHeight(m_EntryHeight, m_EntryLines, termH) != before;
}

void UiView::Draw()
{
  m_UiTopView->Draw();
  m_UiHelpView->Draw();
  m_UiStatusView->Draw();
  m_UiListView->Draw();
  m_UiListBorderView->Draw();
  m_UiHistoryView->Draw();
  m_UiEntryView->Draw();
  curs_set(1);
}

void UiView::TerminalBell()
{
  LOG_DEBUG("bell");
  beep();
}

void UiView::SetEmojiEnabled(bool p_Enabled)
{
  m_EmojiEnabled = p_Enabled;
}

bool UiView::GetEmojiEnabled()
{
  return m_EmojiEnabled;
}

void UiView::SetTopEnabled(bool p_Enabled)
{
  m_TopEnabled = p_Enabled;
}

bool UiView::GetTopEnabled()
{
  return m_TopEnabled;
}

void UiView::SetHelpEnabled(bool p_Enabled)
{
  m_HelpEnabled = p_Enabled;
}

bool UiView::GetHelpEnabled()
{
  return m_HelpEnabled;
}

void UiView::SetListEnabled(bool p_Enabled)
{
  m_ListEnabled = p_Enabled;
}

bool UiView::GetListEnabled()
{
  return m_ListEnabled;
}

bool UiView::GetListRight()
{
  return m_ListRight;
}

void UiView::SetListDirty(bool p_Dirty)
{
  m_UiListView->SetDirty(p_Dirty);
}

void UiView::SetListBorderDirty(bool p_Dirty)
{
  m_UiListBorderView->SetDirty(p_Dirty);
}

void UiView::SetStatusDirty(bool p_Dirty)
{
  m_UiStatusView->SetDirty(p_Dirty);
}

void UiView::SetHistoryDirty(bool p_Dirty)
{
  m_UiHistoryView->SetDirty(p_Dirty);
}

void UiView::SetHelpDirty(bool p_Dirty)
{
  m_UiHelpView->SetDirty(p_Dirty);
}

void UiView::SetEntryDirty(bool p_Dirty)
{
  m_UiEntryView->SetDirty(p_Dirty);
}

int UiView::GetHistoryShowCount()
{
  return m_UiHistoryView->GetHistoryShowCount();
}

int UiView::GetHistoryLines()
{
  return m_UiHistoryView->H();
}

int UiView::GetEntryWidth()
{
  return m_UiEntryView->W();
}

int UiView::GetScreenWidth()
{
  return m_UiScreen->W();
}

int UiView::GetScreenHeight()
{
  return m_UiScreen->H();
}

void UiView::DecreaseListWidth()
{
  if (m_ListWidth > 0)
  {
    --m_ListWidth;
  }
}

void UiView::IncreaseListWidth()
{
  if (m_ListWidth < m_UiScreen->W())
  {
    ++m_ListWidth;
  }
}
