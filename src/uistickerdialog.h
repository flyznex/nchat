// uistickerdialog.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <cstdint>
#include <string>
#include <vector>

#include "protocol.h"
#include "uidialog.h"

class UiStickerDialog : public UiDialog
{
public:
  UiStickerDialog(const UiDialogParams& p_Params, const std::string& p_ProfileId);
  virtual ~UiStickerDialog();

  bool Run();
  int32_t GetSelectedFileId() const;

private:
  void KeyHandler(wint_t p_Key);
  bool OnTimer();
  void UpdateLayout();
  void Draw();
  void DrawSlot(int p_Index, int p_X, int p_Y, bool p_Selected);

  std::string m_ProfileId;
  std::vector<StickerInfo> m_Stickers;
  bool m_Loaded = false;
  bool m_Running = true;
  bool m_Result = false;
  int32_t m_SelectedFileId = 0;
  int m_Index = 0;
  int m_TopRow = 0;
  int m_ImgCols = 1;
  int m_ImgRows = 1;
  int m_Cols = 1;
  int m_VisibleRows = 1;
};
