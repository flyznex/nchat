// layout.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "layout.h"

#include <algorithm>

int EntryContentHeight(int p_EntryMin, int p_EntryLines, int p_TermH)
{
  return std::max(p_EntryMin, std::min(p_EntryLines, p_TermH / 2));
}

LayoutOut ComputeLayout(const LayoutIn& p_In)
{
  const int topH = p_In.top ? 1 : 0;
  const int helpH = p_In.help ? 1 : 0;
  const int statusH = p_In.status ? 1 : 0;
  // row 0 of the entry window is the title line
  const int entryH = p_In.entry ? (1 + EntryContentHeight(p_In.entryMin, p_In.entryLines, p_In.termH)) : 0;
  const int midH = p_In.termH - topH - helpH - entryH - statusH;

  LayoutOut out;
  out.top = { 0, 0, p_In.top ? p_In.termW : 0, topH };
  out.help = { 0, p_In.termH - helpH, p_In.help ? p_In.termW : 0, helpH };
  out.entry = { 0, p_In.termH - helpH - entryH, p_In.entry ? p_In.termW : 0, entryH };
  out.status = { 0, p_In.termH - helpH - entryH - statusH, p_In.status ? p_In.termW : 0, statusH };

  const int listW = p_In.list ? p_In.listWidth : 0;
  const int borderW = (p_In.list && (p_In.listWidth > 0)) ? 1 : 0;
  out.list = { 0, topH, listW, midH };
  out.border = { listW, topH, borderW, midH };
  const int histX = listW + borderW;
  out.history = { histX, topH, p_In.termW - histX, midH };
  return out;
}
