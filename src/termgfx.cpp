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

bool TermGfx::m_Enabled = false;
bool TermGfx::m_InTmux = false;

void TermGfx::Init()
{
  m_Enabled = false;

  const std::string mode = UiConfig::GetStr("inline_images");
  if (mode == "off") return;

  if (!has_colors() || (COLORS < 256))
  {
    LOG_WARNING("inline images disabled: terminal has less than 256 colors");
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
      LOG_DEBUG("inline images: kitty graphics terminal not detected");
      return;
    }
  }

  m_InTmux = (getenv("TMUX") != nullptr);
  if (m_InTmux)
  {
    std::string passthrough;
    SysUtil::RunCommand("tmux show-options -pAqv allow-passthrough", &passthrough);
    if ((passthrough != "on") && (passthrough != "all"))
    {
      LOG_WARNING("inline images disabled: tmux allow-passthrough is '%s', set it to 'on'", passthrough.c_str());
      return;
    }
  }

  m_Enabled = true;
  LOG_DEBUG("inline images enabled (tmux %d)", m_InTmux);
}

bool TermGfx::IsEnabled()
{
  return m_Enabled;
}

void TermGfx::GetCellPixelSize(int& p_Width, int& p_Height)
{
  p_Width = 0;
  p_Height = 0;

  struct winsize ws;
  if ((ioctl(STDOUT_FILENO, TIOCGWINSZ, &ws) == 0) && (ws.ws_col > 0) && (ws.ws_row > 0) &&
      (ws.ws_xpixel > 0) && (ws.ws_ypixel > 0))
  {
    p_Width = ws.ws_xpixel / ws.ws_col;
    p_Height = ws.ws_ypixel / ws.ws_row;
  }
}

void TermGfx::Upload(int p_Id, int p_Cols, int p_Rows, const std::string& p_PngPath)
{
  if (!m_Enabled) return;

  const std::string png = FileUtil::ReadFile(p_PngPath);
  for (const std::string& seq : KittyEncoder::EncodeUpload(p_Id, p_Cols, p_Rows, png))
  {
    Write(seq);
  }
}

void TermGfx::Delete(int p_Id)
{
  if (!m_Enabled) return;

  Write(KittyEncoder::EncodeDelete(p_Id));
}

void TermGfx::Write(const std::string& p_Seq)
{
  const std::string out = m_InTmux ? KittyEncoder::WrapTmux(p_Seq) : p_Seq;
  fwrite(out.data(), 1, out.size(), stdout);
  fflush(stdout);
}
