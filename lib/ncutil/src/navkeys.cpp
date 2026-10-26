// navkeys.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "navkeys.h"

#include <algorithm>
#include <cctype>
#include <sstream>

#include "strutil.h"

static const char* s_Defaults =
  "map list,history j down\n"
  "map list,history k up\n"
  "map list,history G last\n"
  "map list,history <C-d> halfdown\n"
  "map list,history <C-u> halfup\n"
  "map list,history i focusentry\n"
  "map list,history / search\n"
  "map list,history gg first\n"
  "map list l open\n"
  "map list n foldernext\n"
  "map list b folderprev\n"
  "map list u unreadonly\n"
  "map list z togglegroup\n"
  "map history h focuslist\n"
  "map history n findnext\n"
  "map history r reply\n"
  "map history e edit\n"
  "map history y copy\n"
  "map history x delete\n"
  "map all <Pane>a selectcontact\n"
  "map history,entry <Pane>h focuslist\n"
  "map list <Pane>l focushistory\n"
  "map history <Pane>j focusentry\n"
  "map entry <Pane>k focushistory\n";

static const struct { const char* name; NavAction action; } s_Actions[] = {
  { "none", NavAction::None },
  { "focusnext", NavAction::FocusNext },
  { "focusprev", NavAction::FocusPrev },
  { "focusentry", NavAction::FocusEntry },
  { "focuslist", NavAction::FocusList },
  { "focushistory", NavAction::FocusHistory },
  { "up", NavAction::Up },
  { "down", NavAction::Down },
  { "first", NavAction::First },
  { "last", NavAction::Last },
  { "halfup", NavAction::HalfUp },
  { "halfdown", NavAction::HalfDown },
  { "open", NavAction::Open },
  { "reply", NavAction::Reply },
  { "edit", NavAction::Edit },
  { "copy", NavAction::Copy },
  { "delete", NavAction::Delete },
  { "search", NavAction::Search },
  { "findnext", NavAction::FindNext },
  { "unreadonly", NavAction::UnreadOnly },
  { "togglegroup", NavAction::ToggleGroup },
  { "foldernext", NavAction::FolderNext },
  { "folderprev", NavAction::FolderPrev },
  { "selectcontact", NavAction::SelectContact },
};

static std::string Lower(std::string p_Str)
{
  std::transform(p_Str.begin(), p_Str.end(), p_Str.begin(), [](unsigned char c) { return std::tolower(c); });
  return p_Str;
}

static unsigned ParseFocus(const std::string& p_Str)
{
  unsigned mask = 0;
  std::istringstream ss(Lower(p_Str));
  std::string name;
  while (std::getline(ss, name, ','))
  {
    if (name == "list") mask |= 1;
    else if (name == "history") mask |= 2;
    else if (name == "entry") mask |= 4;
    else if (name == "all") mask |= 7;
    else return 0;
  }
  return mask;
}

static bool ParseKeys(const std::string& p_Str, std::vector<wint_t>& p_Keys)
{
  const std::wstring w = StrUtil::ToWString(p_Str);
  for (size_t i = 0; i < w.size(); ++i)
  {
    if (w[i] != L'<')
    {
      p_Keys.push_back(w[i]);
      continue;
    }

    const size_t end = w.find(L'>', i);
    if (end == std::wstring::npos) return false;
    const std::string name = Lower(StrUtil::ToString(w.substr(i + 1, end - i - 1)));
    if (name == "pane") p_Keys.push_back(NavKeyMap::Pane);
    else if (name == "space") p_Keys.push_back(L' ');
    else if (name == "lt") p_Keys.push_back(L'<');
    else if (name == "cr" || name == "enter") p_Keys.push_back(10);
    else if (name == "tab") p_Keys.push_back(9);
    else if (name == "esc") p_Keys.push_back(27);
    else if ((name.size() == 3) && (name[0] == 'c') && (name[1] == '-') && std::isalpha((unsigned char)name[2]))
      p_Keys.push_back(name[2] - 'a' + 1);
    else return false;
    i = end;
  }

  return !p_Keys.empty();
}

static bool ParseAction(const std::string& p_Str, NavAction& p_Action)
{
  const std::string name = Lower(p_Str);
  for (const auto& entry : s_Actions)
  {
    if (name == entry.name)
    {
      p_Action = entry.action;
      return true;
    }
  }

  return false;
}

NavKeyMap::NavKeyMap()
{
  Load(s_Defaults, true);
}

