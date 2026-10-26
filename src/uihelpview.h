// uihelpview.h
//
// Copyright (c) 2019-2024 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <initializer_list>
#include <string>
#include <vector>

#include "navkeys.h"
#include "uiviewbase.h"

class UiHelpView : public UiViewBase
{
public:
  UiHelpView(const UiViewParams& p_Params);

  virtual void Draw();
  static std::string GetKeyDisplay(const std::string& p_Func);

private:
  static std::vector<std::wstring> GetHelpViews(const int p_MaxW, const std::vector<std::wstring>& p_HelpItems,
                                                const std::wstring& p_OtherHelpItem);
  static void AppendHelpItem(const std::string& p_Func, const std::string& p_Desc,
                             std::vector<std::wstring>& p_HelpItems);
  static void AppendLiteralHelpItem(const std::string& p_Key, const std::string& p_Desc,
                                    std::vector<std::wstring>& p_HelpItems);
  // Appends "<keys> <desc>" from the nav bindings (actions joined by '/'), false if none is bound.
  static bool AppendNavHelpItem(const NavKeyMap& p_Map, NavFocus p_Focus, bool p_Vim,
                                std::initializer_list<NavAction> p_Actions, const std::string& p_Desc,
                                std::vector<std::wstring>& p_HelpItems);
};
