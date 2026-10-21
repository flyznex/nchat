// test009.cpp - entry box title line
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <clocale>
#include <iostream>
#include <string>
#include <wchar.h>

#include "entrytitle.h"
#include "strutil.h"

#include "unittest.h"

static const std::string hint = "Enter send | Alt-Enter newline";

static void TestModes()
{
  unittest::ExpectEqual(std::wstring, BuildEntryTitle(EntryMode::Compose, "", hint, "Esc", 60),
                        StrUtil::TrimPadWString(L"Message | Enter send | Alt-Enter newline", 60));
  unittest::ExpectEqual(std::wstring, BuildEntryTitle(EntryMode::Edit, "", "Ctrl-X send", "Esc", 60),
                        StrUtil::TrimPadWString(L"Editing message | Ctrl-X send | Esc cancel", 60));
  unittest::ExpectEqual(std::wstring, BuildEntryTitle(EntryMode::Reply, "Ann", "Ctrl-X send", "Esc", 60),
                        StrUtil::TrimPadWString(L"Replying to Ann | Ctrl-X send | Esc cancel", 60));
}

static void TestSanitize()
{
  // newlines in the target name never reach the title, empty/missing target falls back
  const std::wstring t = BuildEntryTitle(EntryMode::Reply, "A\nB\r\nC", "x", "Esc", 60);
  unittest::ExpectEqual(std::wstring, t.substr(0, 20), std::wstring(L"Replying to A B  C |"));
  unittest::ExpectEqual(size_t, t.find(L'\n'), std::wstring::npos);
  unittest::ExpectEqual(std::wstring, BuildEntryTitle(EntryMode::Reply, "", "x", "Esc", 60).substr(0, 23),
                        std::wstring(L"Replying to message | x"));
}

static void TestWidth()
{
  unittest::ExpectEqual(std::wstring, BuildEntryTitle(EntryMode::Compose, "", hint, "Esc", 0), std::wstring(L""));
  unittest::ExpectEqual(std::wstring, BuildEntryTitle(EntryMode::Compose, "", hint, "Esc", -3), std::wstring(L""));
  unittest::ExpectEqual(std::wstring, BuildEntryTitle(EntryMode::Compose, "", hint, "Esc", 1), std::wstring(L"M"));
  for (int w = 1; w < 80; ++w)
  {
    unittest::ExpectEqual(int, StrUtil::WStringWidth(BuildEntryTitle(EntryMode::Reply, "Ann", hint, "Esc", w)), w);
  }

  // long name is shortened so the hints survive
  const std::wstring t = BuildEntryTitle(EntryMode::Reply, "Averyveryveryverylongname", "Ctrl-X send", "Esc", 45);
  unittest::ExpectEqual(int, StrUtil::WStringWidth(t), 45);
  unittest::ExpectTrue(t.find(L"Ctrl-X send | Esc cancel") != std::wstring::npos);
  unittest::ExpectTrue(t.find(L"...") != std::wstring::npos);
}

static void TestWide()
{
  // wide (cjk), combining (vietnamese) and emoji names stay within the width at every size
  const char* names[] = { "\xe4\xbd\xa0\xe5\xa5\xbd\xe4\xb8\x96\xe7\x95\x8c\xe4\xbd\xa0\xe5\xa5\xbd",
                          "Nguy\xe1\xbb\x85n V\xc4\x83n A", "e\xcc\x81 e\xcc\x81 e\xcc\x81 e\xcc\x81",
                          "\xf0\x9f\x98\x80\xf0\x9f\x98\x80\xf0\x9f\x98\x80\xf0\x9f\x98\x80" };
  for (const char* name : names)
  {
    for (int w = 1; w < 80; ++w)
    {
      unittest::ExpectEqual(int, StrUtil::WStringWidth(BuildEntryTitle(EntryMode::Reply, name, hint, "Esc", w)), w);
    }
  }
}

int main()
{
  int rv = 0;

  setlocale(LC_ALL, "");

  try
  {
    TestModes();
    TestSanitize();
    TestWidth();

    // wide character width depends on a utf-8 capable locale being present
    if (wcwidth(L'你') == 2)
    {
      TestWide();
    }
    else
    {
      std::cout << "skipping wide character tests, no utf-8 locale" << std::endl;
    }
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