std::vector<std::string> NavKeyMap::Load(const std::string& p_Text, bool p_VimOnly)
{
  std::vector<std::string> errors;
  std::istringstream in(p_Text);
  std::string line;
  int lineNo = 0;
  while (std::getline(in, line))
  {
    ++lineNo;
    std::istringstream ls(line);
    std::vector<std::string> tok;
    std::string word;
    while (ls >> word) tok.push_back(word);
    if (tok.empty() || (tok[0][0] == '#')) continue;

    if ((tok[0] == "mapclear") && (tok.size() == 1))
    {
      m_Bindings.clear();
      continue;
    }

    const bool isMap = (tok[0] == "map") && (tok.size() == 4);
    const bool isUnmap = (tok[0] == "unmap") && (tok.size() == 3);
    const std::string where = "line " + std::to_string(lineNo) + ": ";
    if (!isMap && !isUnmap)
    {
      errors.push_back(where + "expected 'map <focus> <keys> <action>', 'unmap <focus> <keys>' or 'mapclear'");
      continue;
    }

    const unsigned mask = ParseFocus(tok[1]);
    if (mask == 0)
    {
      errors.push_back(where + "unknown focus '" + tok[1] + "' (list, history, entry, all)");
      continue;
    }

    std::vector<wint_t> keys;
    if (!ParseKeys(tok[2], keys))
    {
      errors.push_back(where + "bad key sequence '" + tok[2] + "'");
      continue;
    }

    NavAction action = NavAction::None;
    if (isMap && !ParseAction(tok[3], action))
    {
      errors.push_back(where + "unknown action '" + tok[3] + "'");
      continue;
    }

    for (NavBinding& b : m_Bindings)
    {
      if (b.keys == keys) b.focusMask &= ~mask;
    }

    m_Bindings.erase(std::remove_if(m_Bindings.begin(), m_Bindings.end(),
                                    [](const NavBinding& b) { return b.focusMask == 0; }),
                     m_Bindings.end());
    if (isMap) m_Bindings.push_back({ mask, keys, action, p_VimOnly });
  }

  return errors;
}

const NavBinding* NavKeyMap::Find(NavFocus p_Focus, const std::vector<wint_t>& p_Seq, const NavKeys& p_Keys,
                                  bool p_Vim, bool& p_Prefix) const
{
  const unsigned bit = 1u << static_cast<unsigned>(p_Focus);
  p_Prefix = false;
  for (const NavBinding& b : m_Bindings)
  {
    if (!(b.focusMask & bit) || (b.vimOnly && !p_Vim) || (b.keys.size() < p_Seq.size())) continue;

    bool match = true;
    for (size_t i = 0; match && (i < p_Seq.size()); ++i)
    {
      match = (b.keys[i] == p_Seq[i]) || ((b.keys[i] == Pane) && (p_Seq[i] == p_Keys.panePrefix));
    }

    if (!match) continue;
    if (b.keys.size() == p_Seq.size()) return &b;
    p_Prefix = true;
  }

  return nullptr;
}

NavAction NavKeyResolver::Resolve(NavFocus p_Focus, wint_t p_Key, const NavKeys& p_Keys, bool p_Vim)
{
  std::vector<wint_t> seq;
  seq.swap(m_Seq);
  // a failed Ctrl-W sequence is swallowed so the stray key cannot type into the entry
  const bool swallow = !seq.empty() && (seq[0] == p_Keys.panePrefix);
  seq.push_back(p_Key);
  for (;;)
  {
    bool prefix = false;
    const NavBinding* binding = m_Map.Find(p_Focus, seq, p_Keys, p_Vim, prefix);
    if (binding) return binding->action;
    if (prefix)
    {
      m_Seq = seq;
      return NavAction::Pending;
    }

    if (seq.size() == 1) break;
    if (swallow) return NavAction::Pending;
    seq.assign(1, p_Key);
  }

  if (p_Key == p_Keys.focusNext) return NavAction::FocusNext;
  if (p_Key == p_Keys.focusPrev) return NavAction::FocusPrev;
  if (p_Key == p_Keys.focusEntry) return NavAction::FocusEntry;
  if (p_Focus == NavFocus::Entry) return NavAction::None;

  if (p_Key == p_Keys.up) return NavAction::Up;
  if (p_Key == p_Keys.down) return NavAction::Down;
  if (p_Key == p_Keys.home) return NavAction::First;
  if (p_Key == p_Keys.end) return NavAction::Last;
  if (p_Key == p_Keys.ok) return (p_Focus == NavFocus::List) ? NavAction::Open : NavAction::Reply;
  return NavAction::None;
}
