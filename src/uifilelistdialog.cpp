// uifilelistdialog.cpp
//
// Copyright (c) 2019-2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "uifilelistdialog.h"

#include <cstdlib>

#include "fileutil.h"
#include "strutil.h"
#include "uimodel.h"

UiFileListDialog::UiFileListDialog(const UiDialogParams& p_Params, const std::string& p_CurrentDir)
  : UiListDialog(p_Params, true /*p_ShadeHidden*/)
  , m_CurrentDir(p_CurrentDir)
{
  m_Model->SetFileListDialogActive(true);
  m_DirEntrys = FileUtil::ListPaths(m_CurrentDir);
  UpdateList();
}

UiFileListDialog::~UiFileListDialog()
{
  m_Model->SetFileListDialogActive(false);
}

std::string UiFileListDialog::GetCurrentDir()
{
  return m_CurrentDir;
}

std::string UiFileListDialog::GetSelectedPath()
{
  return m_SelectedPath;
}

void UiFileListDialog::ChangeDir(const std::string& p_Dir)
{
  m_CurrentDir = FileUtil::AbsolutePath(p_Dir);
  m_DirEntrys = FileUtil::ListPaths(m_CurrentDir);
  m_FilterStr.clear();
  UpdateList();
  UpdateFooter();
}

// a filter that looks like a path (file names never contain '/') is typed as a path instead of filtering the list
bool UiFileListDialog::IsPathInput() const
{
  const std::string str = StrUtil::ToString(m_FilterStr);
  return (str == "~") || (str.rfind("/", 0) == 0) || (str.rfind("~/", 0) == 0) || (str.rfind("./", 0) == 0) ||
         (str.rfind("../", 0) == 0);
}

std::string UiFileListDialog::ResolvePath(const std::string& p_Path) const
{
  std::string path = p_Path;
  if (path.rfind("~", 0) == 0)
  {
    const char* home = getenv("HOME");
    path = std::string(home ? home : "") + path.substr(1);
  }
  else if (path.rfind("/", 0) != 0)
  {
    path = m_CurrentDir + "/" + path;
  }

  return FileUtil::AbsolutePath(path);
}

// entries of the typed directory whose name starts with the typed name part, hidden ones only if the part starts with '.'
std::set<DirEntry, DirEntryCompare> UiFileListDialog::Candidates(std::string& p_DirPart, std::string& p_Prefix) const
{
  std::string str = StrUtil::ToString(m_FilterStr);
  if (str == "~") str = "~/";

  const size_t pos = str.rfind('/');
  p_DirPart = str.substr(0, pos + 1);
  p_Prefix = str.substr(pos + 1);

  std::set<DirEntry, DirEntryCompare> candidates;
  for (const auto& dirEntry : FileUtil::ListPaths(ResolvePath(p_DirPart)))
  {
    if (dirEntry.name.rfind(p_Prefix, 0) != 0) continue;
    if (p_Prefix.empty() && (dirEntry.name.rfind(".", 0) == 0)) continue;
    candidates.insert(dirEntry);
  }

  return candidates;
}

void UiFileListDialog::OnSelect()
{
  if (IsPathInput())
  {
    const std::string path = ResolvePath(StrUtil::ToString(m_FilterStr));
    if (FileUtil::IsDir(path))
    {
      ChangeDir(path);
    }
    else if (FileUtil::Exists(path))
    {
      m_SelectedPath = path;
      m_Result = true;
      m_Running = false;
    }

    return;
  }

  if (m_CurrentDirEntrys.size() == 0) return;

  if (m_Index < (int)m_CurrentDirEntrys.size())
  {
    const DirEntry& dirEntry = *std::next(m_CurrentDirEntrys.begin(), m_Index);
    if (dirEntry.IsDir())
    {
      ChangeDir(m_CurrentDir + "/" + dirEntry.name);
    }
    else
    {
      m_SelectedPath = FileUtil::AbsolutePath(m_CurrentDir + "/" + dirEntry.name);
      m_Result = true;
      m_Running = false;
    }
  }
}

void UiFileListDialog::OnBack()
{
  ChangeDir(m_CurrentDir + "/..");
}

void UiFileListDialog::OnTab()
{
  if (m_FilterStr.empty()) return;

  if (!IsPathInput())
  {
    m_FilterStr = L"./" + m_FilterStr;
  }

  std::string dirPart;
  std::string prefix;
  const std::set<DirEntry, DirEntryCompare> candidates = Candidates(dirPart, prefix);
  if (!candidates.empty())
  {
    std::string common = candidates.begin()->name;
    for (const auto& dirEntry : candidates)
    {
      size_t n = 0;
      while ((n < common.size()) && (n < dirEntry.name.size()) && (common[n] == dirEntry.name[n])) ++n;
      while ((n > 0) && (n < common.size()) && ((common[n] & 0xC0) == 0x80)) --n; // do not cut a UTF-8 character
      common.resize(n);
    }

    if ((candidates.size() == 1) && candidates.begin()->IsDir()) common += "/";

    m_FilterStr = StrUtil::ToWString(dirPart + common);
  }

  UpdateList();
  UpdateFooter();
}

bool UiFileListDialog::OnTimer()
{
  return false;
}

void UiFileListDialog::UpdateList()
{
  if (IsPathInput())
  {
    std::string dirPart;
    std::string prefix;
    m_CurrentDirEntrys = Candidates(dirPart, prefix);
  }
  else if (m_FilterStr.empty())
  {
    m_CurrentDirEntrys = m_DirEntrys;
  }
  else
  {
    m_CurrentDirEntrys.clear();
    for (const auto& dirEntry : m_DirEntrys)
    {
      if (StrUtil::ToFold(dirEntry.name).find(StrUtil::ToFold(StrUtil::ToString(m_FilterStr))) != std::string::npos)
      {
        m_CurrentDirEntrys.insert(dirEntry);
      }
    }
  }

  int maxNameLen = m_W - 9;
  m_Items.clear();
  for (auto& dirEntry : m_CurrentDirEntrys)
  {
    std::wstring name = StrUtil::ToWString(dirEntry.name);
    name = StrUtil::TrimPadWString(name.substr(0, maxNameLen), maxNameLen);

    // add 9 chars for dir / filesize
    if (dirEntry.IsDir())
    {
      name += L"    (dir)";
    }
    else
    {
      // max 7 - ex: "1234 KB"
      std::string size = FileUtil::GetSuffixedSize(dirEntry.size);
      size = std::string(7 - size.size(), ' ') + size;
      name += L"  " + StrUtil::ToWString(size);
    }

    m_Items.push_back(name);
  }

  m_Index = 0;
}
