// uiviewbase.cpp
//
// Copyright (c) 2019-2021 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "uiviewbase.h"

#include "scrollbar.h"

UiViewBase::UiViewBase(const UiViewParams& p_Params)
  : m_X(p_Params.x)
  , m_Y(p_Params.y)
  , m_W(p_Params.w)
  , m_H(p_Params.h)
  , m_Enabled(p_Params.enabled)
  , m_Model(p_Params.model)
  , m_Dirty(true)
{
  if (!m_Enabled) return;

  m_Win = newwin(m_H, m_W, m_Y, m_X);
}

UiViewBase::~UiViewBase()
{
  if (!m_Enabled) return;

  delwin(m_Win);
  m_Win = nullptr;
}

int UiViewBase::W()
{
  return m_Enabled ? m_W : 0;
}

int UiViewBase::H()
{
  return m_Enabled ? m_H : 0;
}

int UiViewBase::X()
{
  return m_Enabled ? m_X : 0;
}

int UiViewBase::Y()
{
  return m_Enabled ? m_Y : 0;
}

void UiViewBase::SetDirty(bool p_Dirty)
{
  m_Dirty = p_Dirty;
}

void UiViewBase::DrawScrollBar(WINDOW* p_Win, int p_Bkgd, int p_Attr, int p_Row0, int p_Track, int p_Total,
                               int p_First, int p_Visible)
{
  if (p_Win == nullptr) return;

  werase(p_Win);
  wbkgd(p_Win, p_Bkgd);

  int pos = 0;
  int len = 0;
  if (ScrollThumb(p_Total, p_First, p_Visible, p_Track, pos, len))
  {
    wattron(p_Win, p_Attr | A_REVERSE);
    for (int i = 0; i < len; ++i) mvwaddch(p_Win, p_Row0 + pos + i, 0, ' ');
    wattroff(p_Win, p_Attr | A_REVERSE);
  }

  wrefresh(p_Win);
}
