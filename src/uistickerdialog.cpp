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
  m_Model->RequestStickerSets(m_ProfileId);
  m_Model->RequestStickers(m_ProfileId, "recent", 0, "");
  m_Key = "recent";
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

  if (!m_Loaded && m_Model->GetStickers(m_ProfileId, m_Key, m_Stickers))
  {
    m_Loaded = true;
    m_Index = 0;
    m_TopRow = 0;
    dirty = true;
  }

  if (!m_SetsLoaded && m_Model->GetStickerSets(m_ProfileId, m_Sets))
  {
    m_SetsLoaded = true;
    dirty = true;
  }

  if (m_QueryDirty && ((TimeUtil::GetCurrentTimeMSec() - m_QueryTime) >= 300))
  {
    m_QueryDirty = false;
    SelectView();
    dirty = true;
  }

  return dirty;
}

std::string UiStickerDialog::BuildKey() const
{
  if (!m_Query.empty()) return "q:" + StrUtil::ToString(m_Query);
  if (m_Tab == 0) return "recent";
  return "set:" + std::to_string(m_Sets.at(m_Tab - 1).id);
}

void UiStickerDialog::SelectView()
{
  m_Key = BuildKey();
  m_Stickers.clear();
  m_Index = 0;
  m_TopRow = 0;
  m_Loaded = m_Model->GetStickers(m_ProfileId, m_Key, m_Stickers);
  if (m_Loaded) return;

  if (!m_Query.empty()) m_Model->RequestStickers(m_ProfileId, m_Key, 0, StrUtil::ToString(m_Query));
  else if (m_Tab == 0) m_Model->RequestStickers(m_ProfileId, m_Key, 0, "");
  else m_Model->RequestStickers(m_ProfileId, m_Key, m_Sets.at(m_Tab - 1).id, "");
}

void UiStickerDialog::UpdateFooter()
{
  SetFooter(m_Query.empty() ? "" : "Search: " + StrUtil::ToString(m_Query));
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
  // row 0 is the header, the grid is m_H - 1 rows high
  m_ImgRows = std::max(1, std::min(maxRows, (m_H - 2) / 2));
  // image placeholder columns are limited by the diacritic table, and must fit the window
  m_ImgCols = std::max(1, std::min({ (m_ImgRows * cellH) / cellW, KittyEncoder::GetDiacriticCount(), m_W - 2 }));
  m_Cols = StickerUtil::Columns(m_W, m_ImgCols + 2);
  m_VisibleRows = std::max(1, (m_H - 1) / (m_ImgRows + 1));
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
  static wint_t keyFocusNext = UiKeyConfig::GetKey("focus_next");
  static wint_t keyFocusPrev = UiKeyConfig::GetKey("focus_prev");
  static wint_t keyBackspace = UiKeyConfig::GetKey("backspace");
  static wint_t keyBackspaceAlt = UiKeyConfig::GetKey("backspace_alt");
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
  else if ((p_Key == keyFocusNext) || (p_Key == keyFocusPrev))
  {
    if (!m_Query.empty())
    {
      m_Query.clear();
      m_QueryDirty = false;
      UpdateFooter();
    }

    m_Tab = StickerUtil::CycleTab(m_Tab, (p_Key == keyFocusNext) ? 1 : -1, 1 + static_cast<int>(m_Sets.size()));
    SelectView();
  }
  else if ((p_Key == keyBackspace) || (p_Key == keyBackspaceAlt))
  {
    if (!m_Query.empty())
    {
      m_Query.pop_back();
      m_QueryDirty = true;
      m_QueryTime = TimeUtil::GetCurrentTimeMSec();
      UpdateFooter();
    }
  }
  else if (StrUtil::IsValidTextKey(p_Key)) // last, so named keys never type
  {
    m_Query += p_Key;
    m_QueryDirty = true;
    m_QueryTime = TimeUtil::GetCurrentTimeMSec();
    UpdateFooter();
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

  // header row: current tab title, or Search while a query is entered
  const int tabCount = 1 + static_cast<int>(m_Sets.size());
  std::wstring header = L"Search";
  if (m_Query.empty())
  {
    const std::wstring title = (m_Tab == 0) ? L"Recent" : StrUtil::ToWString(m_Sets.at(m_Tab - 1).title);
    header = (tabCount > 1) ? L"< " + title + L"  " + std::to_wstring(m_Tab + 1) + L"/" + std::to_wstring(tabCount) + L" >"
                            : title;
  }

  mvwaddnwstr(m_Win, 0, 0, header.c_str(), std::min(static_cast<int>(header.size()), m_W));

  const int count = static_cast<int>(m_Stickers.size());
  if (count == 0)
  {
    const std::wstring msg = m_Loaded ? L"No stickers" : L"Loading...";
    mvwaddnwstr(m_Win, 1, 0, msg.c_str(), std::min(static_cast<int>(msg.size()), m_W));
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
        DrawSlot(index, col * slotW, (row * slotH) + 1, index == m_Index);
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
