// uistickerdialog.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "uistickerdialog.h"

#include <algorithm>

#include "apputil.h"
#include "imagecache.h"
#include "imagegeom.h"
#include "kittyencoder.h"
#include "stickerutil.h"
#include "strutil.h"
#include "termgfx.h"
#include "timeutil.h"
#include "uicolorconfig.h"
#include "uiconfig.h"
#include "uicontroller.h"
#include "uiimagerow.h"
#include "uikeyconfig.h"
#include "uimodel.h"

UiStickerDialog::UiStickerDialog(const UiDialogParams& p_Params, const std::string& p_ProfileId)
  : UiDialog(p_Params)
  , m_ProfileId(p_ProfileId)
{
  m_Model->SetStickerDialogActive(true);
  m_Model->RequestStickers(m_ProfileId, "recent", 0, "");
  UpdateLayout();
}

UiStickerDialog::~UiStickerDialog()
{
  m_Model->SetStickerDialogActive(false);
}

int32_t UiStickerDialog::GetSelectedFileId() const
{
  return m_SelectedFileId;
}

bool UiStickerDialog::Run()
{
  m_Model->Draw();
  curs_set(0); // needed as UiView::Draw() sets curs_set(1)

  Draw();
  int64_t lastTimerEvent = 0;
  while (m_Running && !AppUtil::IsTerminateRequested())
  {
    wint_t key = UiController::GetKey(50);
    if (key != 0)
    {
      KeyHandler(key);
    }

    int64_t nowTime = TimeUtil::GetCurrentTimeMSec();
    if ((nowTime - lastTimerEvent) > 50)
    {
      lastTimerEvent = nowTime;
      if (OnTimer())
      {
        Draw();
      }
    }
  }

  return m_Result;
}

bool UiStickerDialog::OnTimer()
{
  // the main loop is not running while the dialog is open, so poll finished conversions here
  bool dirty = ImageCache::TakeCompleted();

  if (!m_Loaded && m_Model->GetStickers(m_ProfileId, "recent", m_Stickers))
  {
    m_Loaded = true;
    m_Index = 0;
    m_TopRow = 0;
    dirty = true;
  }

  return dirty;
}

void UiStickerDialog::UpdateLayout()
{
  static const int maxRows =
    std::max(1, std::min(UiConfig::GetNum("inline_sticker_max_rows"), KittyEncoder::GetDiacriticCount()));

  int cellW = 0;
  int cellH = 0;
  TermGfx::GetCellPixelSize(cellW, cellH);
  if ((cellW <= 0) || (cellH <= 0))
  {
    cellW = 1;
    cellH = 2;
  }

  // slot = padding column + image + padding column wide, image rows + one caption row high
  m_ImgRows = std::max(1, std::min(maxRows, (m_H - 1) / 2));
  // image placeholder columns are limited by the diacritic table, and must fit the window
  m_ImgCols = std::max(1, std::min({ (m_ImgRows * cellH) / cellW, KittyEncoder::GetDiacriticCount(), m_W - 2 }));
  m_Cols = StickerUtil::Columns(m_W, m_ImgCols + 2);
  m_VisibleRows = std::max(1, m_H / (m_ImgRows + 1));
}

void UiStickerDialog::KeyHandler(wint_t p_Key)
{
  static wint_t keyCancel = UiKeyConfig::GetKey("cancel");
  static wint_t keyQuit = UiKeyConfig::GetKey("quit");
  static wint_t keyFocusEntry = UiKeyConfig::GetKey("focus_entry");
  static wint_t keyOk = UiKeyConfig::GetKey("ok");
  static wint_t keyLeft = UiKeyConfig::GetKey("left");
  static wint_t keyRight = UiKeyConfig::GetKey("right");
  static wint_t keyUp = UiKeyConfig::GetKey("up");
  static wint_t keyDown = UiKeyConfig::GetKey("down");
  static wint_t keyPrevPage = UiKeyConfig::GetKey("prev_page");
  static wint_t keyNextPage = UiKeyConfig::GetKey("next_page");
  static wint_t keyHome = UiKeyConfig::GetKey("home");
  static wint_t keyEnd = UiKeyConfig::GetKey("end");
  static wint_t keyTerminalFocusIn = UiKeyConfig::GetKey("terminal_focus_in");
  static wint_t keyTerminalFocusOut = UiKeyConfig::GetKey("terminal_focus_out");
  static wint_t keyTerminalResize = UiKeyConfig::GetKey("terminal_resize");

  const int count = static_cast<int>(m_Stickers.size());
  bool isDirty = true;
  if (p_Key == keyTerminalResize)
  {
    Cleanup();
    m_Model->SetHelpOffset(0);
    m_Model->ReinitView();
    m_Model->Draw();
    curs_set(0);
    Init();
    UpdateLayout();
  }
  else if (p_Key == keyTerminalFocusIn)
  {
    m_Model->SetTerminalActive(true);
  }
  else if (p_Key == keyTerminalFocusOut)
  {
    m_Model->SetTerminalActive(false);
  }
  else if ((p_Key == keyCancel) || (p_Key == keyQuit) || (p_Key == keyFocusEntry))
  {
    m_Result = false;
    m_Running = false;
  }
  else if (p_Key == keyOk)
  {
    if (count > 0)
    {
      m_SelectedFileId = m_Stickers.at(m_Index).fileId;
      m_Result = true;
      m_Running = false;
    }
  }
  else if (p_Key == keyLeft)
  {
    m_Index = StickerUtil::Move(m_Index, -1, 0, m_Cols, count);
  }
  else if (p_Key == keyRight)
  {
    m_Index = StickerUtil::Move(m_Index, 1, 0, m_Cols, count);
  }
  else if (p_Key == keyUp)
  {
    m_Index = StickerUtil::Move(m_Index, 0, -1, m_Cols, count);
  }
  else if (p_Key == keyDown)
  {
    m_Index = StickerUtil::Move(m_Index, 0, 1, m_Cols, count);
  }
  else if (p_Key == keyPrevPage)
  {
    for (int i = 0; i < m_VisibleRows; ++i) m_Index = StickerUtil::Move(m_Index, 0, -1, m_Cols, count);
  }
  else if (p_Key == keyNextPage)
  {
    for (int i = 0; i < m_VisibleRows; ++i) m_Index = StickerUtil::Move(m_Index, 0, 1, m_Cols, count);
  }
  else if (p_Key == keyHome)
  {
    m_Index = 0;
  }
  else if (p_Key == keyEnd)
  {
    m_Index = std::max(0, count - 1);
  }
  else
  {
    isDirty = false;
  }

  if (isDirty && m_Running)
  {
    Draw();
  }
}

