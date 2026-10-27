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

static void ExpectNoOverlap(const LayoutOut& out);

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
  ExpectNoOverlap(ComputeLayout(Base()));
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

static bool Overlaps(const LayoutRect& a, const LayoutRect& b)
{
  if (a.w <= 0 || a.h <= 0 || b.w <= 0 || b.h <= 0) return false;
  return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
}

static void ExpectNoOverlap(const LayoutOut& out)
{
  const LayoutRect* rects[] = { &out.top, &out.help, &out.entry, &out.status, &out.list, &out.border, &out.history };
  for (int i = 0; i < 7; ++i)
  {
    for (int j = i + 1; j < 7; ++j)
    {
      unittest::ExpectEqual(bool, Overlaps(*rects[i], *rects[j]), false);
    }
  }
}

static void TestClassicRight()
{
  LayoutIn in = Base();
  in.listRight = true;
  const LayoutOut out = ComputeLayout(in);
  ExpectRect(out.history, 0, 1, 65, 16);
  ExpectRect(out.border, 65, 1, 1, 16);
  ExpectRect(out.list, 66, 1, 14, 16);
  ExpectRect(out.status, 0, 17, 80, 1);
  ExpectRect(out.entry, 0, 18, 80, 5);
  ExpectNoOverlap(out);
}

static void TestSidebarLeft()
{
  LayoutIn in = Base();
  in.sidebar = true;
  const LayoutOut out = ComputeLayout(in);
  ExpectRect(out.top, 0, 0, 80, 1);
  ExpectRect(out.help, 0, 23, 80, 1);
  ExpectRect(out.list, 0, 1, 14, 22);
  ExpectRect(out.border, 14, 1, 1, 22);
  ExpectRect(out.history, 15, 1, 65, 16);
  ExpectRect(out.status, 15, 17, 65, 1);
  ExpectRect(out.entry, 15, 18, 65, 5);
  ExpectNoOverlap(out);
}

static void TestSidebarRight()
{
  LayoutIn in = Base();
  in.sidebar = true;
  in.listRight = true;
  const LayoutOut out = ComputeLayout(in);
  ExpectRect(out.history, 0, 1, 65, 16);
  ExpectRect(out.status, 0, 17, 65, 1);
  ExpectRect(out.entry, 0, 18, 65, 5);
  ExpectRect(out.border, 65, 1, 1, 22);
  ExpectRect(out.list, 66, 1, 14, 22);
  ExpectNoOverlap(out);
}

static void TestSidebarGrowAndDisabled()
{
  LayoutIn in = Base();
  in.sidebar = true;
  in.entryLines = 30;
  LayoutOut out = ComputeLayout(in);
  ExpectRect(out.entry, 15, 10, 65, 13);
  ExpectRect(out.status, 15, 9, 65, 1);
  ExpectRect(out.history, 15, 1, 65, 8);
  ExpectRect(out.list, 0, 1, 14, 22);

  // no list: the column is full width, same rects as classic for history/status/entry
  in = Base();
  in.sidebar = true;
  in.list = false;
  out = ComputeLayout(in);
  ExpectRect(out.history, 0, 1, 80, 16);
  ExpectRect(out.status, 0, 17, 80, 1);
  ExpectRect(out.entry, 0, 18, 80, 5);
  unittest::ExpectEqual(int, out.list.w, 0);
  unittest::ExpectEqual(int, out.border.w, 0);

  in.listRight = true;
  out = ComputeLayout(in);
  ExpectRect(out.history, 0, 1, 80, 16);
  unittest::ExpectEqual(int, out.list.w, 0);
  unittest::ExpectEqual(int, out.border.w, 0);

  in = Base();
  in.sidebar = true;
  in.listWidth = 0;
  out = ComputeLayout(in);
  ExpectRect(out.history, 0, 1, 80, 16);
  unittest::ExpectEqual(int, out.border.w, 0);
}

static void TestSidebarTinyTerminal()
{
  for (int right = 0; right < 2; ++right)
  {
    LayoutIn in = Base();
    in.sidebar = true;
    in.listRight = (right == 1);
    in.termW = 20;
    in.termH = 6;
    in.listWidth = 30;
    const LayoutOut out = ComputeLayout(in);
    unittest::ExpectEqual(bool, HasNegativeValues(out.top), false);
    unittest::ExpectEqual(bool, HasNegativeValues(out.help), false);
    unittest::ExpectEqual(bool, HasNegativeValues(out.entry), false);
    unittest::ExpectEqual(bool, HasNegativeValues(out.status), false);
    unittest::ExpectEqual(bool, HasNegativeValues(out.list), false);
    unittest::ExpectEqual(bool, HasNegativeValues(out.border), false);
    unittest::ExpectEqual(bool, HasNegativeValues(out.history), false);
    unittest::ExpectEqual(int, out.history.w, 0);
    unittest::ExpectEqual(int, out.entry.w, 0);
    unittest::ExpectEqual(int, out.status.w, 0);
  }
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
    TestClassicRight();
    TestSidebarLeft();
    TestSidebarRight();
    TestSidebarGrowAndDisabled();
    TestSidebarTinyTerminal();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
