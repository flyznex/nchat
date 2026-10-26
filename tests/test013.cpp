// test013.cpp - inline markdown styling of message text
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <iostream>

#include "mdstyle.h"
#include "strutil.h"

#include "unittest.h"

static std::wstring Strip(const std::wstring& p_In, std::vector<uint8_t>& p_Styles)
{
  std::wstring out;
  MdStyleUtil::Parse(p_In, out, p_Styles);
  return out;
}

static void TestBasic()
{
  std::vector<uint8_t> st;
  unittest::ExpectEqual(std::wstring, Strip(L"*bold* and _it_", st), L"bold and it");
  const std::vector<uint8_t> expected = { 1, 1, 1, 1, 0, 0, 0, 0, 0, 2, 2 };
  unittest::ExpectTrue(st == expected);

  unittest::ExpectEqual(std::wstring, Strip(L"use `x*y*` now", st), L"use x*y* now");
  unittest::ExpectTrue(st[4] == MdStyleCode);
  unittest::ExpectTrue(st[7] == MdStyleCode);
  unittest::ExpectTrue(st[8] == MdStyleNone);

  unittest::ExpectEqual(std::wstring, Strip(L"*a _b_ c*", st), L"a b c");
  unittest::ExpectTrue(st[0] == MdStyleBold);
  unittest::ExpectTrue(st[2] == (MdStyleBold | MdStyleItalic));

  unittest::ExpectEqual(std::wstring, Strip(L"```\nls -l\n```", st), L"ls -l");
  unittest::ExpectTrue(st[0] == MdStyleCode);
}

static void TestNotMarkup()
{
  std::vector<uint8_t> st;
  unittest::ExpectEqual(std::wstring, Strip(L"snake_case_name", st), L"snake_case_name");
  unittest::ExpectEqual(std::wstring, Strip(L"2 * 3 * 4", st), L"2 * 3 * 4");
  unittest::ExpectEqual(std::wstring, Strip(L"*unclosed", st), L"*unclosed");
  unittest::ExpectEqual(std::wstring, Strip(L"see https://x.com/_a_/b_c_ ok", st), L"see https://x.com/_a_/b_c_ ok");
  unittest::ExpectEqual(std::wstring, Strip(L"*a\n\nb*", st), L"*a\n\nb*");
  unittest::ExpectEqual(std::wstring, Strip(L"``", st), L"``");
}

static void TestEscapes()
{
  std::vector<uint8_t> st;
  unittest::ExpectEqual(std::wstring, Strip(L"\\*not bold\\*", st), L"*not bold*");
  unittest::ExpectTrue(st[0] == MdStyleNone);
  unittest::ExpectEqual(std::wstring, Strip(L"C:\\dir", st), L"C:\\dir");
}

static void TestWrapStyles()
{
  std::vector<uint8_t> st;
  const std::wstring text = Strip(L"aaa *bbb ccc* ddd", st); // "aaa bbb ccc ddd"
  const std::vector<std::wstring> lines = StrUtil::WordWrap(text, 9, false, false, false, 2);
  const std::vector<std::vector<uint8_t>> ls = MdStyleUtil::WrapStyles(text, st, lines);
  unittest::ExpectTrue(lines.size() == ls.size());
  for (size_t i = 0; i < lines.size(); ++i)
  {
    unittest::ExpectTrue(lines[i].size() == ls[i].size());
    for (size_t j = 0; j < lines[i].size(); ++j)
    {
      const bool inBold = (lines[i][j] == L'b') || (lines[i][j] == L'c');
      unittest::ExpectTrue(ls[i][j] == (inBold ? MdStyleBold : MdStyleNone));
    }
  }

  // tab expanded by the wrapper
  const std::wstring tabText = Strip(L"\t*x*", st);
  const std::vector<std::wstring> tabLines = StrUtil::WordWrap(tabText, 20, false, false, false, 2);
  const std::vector<std::vector<uint8_t>> tls = MdStyleUtil::WrapStyles(tabText, st, tabLines);
  unittest::ExpectTrue(tls.size() == 1);
  unittest::ExpectTrue(tls[0].size() == tabLines[0].size());
  unittest::ExpectTrue(tls[0].back() == MdStyleBold);
}

int main()
{
  int rv = 0;
  try
  {
    TestBasic();
    TestNotMarkup();
    TestEscapes();
    TestWrapStyles();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