void UiStickerDialog::Draw()
{
  static int colorPair = UiColorConfig::GetColorPair("dialog_color");
  static int attribute = UiColorConfig::GetAttribute("dialog_attr");

  ImageCache::BeginDrawPass();

  werase(m_Win);
  wbkgd(m_Win, colorPair | ' ');
  wattron(m_Win, attribute | colorPair);

  const int count = static_cast<int>(m_Stickers.size());
  if (count == 0)
  {
    const std::wstring msg = m_Loaded ? L"No stickers" : L"Loading...";
    mvwaddnwstr(m_Win, 0, 0, msg.c_str(), std::min(static_cast<int>(msg.size()), m_W));
  }
  else
  {
    const int totalRows = (count + m_Cols - 1) / m_Cols;
    m_TopRow = StickerUtil::ScrollRow(m_Index / m_Cols, m_VisibleRows, totalRows, m_TopRow);
    const int slotW = m_ImgCols + 2;
    const int slotH = m_ImgRows + 1;
    for (int row = 0; row < m_VisibleRows; ++row)
    {
      for (int col = 0; col < m_Cols; ++col)
      {
        const int index = ((m_TopRow + row) * m_Cols) + col;
        if (index >= count) break;
        DrawSlot(index, col * slotW, row * slotH, index == m_Index);
      }
    }
  }

  wattroff(m_Win, attribute | colorPair);
  wrefresh(m_Win);
}

void UiStickerDialog::DrawSlot(int p_Index, int p_X, int p_Y, bool p_Selected)
{
  static int attribute = UiColorConfig::GetAttribute("dialog_attr");
  static int attributeSelected = UiColorConfig::GetAttribute("dialog_attr_selected");

  const StickerInfo& sticker = m_Stickers.at(p_Index);

  // the thumbnail path is a plain filesystem path, the same key type the history view gives ImageCache
  ImageInfo info;
  if (TermGfx::IsEnabled() && !sticker.thumbnailPath.empty() &&
      (ImageCache::Request(sticker.thumbnailPath, &info) == ImageCache::State::Ready))
  {
    int cellW = 0;
    int cellH = 0;
    TermGfx::GetCellPixelSize(cellW, cellH);
    const CellGrid grid = ImageGeom::FitToCells(info.pxWidth, info.pxHeight, cellW, cellH, m_ImgCols, m_ImgRows);
    const int id = (grid.cols > 0) ? ImageCache::Acquire(sticker.thumbnailPath, info, grid.cols, grid.rows) : 0;
    for (int row = 0; (id != 0) && (row < grid.rows); ++row)
    {
      UiImageRow::Draw(m_Win, p_Y + row, p_X + 1, row, id, grid.cols);
    }
  }

  // image cells cannot be reversed, so the selection is a bar on each side plus a highlighted caption
  if (p_Selected)
  {
    for (int row = 0; row < m_ImgRows; ++row)
    {
      mvwaddch(m_Win, p_Y + row, p_X, '|');
      mvwaddch(m_Win, p_Y + row, p_X + m_ImgCols + 1, '|');
    }
  }

  std::string emoji = sticker.emoji;
  if (!m_Model->GetEmojiEnabled()) emoji = StrUtil::Textize(emoji);
  const std::wstring caption = StrUtil::TrimPadWString(StrUtil::ToWString(emoji), m_ImgCols);

  const int captionAttribute = p_Selected ? attributeSelected : attribute;
  wattroff(m_Win, attribute);
  wattron(m_Win, captionAttribute);
  mvwaddnwstr(m_Win, p_Y + m_ImgRows, p_X + 1, caption.c_str(), std::min(static_cast<int>(caption.size()), m_ImgCols));
  wattroff(m_Win, captionAttribute);
  wattron(m_Win, attribute);
}
