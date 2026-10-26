// mdstyle.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "mdstyle.h"

#include <cwctype>

namespace
{
  struct Out
  {
    std::wstring& text;
    std::vector<uint8_t>& styles;

    void Push(wchar_t p_Char, uint8_t p_Style)
    {
      text += p_Char;
      styles.push_back(p_Style);
    }
  };

  // opener must follow a non-word char and precede a non-space, closer the reverse, same paragraph only
  size_t FindClose(const std::wstring& p_In, size_t p_Open, size_t p_End)
  {
    const wchar_t delim = p_In[p_Open];
    if ((p_Open + 1 >= p_End) || std::iswspace(p_In[p_Open + 1])) return std::wstring::npos;
    if ((p_Open > 0) && std::iswalnum(p_In[p_Open - 1])) return std::wstring::npos;

    for (size_t j = p_Open + 1; j < p_End; ++j)
    {
      const wchar_t wc = p_In[j];
      if (wc == L'\\')
      {
        ++j;
        continue;
      }

      if ((wc == L'\n') && (j + 1 < p_End) && (p_In[j + 1] == L'\n')) return std::wstring::npos;

      if ((wc == delim) && (j > p_Open + 1) && !std::iswspace(p_In[j - 1]) &&
          ((j + 1 >= p_End) || !std::iswalnum(p_In[j + 1])))
      {
        return j;
      }
    }

    return std::wstring::npos;
  }

  void ParseRange(const std::wstring& p_In, size_t p_Begin, size_t p_End, uint8_t p_Style, Out& p_Out)
  {
    size_t i = p_Begin;
    while (i < p_End)
    {
      const wchar_t wc = p_In[i];

      // telegram escapes markup chars in plain text as \* \_ \` \[
      if ((wc == L'\\') && (i + 1 < p_End) && (std::wstring(L"*_`[").find(p_In[i + 1]) != std::wstring::npos))
      {
        p_Out.Push(p_In[i + 1], p_Style);
        i += 2;
        continue;
      }

      // urls are copied as is, they often contain _ and *
      if ((wc == L'h') && ((i == p_Begin) || !std::iswalnum(p_In[i - 1])) &&
          ((p_In.compare(i, 7, L"http://") == 0) || (p_In.compare(i, 8, L"https://") == 0)))
      {
        while ((i < p_End) && !std::iswspace(p_In[i]))
        {
          p_Out.Push(p_In[i++], p_Style);
        }
        continue;
      }

      if (wc == L'`')
      {
        size_t close = std::wstring::npos;
        size_t contentBegin = 0;
        size_t next = 0;
        if (p_In.compare(i, 3, L"```") == 0)
        {
          const size_t found = p_In.find(L"```", i + 3);
          if ((found != std::wstring::npos) && (found < p_End) && (found > i + 3))
          {
            close = found;
            contentBegin = i + 3;
            next = found + 3;
          }
        }
        else
        {
          const size_t found = p_In.find(L'`', i + 1);
          if ((found != std::wstring::npos) && (found < p_End) && (found > i + 1) &&
              (p_In.find(L'\n', i + 1) >= found))
          {
            close = found;
            contentBegin = i + 1;
            next = found + 1;
          }
        }

        if (close != std::wstring::npos)
        {
          size_t contentEnd = close;
          if ((p_In[contentBegin] == L'\n') && (contentBegin + 1 < contentEnd)) ++contentBegin;
          if ((contentEnd > contentBegin + 1) && (p_In[contentEnd - 1] == L'\n')) --contentEnd;
          for (size_t k = contentBegin; k < contentEnd; ++k)
          {
            p_Out.Push(p_In[k], p_Style | MdStyleCode);
          }

          i = next;
          continue;
        }
      }
      else if ((wc == L'*') || (wc == L'_'))
      {
        const uint8_t bit = (wc == L'*') ? MdStyleBold : MdStyleItalic;
        if (!(p_Style & bit))
        {
          const size_t close = FindClose(p_In, i, p_End);
          if (close != std::wstring::npos)
          {
            ParseRange(p_In, i + 1, close, p_Style | bit, p_Out);
            i = close + 1;
            continue;
          }
        }
      }

      p_Out.Push(wc, p_Style);
      ++i;
    }
  }
}

void MdStyleUtil::Parse(const std::wstring& p_In, std::wstring& p_Out, std::vector<uint8_t>& p_Styles)
{
  p_Out.clear();
  p_Styles.clear();
  Out out{ p_Out, p_Styles };
  ParseRange(p_In, 0, p_In.size(), MdStyleNone, out);
}

std::vector<std::vector<uint8_t>> MdStyleUtil::WrapStyles(const std::wstring& p_Text,
                                                          const std::vector<uint8_t>& p_Styles,
                                                          const std::vector<std::wstring>& p_Lines)
{
  std::vector<std::vector<uint8_t>> lineStyles;
  lineStyles.reserve(p_Lines.size());

  size_t pos = 0;
  for (const std::wstring& line : p_Lines)
  {
    std::vector<uint8_t> styles;
    styles.reserve(line.size());
    for (size_t i = 0; i < line.size(); ++i)
    {
      const wchar_t wc = line[i];

      // skip what wrapping dropped (break spaces, newlines); a tab matches the spaces it was expanded to
      while ((pos < p_Text.size()) && (p_Text[pos] != wc) && !((p_Text[pos] == L'\t') && (wc == L' ')))
      {
        ++pos;
      }

      if (pos >= p_Text.size())
      {
        styles.push_back(MdStyleNone);
        continue;
      }

      styles.push_back(p_Styles[pos]);
      const bool inTab = (p_Text[pos] == L'\t');
      if (!inTab || (i + 1 >= line.size()) || (line[i + 1] != L' '))
      {
        ++pos;
      }
    }

    lineStyles.push_back(std::move(styles));
  }

  return lineStyles;
}
