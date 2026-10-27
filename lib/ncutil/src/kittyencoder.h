// kittyencoder.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <string>
#include <vector>

class KittyEncoder
{
public:
  static int GetDiacriticCount();
  static wchar_t Diacritic(int p_Index);
  static bool CellChars(int p_Row, int p_Col, wchar_t p_Out[4]);
  static std::string Base64(const std::string& p_Data);
  static std::vector<std::string> EncodeUpload(int p_Id, int p_Cols, int p_Rows, const std::string& p_Png);
  static std::string EncodeUploadFile(int p_Id, int p_Cols, int p_Rows, const std::string& p_PngPath);
  static std::string EncodeDelete(int p_Id);
  static std::string WrapTmux(const std::string& p_Seq);
  static bool IsSupportedTerm(bool p_HasKittyWindowId, bool p_HasGhosttyResourcesDir, const std::string& p_Term);
};
