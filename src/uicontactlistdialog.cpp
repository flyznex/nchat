// uicontactlistdialog.cpp
//
// Copyright (c) 2019-2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "uicontactlistdialog.h"

#include <algorithm>
#include <cctype>

#include "apputil.h"
#include "log.h"
#include "uimodel.h"
#include "strutil.h"

// "@username" (5-32 chars) or a phone number (7-15 digits, optional +, spaces and dashes ignored), else empty
static std::string GetLookupQuery(const std::string& p_Filter)
{
  if ((p_Filter.size() >= 6) && (p_Filter.size() <= 33) && (p_Filter[0] == '@') &&
      std::all_of(p_Filter.begin() + 1, p_Filter.end(),
                  [](unsigned char c) { return std::isalnum(c) || (c == '_'); }))
  {
    return p_Filter;
  }

  std::string digits;
  for (size_t i = 0; i < p_Filter.size(); ++i)
  {
    const unsigned char c = p_Filter[i];
    if (std::isdigit(c)) digits += c;
    else if (!((c == '+' && i == 0) || c == ' ' || c == '-')) return "";
  }

  return ((digits.size() >= 7) && (digits.size() <= 15)) ? digits : "";
}

UiContactListDialog::UiContactListDialog(const UiDialogParams& p_Params)
  : UiListDialog(p_Params, false /*p_ShadeHidden*/)
{
  static bool s_ContactsRequested = false;
  if (!s_ContactsRequested)
  {
    m_Model->RequestContacts();
    s_ContactsRequested = true;
  }

  UpdateList();
}

UiContactListDialog::~UiContactListDialog()
{
}

UiContactListItem UiContactListDialog::GetSelectedContactItem()
{
  return m_SelectedContactItem;
}

void UiContactListDialog::OnSelect()
{
  if (m_ContactListItemVec.empty()) return;

  m_SelectedContactItem = m_ContactListItemVec[m_Index];
  m_Result = true;
  m_Running = false;
}

void UiContactListDialog::OnBack()
{
}

bool UiContactListDialog::OnTimer()
{
  int64_t modelContactInfosUpdateTime = m_Model->GetContactInfosUpdateTime();
  if (m_DialogContactInfosUpdateTime != modelContactInfosUpdateTime)
  {
    UpdateList();
    return true;
  }

  return false;
}

void UiContactListDialog::UpdateList()
{
  int64_t modelContactInfosUpdateTime = m_Model->GetContactInfosUpdateTime();
  if (m_DialogContactInfosUpdateTime != modelContactInfosUpdateTime)
  {
    m_DialogContactInfosUpdateTime = modelContactInfosUpdateTime;
    m_DialogContactInfos = m_Model->GetContactInfos();
  }

  const bool emojiEnabled = m_Model->GetEmojiEnabled();

  // Get current item based on index
  UiContactListItem currentContactItem;
  if (m_Index < (int)m_ContactListItemVec.size())
  {
    currentContactItem = m_ContactListItemVec[m_Index];
  }

  m_Index = 0;
  m_Items.clear();
  m_ContactListItemVec.clear();

  // Use a local vector which is sorted before populating dialog members, which need to be in sync
  std::vector<UiContactListItem> localContactListItemVec;

  for (const auto& profileContactInfos : m_DialogContactInfos)
  {
    for (const auto& idContactInfo : profileContactInfos.second)
    {
      const std::string& profileId = profileContactInfos.first;
      const std::string& contactId = idContactInfo.first;
      if (contactId.find(':') != std::string::npos) continue; // forum topic rows are not contacts
      const std::string& name = m_Model->GetContactListName(profileId, contactId, false /*p_AllowId*/,
                                                            false /*p_AllowAlias*/);;

      if (name.empty()) continue;

      if (m_FilterStr.empty() ||
          (StrUtil::ToFold(name).find(StrUtil::ToFold(StrUtil::ToString(m_FilterStr))) != std::string::npos))
      {
        std::string displayName = name + m_Model->GetProfileSuffix(profileId);

        if (!emojiEnabled)
        {
          displayName = StrUtil::Textize(displayName);
        }

        static const bool developerMode = AppUtil::GetDeveloperMode();
        if (developerMode)
        {
          displayName += " [" + contactId + "]";
        }

        UiContactListItem contactListItem;
        contactListItem.profileId = profileId;
        contactListItem.contactId = contactId;
        contactListItem.name = displayName;

        localContactListItemVec.push_back(contactListItem);
      }
    }
  }

  std::sort(localContactListItemVec.begin(), localContactListItemVec.end(),
            [&](const UiContactListItem& lhs, const UiContactListItem& rhs) -> bool
  {
    return lhs.name < rhs.name;
  });

  const std::string lookupQuery = GetLookupQuery(StrUtil::ToString(m_FilterStr));
  if (!lookupQuery.empty())
  {
    std::vector<std::string> telegramProfileIds;
    for (const auto& profileProtocol : m_Model->GetProtocols())
    {
      if (StrUtil::GetProtocolName(profileProtocol.first) == "Telegram")
      {
        telegramProfileIds.push_back(profileProtocol.first);
      }
    }

    std::sort(telegramProfileIds.begin(), telegramProfileIds.end());
    std::vector<UiContactListItem> lookupItems;
    for (const std::string& profileId : telegramProfileIds)
    {
      UiContactListItem lookupItem;
      lookupItem.profileId = profileId;
      lookupItem.lookupQuery = lookupQuery;
      lookupItem.name = "Open chat with " + lookupQuery + m_Model->GetProfileSuffix(profileId);
      lookupItems.push_back(lookupItem);
    }

    localContactListItemVec.insert(localContactListItemVec.begin(), lookupItems.begin(), lookupItems.end());
  }

  for (const auto& contactListItem : localContactListItemVec)
  {
    m_Items.push_back(StrUtil::TrimPadWString(StrUtil::ToWString(contactListItem.name), m_W));
    m_ContactListItemVec.push_back(contactListItem);
  }

  // Set index based on current item
  if (!currentContactItem.profileId.empty() && !currentContactItem.contactId.empty())
  {
    for (auto it = m_ContactListItemVec.begin(); it != m_ContactListItemVec.end(); ++it)
    {
      if ((it->contactId == currentContactItem.contactId) &&
          (it->profileId == currentContactItem.profileId))
      {
        m_Index = std::distance(m_ContactListItemVec.begin(), it);
        break;
      }
    }
  }
}
