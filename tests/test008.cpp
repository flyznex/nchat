// test008.cpp - screen layout computation
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <iostream>

#include "layout.h"

#include "unittest.h"

static LayoutIn Base()
{
  LayoutIn in;
  in.termW = 80;
  in.termH = 24;
  in.top = true;
  in.help = true;
  in.status = true;
  in.list = true;
  in.history = true;
  in.entry = true;
  in.listWidth = 14;
  in.entryMin = 4;
  in.entryLines = 1;
  return in;
}

static void ExpectRect(const LayoutRect& p_Rect, int p_X, int p_Y, int p_W, int p_H)
{
  unittest::ExpectEqual(int, p_Rect.x, p_X);
  unittest::ExpectEqual(int, p_Rect.y, p_Y);
  unittest::ExpectEqual(int, p_Rect.w, p_W);
  unittest::ExpectEqual(int, p_Rect.h, p_H);
}

static bool HasNegativeValues(const LayoutRect& p_Rect)
{
  return p_Rect.x < 0 || p_Rect.y < 0 || p_Rect.w < 0 || p_Rect.h < 0;
}

static void TestDefault()
{
  const LayoutOut out = ComputeLayout(Base());
  ExpectRect(out.top, 0, 0, 80, 1);
  ExpectRect(out.help, 0, 23, 80, 1);
  ExpectRect(out.entry, 0, 18, 80, 5);
  ExpectRect(out.status, 0, 17, 80, 1);
  ExpectRect(out.list, 0, 1, 14, 16);
  ExpectRect(out.border, 14, 1, 1, 16);
  ExpectRect(out.history, 15, 1, 65, 16);
}

static void TestEntryGrow()
{
  unittest::ExpectEqual(int, EntryContentHeight(4, 1, 24), 4);
  unittest::ExpectEqual(int, EntryContentHeight(4, 7, 24), 7);
  unittest::ExpectEqual(int, EntryContentHeight(4, 30, 24), 12);
  unittest::ExpectEqual(int, EntryContentHeight(20, 1, 24), 20);

  LayoutIn in = Base();
  in.entryLines = 30;
  const LayoutOut out = ComputeLayout(in);
  ExpectRect(out.entry, 0, 10, 80, 13);
  ExpectRect(out.status, 0, 9, 80, 1);
  ExpectRect(out.history, 15, 1, 65, 8);
}

static void TestDisabled()
{
  LayoutIn in = Base();
  in.list = false;
  LayoutOut out = ComputeLayout(in);
  ExpectRect(out.list, 0, 1, 0, 16);
  unittest::ExpectEqual(int, out.border.w, 0);
  ExpectRect(out.history, 0, 1, 80, 16);

  in = Base();
  in.listWidth = 0;
  out = ComputeLayout(in);
  unittest::ExpectEqual(int, out.border.w, 0);
  ExpectRect(out.history, 0, 1, 80, 16);

  in = Base();
  in.help = false;
  in.top = false;
  out = ComputeLayout(in);
  ExpectRect(out.entry, 0, 19, 80, 5);
  ExpectRect(out.history, 15, 0, 65, 18);

  in = Base();
  in.entry = false;
  out = ComputeLayout(in);
  unittest::ExpectEqual(int, out.entry.h, 0);
  ExpectRect(out.status, 0, 22, 80, 1);
}

static void TestTinyTerminal()
{
  LayoutIn in = Base();
  in.termW = 20;
  in.termH = 6;
  in.listWidth = 30;
  const LayoutOut out = ComputeLayout(in);

  // All rects must have non-negative dimensions
  unittest::ExpectEqual(bool, HasNegativeValues(out.top), false);
  unittest::ExpectEqual(bool, HasNegativeValues(out.help), false);
  unittest::ExpectEqual(bool, HasNegativeValues(out.entry), false);
  unittest::ExpectEqual(bool, HasNegativeValues(out.status), false);
  unittest::ExpectEqual(bool, HasNegativeValues(out.list), false);
  unittest::ExpectEqual(bool, HasNegativeValues(out.border), false);
  unittest::ExpectEqual(bool, HasNegativeValues(out.history), false);

  // History width should be 0 when terminal is narrower than list+border
  unittest::ExpectEqual(int, out.history.w, 0);
}

int main()
{
  int rv = 0;
  try
  {
    TestDefault();
    TestEntryGrow();
    TestDisabled();
    TestTinyTerminal();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
