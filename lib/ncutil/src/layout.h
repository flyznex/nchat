// layout.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

struct LayoutRect
{
  int x = 0;
  int y = 0;
  int w = 0;
  int h = 0;
};

struct LayoutIn
{
  int termW;
  int termH;
  bool top;
  bool help;
  bool status;
  bool list;
  bool history;
  bool entry;
  int listWidth;
  int entryMin;
  int entryLines;
};

struct LayoutOut
{
  LayoutRect top;
  LayoutRect help;
  LayoutRect entry;
  LayoutRect status;
  LayoutRect list;
  LayoutRect border;
  LayoutRect history;
};

int EntryContentHeight(int p_EntryMin, int p_EntryLines, int p_TermH);
LayoutOut ComputeLayout(const LayoutIn& p_In);
