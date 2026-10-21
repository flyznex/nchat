// entrytitle.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "entrytitle.h"

#include "strutil.h"

static std::wstring Compose(EntryMode p_Mode, const std::wstring& p_Who, const std::string& p_Hint,
                            const std::string& p_CancelKey)
{
  const std::wstring hint = StrUtil::ToWString(p_Hint);
  const std::wstring cancel = StrUtil::ToWString(p_CancelKey + " cancel");
  switch (p_Mode)
  {
    case EntryMode::Edit:
      return L"Editing message | " + hint + L" | " + cancel;
    case EntryMode::Reply:
      return L"Replying to " + p_Who + L" | " + hint + L" | " + cancel;
    case EntryMode::Compose:
    default:
      return L"Message | " + hint;
  }
}

std::wstring BuildEntryTitle(EntryMode p_Mode, const std::string& p_Who, const std::string& p_Hint,
                             const std::string& p_CancelKey, int p_MaxW)
{
  if (p_MaxW <= 0) return L"";

  // control chars are single bytes in utf-8, so a bytewise replace keeps multibyte sequences intact
  std::string who = p_Who;
  for (char& c : who)
  {
    if (static_cast<unsigned char>(c) < 0x20) c = ' ';
  }

  std::wstring wwho = StrUtil::ToWString(who);
  if (wwho.empty()) wwho = L"message";

  std::wstring title = Compose(p_Mode, wwho, p_Hint, p_CancelKey);
  const int over = StrUtil::WStringWidth(title) - p_MaxW;
  if ((p_Mode == EntryMode::Reply) && (over > 0))
  {
    const int whoW = StrUtil::WStringWidth(wwho);
    if (whoW - over >= 4) // keep at least one char plus "..."
    {
      wwho = StrUtil::TrimPadWString(wwho, whoW - over - 3) + L"...";
      title = Compose(p_Mode, wwho, p_Hint, p_CancelKey);
    }
  }

  // ponytail: first call truncates (a wide char may leave one column spare), second pads to exact width
  return StrUtil::TrimPadWString(StrUtil::TrimPadWString(title, p_MaxW), p_MaxW);
}
