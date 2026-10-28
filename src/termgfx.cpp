// termgfx.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "termgfx.h"

#include <cstdio>
#include <cstdlib>

#include <sys/ioctl.h>
#include <unistd.h>

#include <ncurses.h>

#include "fileutil.h"
#include "kittyencoder.h"
#include "log.h"
#include "sysutil.h"
#include "uiconfig.h"

TermGfx::Mode TermGfx::m_Mode = TermGfx::Mode::Off;
bool TermGfx::m_InTmux = false;
bool TermGfx::m_LocalFiles = false;

void TermGfx::Init()
{
  m_Mode = Mode::Off;

  const std::string mode = UiConfig::GetStr("inline_images");
  if (mode == "off") return;

  if (!has_colors() || (COLORS < 256))
  {
    LOG_WARNING("inline images disabled: terminal has less than 256 colors");
    return;
  }

  if (mode == "blocks")
  {
    EnableBlocks();
    return;
  }

  if (mode != "on")
  {
    const char* term = getenv("TERM");
    const bool supported = KittyEncoder::IsSupportedTerm(getenv("KITTY_WINDOW_ID") != nullptr,
                                                         getenv("GHOSTTY_RESOURCES_DIR") != nullptr,
                                                         term != nullptr ? term : "");
    if (!supported)
    {
      LOG_DEBUG("inline images: kitty graphics terminal not detected, using block characters");
      EnableBlocks();
      return;
    }
  }

  // terminal reads the png itself (t=f) unless it may be on another machine, which would stream 1 MB per image
  m_LocalFiles = (getenv("SSH_CONNECTION") == nullptr) && (getenv("SSH_TTY") == nullptr);
  m_InTmux = (getenv("TMUX") != nullptr);
  if (m_InTmux)
  {
    std::string passthrough;
    SysUtil::RunCommand("tmux show-options -pAqv allow-passthrough", &passthrough);
    if ((passthrough != "on") && (passthrough != "all"))
    {
      LOG_WARNING("inline images: tmux allow-passthrough is '%s', set it to 'on' for kitty graphics",
                  passthrough.c_str());
      if (mode != "on") EnableBlocks();
      return;
    }
  }

  m_Mode = Mode::Kitty;
  LOG_DEBUG("inline images enabled (kitty, tmux %d)", m_InTmux);
}

void TermGfx::EnableBlocks()
{
  if (COLOR_PAIRS < 1024)
  {
    LOG_WARNING("inline images disabled: terminal supports only %d color pairs", COLOR_PAIRS);
    return;
  }

  m_Mode = Mode::Blocks;
  LOG_DEBUG("inline images enabled (block characters)");
}

bool TermGfx::IsEnabled()
{
  return m_Mode != Mode::Off;
}

bool TermGfx::IsBlocks()
{
  return m_Mode == Mode::Blocks;
}

void TermGfx::GetCellPixelSize(int& p_Width, int& p_Height)
{
  p_Width = 0;
  p_Height = 0;
  if (m_Mode == Mode::Blocks) return; // callers assume square pixels (1:2 cells) when unknown

  struct winsize ws;
  if ((ioctl(STDOUT_FILENO, TIOCGWINSZ, &ws) == 0) && (ws.ws_col > 0) && (ws.ws_row > 0) &&
      (ws.ws_xpixel > 0) && (ws.ws_ypixel > 0))
  {
    p_Width = ws.ws_xpixel / ws.ws_col;
    p_Height = ws.ws_ypixel / ws.ws_row;
  }
}

bool TermGfx::Upload(int p_Id, int p_Cols, int p_Rows, const std::string& p_PngPath)
{
  if (m_Mode != Mode::Kitty) return false;

  if (m_LocalFiles)
  {
    if (!FileUtil::Exists(p_PngPath))
    {
      LOG_WARNING("inline image upload failed: cannot find %s", p_PngPath.c_str());
      return false;
    }

    Write(KittyEncoder::EncodeUploadFile(p_Id, p_Cols, p_Rows, p_PngPath));
    return true;
  }

  const std::string png = FileUtil::ReadFile(p_PngPath);
  if (png.empty())
  {
    LOG_WARNING("inline image upload failed: cannot read %s", p_PngPath.c_str());
    return false;
  }

  for (const std::string& seq : KittyEncoder::EncodeUpload(p_Id, p_Cols, p_Rows, png))
  {
    Write(seq);
  }
  return true;
}

void TermGfx::Delete(int p_Id)
{
  if (m_Mode != Mode::Kitty) return;

  Write(KittyEncoder::EncodeDelete(p_Id));
}

void TermGfx::Write(const std::string& p_Seq)
{
  const std::string out = m_InTmux ? KittyEncoder::WrapTmux(p_Seq) : p_Seq;
  fwrite(out.data(), 1, out.size(), stdout);
  fflush(stdout);
}
