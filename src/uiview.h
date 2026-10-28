// uiview.h
//
// Copyright (c) 2019-2025 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <memory>

#include "chatlist.h"
#include "layout.h"

class UiEntryView;
class UiHelpView;
class UiHistoryView;
class UiListView;
class UiListBorderView;
class UiModel;
class UiScreen;
class UiStatusView;
class UiTopView;

class UiView
{
public:
  UiView(UiModel* p_UiModel);
  virtual ~UiView();

  void TerminalBell();
  void SetEmojiEnabled(bool p_Enabled);
  bool GetEmojiEnabled();
  void SetTopEnabled(bool p_Enabled);
  bool GetTopEnabled();
  void SetHelpEnabled(bool p_Enabled);
  bool GetHelpEnabled();
  void SetListEnabled(bool p_Enabled);
  bool GetListEnabled();
  bool GetListRight();
  void SetListDirty(bool p_Dirty);
  void SetListBorderDirty(bool p_Dirty);
  void SetStatusDirty(bool p_Dirty);
  void SetHistoryDirty(bool p_Dirty);
  void SetHelpDirty(bool p_Dirty);
  void SetEntryDirty(bool p_Dirty);
  int GetHistoryShowCount();
  int GetHistoryLines();
  int GetEntryWidth();
  int GetScreenWidth();
  int GetScreenHeight();
  void DecreaseListWidth();
  void IncreaseListWidth();
  bool SetEntryLines(int p_Lines);
  const LayoutOut& GetLayout() const { return m_Layout; }
  bool GetListRowAt(int p_ScreenY, ListRow& p_Row);
  int GetHistoryMsgAt(int p_ScreenY);

protected:
  void Init();
  void Draw();

private:
  UiModel* m_UiModel = nullptr;

  std::shared_ptr<UiScreen> m_UiScreen;

  std::shared_ptr<UiTopView> m_UiTopView;
  std::shared_ptr<UiHelpView> m_UiHelpView;
  std::shared_ptr<UiEntryView> m_UiEntryView;
  std::shared_ptr<UiStatusView> m_UiStatusView;
  std::shared_ptr<UiListView> m_UiListView;
  std::shared_ptr<UiListBorderView> m_UiListBorderView;
  std::shared_ptr<UiHistoryView> m_UiHistoryView;

  bool m_EmojiEnabled = true;
  bool m_TopEnabled = true;
  bool m_HelpEnabled = true;
  const bool m_EntryEnabled = true;
  bool m_StatusEnabled = true;
  bool m_ListEnabled = true;
  const bool m_HistoryEnabled = true;
  int m_ListWidth = 14;
  bool m_Sidebar = true;
  bool m_ListRight = false;
  int m_EntryHeight = 4;
  int m_EntryLines = 1;
  LayoutOut m_Layout;

  friend class UiModel;
};
