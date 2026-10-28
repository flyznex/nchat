// test016.cpp - PaneAt hit test over ComputeLayout output
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
  in.termW = 100;
  in.termH = 40;
  in.top = true;
  in.help = true;
  in.status = true;
  in.list = true;
  in.history = true;
  in.entry = true;
  in.listWidth = 30;
  in.entryMin = 4;
  in.entryLines = 1;
  return in;
}

static int Mid(int p_Start, int p_Len) { return p_Start + p_Len / 2; }

static void CheckLayout(const LayoutIn& p_In)
{
  const LayoutOut lo = ComputeLayout(p_In);
  unittest::ExpectTrue(PaneAt(lo, Mid(lo.list.x, lo.list.w), Mid(lo.list.y, lo.list.h)) == Pane::List);
  unittest::ExpectTrue(PaneAt(lo, Mid(lo.history.x, lo.history.w), Mid(lo.history.y, lo.history.h)) == Pane::History);
  unittest::ExpectTrue(PaneAt(lo, Mid(lo.entry.x, lo.entry.w), Mid(lo.entry.y, lo.entry.h)) == Pane::Entry);
  unittest::ExpectTrue(PaneAt(lo, lo.border.x, Mid(lo.border.y, lo.border.h)) == Pane::None);
  unittest::ExpectTrue(PaneAt(lo, Mid(lo.top.x, lo.top.w), lo.top.y) == Pane::None);
  unittest::ExpectTrue(PaneAt(lo, Mid(lo.status.x, lo.status.w), lo.status.y) == Pane::None);
  unittest::ExpectTrue(PaneAt(lo, -1, -1) == Pane::None);
  unittest::ExpectTrue(PaneAt(lo, p_In.termW, p_In.termH) == Pane::None);
}

static void TestLayouts()
{
  for (const bool sidebar : { true, false })
  {
    for (const bool listRight : { false, true })
    {
      LayoutIn in = Base();
      in.sidebar = sidebar;
      in.listRight = listRight;
      CheckLayout(in);
    }
  }
}

static void TestListHidden()
{
  LayoutIn in = Base();
  const LayoutOut shown = ComputeLayout(in);
  in.list = false;
  const LayoutOut hidden = ComputeLayout(in);
  for (int x = shown.list.x; x < (shown.list.x + shown.list.w); ++x)
  {
    for (int y = shown.list.y; y < (shown.list.y + shown.list.h); ++y)
    {
      unittest::ExpectTrue(PaneAt(hidden, x, y) != Pane::List);
    }
  }
}

int main()
{
  int rv = 0;

  try
  {
    TestLayouts();
    TestListHidden();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
