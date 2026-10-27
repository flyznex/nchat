// test015.cpp - translate-shell wrapper with a fake translate command
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <atomic>
#include <chrono>
#include <iostream>
#include <string>
#include <thread>

#include "fileutil.h"
#include "translator.h"

#include "unittest.h"

static const std::string g_Dir = "/tmp/nchat-test015";
static std::string g_Cmd;

static void SetupFake()
{
  FileUtil::SetApplicationDir(g_Dir);
  FileUtil::MkDir(FileUtil::GetTempDir());
  const std::string script = g_Dir + "/fake.sh";
  // args: -b -t <lang> -i <file>; counts calls, "same*" is echoed back, "fail*" exits 1
  FileUtil::WriteFile(script,
                      "#!/bin/sh\n"
                      "echo x >> \"$(dirname \"$0\")/calls\"\n"
                      "text=$(cat \"$5\")\n"
                      "case \"$text\" in\n"
                      "  fail*) exit 1;;\n"
                      "  same*) printf '%s' \"$text\";;\n"
                      "  *) printf '[%s] %s' \"$3\" \"$text\";;\n"
                      "esac\n");
  g_Cmd = "sh '" + script + "'";
  FileUtil::WriteFile(g_Dir + "/calls", "");
}

static int Calls()
{
  const std::string calls = FileUtil::ReadFile(g_Dir + "/calls");
  int n = 0;
  for (char c : calls) if (c == '\n') ++n;
  return n;
}

static Translator::State WaitDone(const std::string& p_Text, std::string& p_Out)
{
  for (int i = 0; i < 250; ++i)
  {
    const Translator::State st = Translator::Get(p_Text, p_Out);
    if (st != Translator::State::Pending) return st;
    std::this_thread::sleep_for(std::chrono::milliseconds(20));
  }

  return Translator::State::Pending;
}

static void TestIsValidLang()
{
  unittest::ExpectTrue(Translator::IsValidLang("en"));
  unittest::ExpectTrue(Translator::IsValidLang("vi"));
  unittest::ExpectTrue(Translator::IsValidLang("fil"));
  unittest::ExpectTrue(Translator::IsValidLang("pt-BR"));
  unittest::ExpectTrue(Translator::IsValidLang("zh_CN"));
  unittest::ExpectFalse(Translator::IsValidLang(""));
  unittest::ExpectFalse(Translator::IsValidLang("e"));
  unittest::ExpectFalse(Translator::IsValidLang("en;rm"));
  unittest::ExpectFalse(Translator::IsValidLang("en'x"));
  unittest::ExpectFalse(Translator::IsValidLang("toolong-xx"));
  unittest::ExpectFalse(Translator::IsValidLang("en-"));
}

static void TestTranslateNowAndCache()
{
  Translator::Init(g_Cmd, "vi", nullptr);
  std::string out;
  std::string err;
  unittest::ExpectTrue(Translator::TranslateNow("hello", out, err));
  unittest::ExpectEqual(std::string, out, std::string("[vi] hello"));
  const int callsAfterFirst = Calls();
  unittest::ExpectTrue(Translator::TranslateNow("hello", out, err));
  unittest::ExpectEqual(std::string, out, std::string("[vi] hello"));
  unittest::ExpectEqual(int, Calls(), callsAfterFirst);
  Translator::Cleanup();
}

static void TestGetAsync()
{
  std::atomic<int> updates(0);
  Translator::Init(g_Cmd, "vi", [&updates]() { ++updates; });
  std::string out;
  unittest::ExpectTrue(Translator::Get("good morning", out) == Translator::State::Pending);
  unittest::ExpectTrue(WaitDone("good morning", out) == Translator::State::Ready);
  unittest::ExpectEqual(std::string, out, std::string("[vi] good morning"));
  unittest::ExpectTrue(updates.load() >= 1);
  unittest::ExpectTrue(Translator::Get("good morning", out) == Translator::State::Ready);

  unittest::ExpectTrue(Translator::Get("same text", out) == Translator::State::Pending);
  unittest::ExpectTrue(WaitDone("same text", out) == Translator::State::Unchanged);

  unittest::ExpectTrue(Translator::Get("fail now", out) == Translator::State::Pending);
  unittest::ExpectTrue(WaitDone("fail now", out) == Translator::State::Failed);
  std::string err;
  unittest::ExpectFalse(Translator::TranslateNow("fail now", out, err));
  unittest::ExpectFalse(err.empty());
  Translator::Cleanup();
}

static void TestLimitsAndInvalid()
{
  Translator::Init(g_Cmd, "vi", nullptr);
  std::string out;
  std::string err;
  const std::string longText(2001, 'a');
  unittest::ExpectTrue(Translator::Get(longText, out) == Translator::State::Failed);
  unittest::ExpectFalse(Translator::TranslateNow(longText, out, err));
  unittest::ExpectTrue(Translator::Get("", out) == Translator::State::Failed);
  Translator::Cleanup();

  Translator::Init(g_Cmd, "e;x", nullptr);
  unittest::ExpectFalse(Translator::TranslateNow("hello", out, err));
  Translator::Cleanup();
}

static void TestMissingCommand()
{
  Translator::Init("/nonexistent/trans", "vi", nullptr);
  std::string out;
  std::string err;
  unittest::ExpectFalse(Translator::TranslateNow("hello", out, err));
  unittest::ExpectTrue(err.find("not found") != std::string::npos);
  // disabled for the session: no more queueing
  unittest::ExpectTrue(Translator::Get("hello again", out) == Translator::State::Failed);
  Translator::Cleanup();
}

int main()
{
  int rv = 0;

  try
  {
    SetupFake();
    TestIsValidLang();
    TestTranslateNowAndCache();
    TestGetAsync();
    TestLimitsAndInvalid();
    TestMissingCommand();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
