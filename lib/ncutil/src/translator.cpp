// translator.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "translator.h"

#include <atomic>
#include <cctype>
#include <condition_variable>
#include <deque>
#include <map>
#include <mutex>
#include <set>
#include <thread>

#include <sys/wait.h>
#include <unistd.h>

#include "fileutil.h"
#include "strutil.h"
#include "sysutil.h"

namespace
{
  struct Entry
  {
    Translator::State state;
    std::string text;
  };

  const size_t MaxTextLen = 2000;
  const size_t MaxCacheSize = 500;

  std::mutex s_Mutex;
  std::condition_variable s_Cv;
  std::thread s_Thread;
  bool s_Running = false;
  bool s_Disabled = false;
  std::string s_Cmd;
  std::string s_Target;
  std::function<void()> s_OnUpdate;
  std::map<std::string, Entry> s_Cache;
  std::deque<std::string> s_Queue;
  std::set<std::string> s_Queued;
  std::atomic<int> s_Seq(0);

  // ponytail: p_Cmd is used as a shell prefix like the other *_command configs; the text only ever goes via a file
  Translator::State Run(const std::string& p_Cmd, const std::string& p_Target, const std::string& p_Text,
                        std::string& p_Out, bool& p_Missing)
  {
    p_Missing = false;
    p_Out.clear();
    if (p_Cmd.empty() || p_Text.empty() || (p_Text.size() > MaxTextLen) || !Translator::IsValidLang(p_Target))
    {
      return Translator::State::Failed;
    }

    const std::string base = FileUtil::GetTempDir() + "/translate-" + std::to_string(getpid()) + "-" +
      std::to_string(++s_Seq);
    const std::string inPath = base + ".in";
    const std::string outPath = base + ".out";
    FileUtil::WriteFile(inPath, p_Text);
    const std::string cmd = p_Cmd + " -b -t " + p_Target + " -i '" + StrUtil::EscapeSingleQuote(inPath) + "' >'" +
      StrUtil::EscapeSingleQuote(outPath) + "' 2>/dev/null </dev/null";
    int status = SysUtil::System(cmd); // non-const: macOS WEXITSTATUS casts away const (-Wcast-qual)
    p_Out = FileUtil::ReadFile(outPath);
    FileUtil::RmFile(inPath);
    FileUtil::RmFile(outPath);

    if (WIFEXITED(status) && (WEXITSTATUS(status) == 127))
    {
      p_Missing = true;
      p_Out.clear();
      return Translator::State::Failed;
    }

    StrUtil::Trim(p_Out);
    if ((status != 0) || p_Out.empty())
    {
      p_Out.clear();
      return Translator::State::Failed;
    }

    std::string src = StrUtil::ToLower(p_Text);
    std::string dst = StrUtil::ToLower(p_Out);
    StrUtil::Trim(src);
    StrUtil::Trim(dst);
    return (src == dst) ? Translator::State::Unchanged : Translator::State::Ready;
  }

  // caller holds s_Mutex
  void Store(const std::string& p_Text, Translator::State p_State, const std::string& p_Out)
  {
    if (s_Cache.size() >= MaxCacheSize) s_Cache.clear();
    s_Cache[p_Text] = Entry{ p_State, (p_State == Translator::State::Ready) ? p_Out : std::string() };
  }

  void Worker()
  {
    std::unique_lock<std::mutex> lock(s_Mutex);
    while (s_Running)
    {
      if (s_Queue.empty())
      {
        s_Cv.wait(lock);
        continue;
      }

      const std::string text = s_Queue.front();
      s_Queue.pop_front();
      const std::string cmd = s_Cmd;
      const std::string target = s_Target;
      const std::function<void()> onUpdate = s_OnUpdate;
      lock.unlock();

      std::string out;
      bool missing = false;
      const Translator::State state = Run(cmd, target, text, out, missing);

      lock.lock();
      s_Queued.erase(text);
      if (missing) s_Disabled = true;
      Store(text, state, out);
      lock.unlock();

      if (onUpdate) onUpdate(); // without s_Mutex: onUpdate takes the UI model mutex
      lock.lock();
    }
  }
}

