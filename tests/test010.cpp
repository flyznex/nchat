// test010.cpp - sticker list de-duplication and picker grid math
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <cstdint>
#include <iostream>
#include <string>
#include <vector>

#include "stickerutil.h"

#include "unittest.h"

static std::string Join(const std::vector<size_t>& p_Values)
{
  std::string s;
  for (size_t v : p_Values) s += std::to_string(v) + ",";
  return s;
}

static void TestDedupeCap()
{
  unittest::ExpectEqual(std::string, Join(StickerUtil::DedupeCap({ 5, 3, 5, 7, 3, 9 }, 10)), "0,1,3,5,");
  unittest::ExpectEqual(std::string, Join(StickerUtil::DedupeCap({ 5, 3, 5, 7, 3, 9 }, 2)), "0,1,");
  unittest::ExpectEqual(std::string, Join(StickerUtil::DedupeCap({ 5, 3 }, 0)), "");
  unittest::ExpectEqual(std::string, Join(StickerUtil::DedupeCap({}, 5)), "");
}

static void TestColumns()
{
  unittest::ExpectEqual(int, StickerUtil::Columns(146, 14), 10);
  unittest::ExpectEqual(int, StickerUtil::Columns(28, 14), 2);
  unittest::ExpectEqual(int, StickerUtil::Columns(5, 14), 1);
  unittest::ExpectEqual(int, StickerUtil::Columns(100, 0), 1);
}

static void TestMove()
{
  // 7 items, 3 columns: rows are [0 1 2] [3 4 5] [6]
  unittest::ExpectEqual(int, StickerUtil::Move(0, 1, 0, 3, 7), 1);
  unittest::ExpectEqual(int, StickerUtil::Move(6, 1, 0, 3, 7), 6);
  unittest::ExpectEqual(int, StickerUtil::Move(0, -1, 0, 3, 7), 0);
  unittest::ExpectEqual(int, StickerUtil::Move(1, 0, 1, 3, 7), 4);
  unittest::ExpectEqual(int, StickerUtil::Move(4, 0, 1, 3, 7), 6); // short last row: land on last item
  unittest::ExpectEqual(int, StickerUtil::Move(6, 0, 1, 3, 7), 6);
  unittest::ExpectEqual(int, StickerUtil::Move(4, 0, -1, 3, 7), 1);
  unittest::ExpectEqual(int, StickerUtil::Move(1, 0, -1, 3, 7), 1);
  unittest::ExpectEqual(int, StickerUtil::Move(3, 0, 0, 3, 7), 3);
  unittest::ExpectEqual(int, StickerUtil::Move(2, 1, 0, 3, 0), 0);
}

static void TestScrollRow()
{
  unittest::ExpectEqual(int, StickerUtil::ScrollRow(0, 3, 10, 0), 0);
  unittest::ExpectEqual(int, StickerUtil::ScrollRow(3, 3, 10, 0), 1);
  unittest::ExpectEqual(int, StickerUtil::ScrollRow(2, 3, 10, 0), 0);
  unittest::ExpectEqual(int, StickerUtil::ScrollRow(9, 3, 10, 0), 7);
  unittest::ExpectEqual(int, StickerUtil::ScrollRow(1, 3, 10, 5), 1);
  unittest::ExpectEqual(int, StickerUtil::ScrollRow(9, 3, 10, 9), 7);
  unittest::ExpectEqual(int, StickerUtil::ScrollRow(4, 0, 10, 0), 4);
  unittest::ExpectEqual(int, StickerUtil::ScrollRow(0, 5, 2, 3), 0);
}

int main()
{
  int rv = 0;

  try
  {
    TestDedupeCap();
    TestColumns();
    TestMove();
    TestScrollRow();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
