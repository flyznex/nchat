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
  const int midH = std::max(0, p_In.termH - topH - helpH - entryH - statusH);  // ponytail: tiny terminals collapse to zero-size regions

  const int bodyH = std::max(0, p_In.termH - topH - helpH);

  const int listW = p_In.list ? p_In.listWidth : 0;
  const int borderW = (p_In.list && (p_In.listWidth > 0)) ? 1 : 0;
  const int colW = std::max(0, p_In.termW - listW - borderW);
  const int colX = p_In.listRight ? 0 : (listW + borderW);
  const int borderX = p_In.listRight ? colW : listW;
  const int listX = p_In.listRight ? (colW + borderW) : 0;

  // sidebar: the list runs the full height and status/entry share the history column; classic: they span the width
  const int lowX = p_In.sidebar ? colX : 0;
  const int lowW = p_In.sidebar ? colW : p_In.termW;
  const int sideH = p_In.sidebar ? bodyH : midH;

  LayoutOut out;
  out.top = { 0, 0, p_In.top ? p_In.termW : 0, topH };
  out.help = { 0, std::max(0, p_In.termH - helpH), p_In.help ? p_In.termW : 0, helpH };
  out.entry = { lowX, std::max(0, p_In.termH - helpH - entryH), p_In.entry ? lowW : 0, entryH };
  out.status = { lowX, std::max(0, p_In.termH - helpH - entryH - statusH), p_In.status ? lowW : 0, statusH };
  out.list = { listX, topH, listW, sideH };
  out.border = { borderX, topH, borderW, sideH };
  out.history = { colX, topH, colW, midH };
  return out;
}