void Translator::Init(const std::string& p_Cmd, const std::string& p_Target, std::function<void()> p_OnUpdate)
{
  Cleanup();
  {
    std::unique_lock<std::mutex> lock(s_Mutex);
    s_Cmd = p_Cmd;
    s_Target = p_Target;
    s_OnUpdate = p_OnUpdate;
    s_Disabled = false;
    s_Running = true;
  }

  s_Thread = std::thread(Worker);
}

void Translator::Cleanup()
{
  {
    std::unique_lock<std::mutex> lock(s_Mutex);
    s_Running = false;
  }

  s_Cv.notify_all();
  if (s_Thread.joinable()) s_Thread.join();

  std::unique_lock<std::mutex> lock(s_Mutex);
  s_Cache.clear();
  s_Queue.clear();
  s_Queued.clear();
  s_OnUpdate = nullptr;
}

Translator::State Translator::Get(const std::string& p_Text, std::string& p_Translated)
{
  std::unique_lock<std::mutex> lock(s_Mutex);
  if (!s_Running || s_Disabled || p_Text.empty() || (p_Text.size() > MaxTextLen)) return State::Failed;

  auto it = s_Cache.find(p_Text);
  if (it != s_Cache.end())
  {
    p_Translated = it->second.text;
    return it->second.state;
  }

  // ponytail: LIFO queue capped at 64, newest (visible) first and stale scrolled-past requests dropped; no re-prioritising
  if (s_Queued.insert(p_Text).second)
  {
    s_Queue.push_front(p_Text);
    if (s_Queue.size() > 64)
    {
      s_Queued.erase(s_Queue.back());
      s_Queue.pop_back();
    }

    s_Cv.notify_one();
  }

  return State::Pending;
}

bool Translator::TranslateNow(const std::string& p_Text, std::string& p_Translated, std::string& p_Error)
{
  std::string cmd;
  std::string target;
  {
    std::unique_lock<std::mutex> lock(s_Mutex);
    if (!s_Running)
    {
      p_Error = "translation is not initialised";
      return false;
    }

    auto it = s_Cache.find(p_Text);
    if ((it != s_Cache.end()) && (it->second.state != State::Failed))
    {
      p_Translated = (it->second.state == State::Ready) ? it->second.text : p_Text;
      return true;
    }

    cmd = s_Cmd;
    target = s_Target;
  }

  if (!IsValidLang(target))
  {
    p_Error = "invalid translate_target: " + target;
    return false;
  }

  if (p_Text.size() > MaxTextLen)
  {
    p_Error = "message is too long to translate";
    return false;
  }

  std::string out;
  bool missing = false;
  const State state = Run(cmd, target, p_Text, out, missing);
  std::unique_lock<std::mutex> lock(s_Mutex);
  if (missing)
  {
    s_Disabled = true;
    p_Error = "translate-shell not found: install it (brew install translate-shell) or set translate_command";
    return false;
  }

  if (state == State::Failed)
  {
    p_Error = "translation failed (check network and translate_command)";
    return false;
  }

  s_Disabled = false;
  Store(p_Text, state, out);
  p_Translated = (state == State::Ready) ? out : p_Text;
  return true;
}

bool Translator::IsValidLang(const std::string& p_Lang)
{
  // 2-3 letters, optionally followed by '-' or '_' and 2-8 letters/digits (en, fil, pt-BR, zh_CN)
  size_t i = 0;
  while ((i < p_Lang.size()) && std::isalpha(static_cast<unsigned char>(p_Lang[i]))) ++i;
  if ((i < 2) || (i > 3)) return false;
  if (i == p_Lang.size()) return true;
  if ((p_Lang[i] != '-') && (p_Lang[i] != '_')) return false;

  const size_t rest = p_Lang.size() - i - 1;
  if ((rest < 2) || (rest > 8)) return false;
  for (size_t j = i + 1; j < p_Lang.size(); ++j)
  {
    if (!std::isalnum(static_cast<unsigned char>(p_Lang[j]))) return false;
  }

  return true;
}
