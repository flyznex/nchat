// kittyencoder.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "kittyencoder.h"

static const wchar_t s_Diacritics[] =
{
#include "kittydiacritics.inc"
};

static const int s_DiacriticCount = static_cast<int>(sizeof(s_Diacritics) / sizeof(s_Diacritics[0]));

int KittyEncoder::GetDiacriticCount()
{
  return s_DiacriticCount;
}

wchar_t KittyEncoder::Diacritic(int p_Index)
{
  if ((p_Index < 0) || (p_Index >= s_DiacriticCount)) return 0;
  return s_Diacritics[p_Index];
}

bool KittyEncoder::CellChars(int p_Row, int p_Col, wchar_t p_Out[4])
{
  const wchar_t rowDia = Diacritic(p_Row);
  const wchar_t colDia = Diacritic(p_Col);
  if ((rowDia == 0) || (colDia == 0))
  {
    p_Out[0] = L' ';
    p_Out[1] = 0;
    p_Out[2] = 0;
    p_Out[3] = 0;
    return false;
  }

  p_Out[0] = 0x10EEEE;
  p_Out[1] = rowDia;
  p_Out[2] = colDia;
  p_Out[3] = 0;
  return true;
}

std::string KittyEncoder::Base64(const std::string& p_Data)
{
  static const char table[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
  const size_t n = p_Data.size();
  std::string out;
  out.reserve(((n + 2) / 3) * 4);
  for (size_t i = 0; i < n; i += 3)
  {
    unsigned v = static_cast<unsigned>(static_cast<unsigned char>(p_Data[i])) << 16;
    if (i + 1 < n) v |= static_cast<unsigned>(static_cast<unsigned char>(p_Data[i + 1])) << 8;
    if (i + 2 < n) v |= static_cast<unsigned>(static_cast<unsigned char>(p_Data[i + 2]));
    out += table[(v >> 18) & 63];
    out += table[(v >> 12) & 63];
    out += (i + 1 < n) ? table[(v >> 6) & 63] : '=';
    out += (i + 2 < n) ? table[v & 63] : '=';
  }

  return out;
}

std::vector<std::string> KittyEncoder::EncodeUpload(int p_Id, int p_Cols, int p_Rows, const std::string& p_Png)
{
  static const size_t chunkSize = 4096;
  const std::string b64 = Base64(p_Png);

  std::vector<std::string> seqs;
  size_t pos = 0;
  do
  {
    const std::string chunk = b64.substr(pos, chunkSize);
    pos += chunk.size();
    const bool more = (pos < b64.size());

    std::string ctrl;
    if (seqs.empty())
    {
      ctrl = "a=T,U=1,f=100,i=" + std::to_string(p_Id) + ",c=" + std::to_string(p_Cols) +
        ",r=" + std::to_string(p_Rows) + ",q=2,";
    }

    ctrl += more ? "m=1" : "m=0";
    seqs.push_back("\033_G" + ctrl + ";" + chunk + "\033\\");
  } while (pos < b64.size());

  return seqs;
}

std::string KittyEncoder::EncodeUploadFile(int p_Id, int p_Cols, int p_Rows, const std::string& p_PngPath)
{
  return "\033_Ga=T,U=1,f=100,t=f,i=" + std::to_string(p_Id) + ",c=" + std::to_string(p_Cols) +
    ",r=" + std::to_string(p_Rows) + ",q=2;" + Base64(p_PngPath) + "\033\\";
}

std::string KittyEncoder::EncodeDelete(int p_Id)
{
  return "\033_Ga=d,d=I,i=" + std::to_string(p_Id) + ",q=2\033\\";
}

std::string KittyEncoder::WrapTmux(const std::string& p_Seq)
{
  std::string out = "\033Ptmux;";
  for (const char c : p_Seq)
  {
    if (c == '\033') out += '\033';
    out += c;
  }

  out += "\033\\";
  return out;
}

bool KittyEncoder::IsSupportedTerm(bool p_HasKittyWindowId, bool p_HasGhosttyResourcesDir, const std::string& p_Term)
{
  if (p_HasKittyWindowId || p_HasGhosttyResourcesDir) return true;
  return (p_Term.find("kitty") != std::string::npos) || (p_Term.find("ghostty") != std::string::npos);
}
