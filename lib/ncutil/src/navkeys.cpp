// navkeys.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "navkeys.h"

NavAction NavKeyResolver::Resolve(NavFocus p_Focus, wint_t p_Key, const NavKeys& p_Keys, bool p_Vim)
{
  const bool pendingG = m_PendingG;
  m_PendingG = false;
  const bool pendingPane = m_PendingPane;
  m_PendingPane = false;

  if (p_Vim && pendingPane)
  {
    // vim Ctrl-W h/j/k/l on the layout: list left, history top right, entry bottom right,
    // Ctrl-W a opens the address book dialog (select_contact).
    // Any other key after the prefix is swallowed (Pending) so it cannot type into the entry.
    const bool list = (p_Focus == NavFocus::List);
    switch (p_Key)
    {
      case L'h': return list ? NavAction::Pending : NavAction::FocusList;
      case L'l': return list ? NavAction::FocusHistory : NavAction::Pending;
      case L'j': return (p_Focus == NavFocus::History) ? NavAction::FocusEntry : NavAction::Pending;
      case L'k': return (p_Focus == NavFocus::Entry) ? NavAction::FocusHistory : NavAction::Pending;
      case L'a': return NavAction::SelectContact;
      default: return NavAction::Pending;
    }
  }

  if (p_Vim && (p_Key == p_Keys.panePrefix))
  {
    m_PendingPane = true;
    return NavAction::Pending;
  }

  if (p_Key == p_Keys.focusNext) return NavAction::FocusNext;
  if (p_Key == p_Keys.focusPrev) return NavAction::FocusPrev;
  if (p_Key == p_Keys.focusEntry) return NavAction::FocusEntry;
  if (p_Focus == NavFocus::Entry) return NavAction::None;

  const bool list = (p_Focus == NavFocus::List);
  if (p_Key == p_Keys.up) return NavAction::Up;
  if (p_Key == p_Keys.down) return NavAction::Down;
  if (p_Key == p_Keys.home) return NavAction::First;
  if (p_Key == p_Keys.end) return NavAction::Last;
  if (p_Key == p_Keys.ok) return list ? NavAction::Open : NavAction::Reply;
  if (!p_Vim) return NavAction::None;

  // ponytail: vim letters are literals, only the vim_keys flag switches them
  switch (p_Key)
  {
    case L'j': return NavAction::Down;
    case L'k': return NavAction::Up;
    case L'G': return NavAction::Last;
    case 4: return NavAction::HalfDown; // Ctrl-d
    case 21: return NavAction::HalfUp; // Ctrl-u
    case L'i': return NavAction::FocusEntry;
    case L'/': return NavAction::Search;
    case L'g':
      if (pendingG) return NavAction::First;
      m_PendingG = true;
      return NavAction::Pending;
    case L'h': return list ? NavAction::None : NavAction::FocusList;
    case L'l': return list ? NavAction::Open : NavAction::None;
    case L'n': return list ? NavAction::FolderNext : NavAction::FindNext;
    case L'b': return list ? NavAction::FolderPrev : NavAction::None;
    case L'r': return list ? NavAction::None : NavAction::Reply;
    case L'e': return list ? NavAction::None : NavAction::Edit;
    case L'y': return list ? NavAction::None : NavAction::Copy;
    case L'x': return list ? NavAction::None : NavAction::Delete;
    case L'u': return list ? NavAction::UnreadOnly : NavAction::None;
    case L'z': return list ? NavAction::ToggleGroup : NavAction::None;
    default: return NavAction::None;
  }
}
