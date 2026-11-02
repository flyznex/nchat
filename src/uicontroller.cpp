// uicontroller.cpp
//
// Copyright (c) 2019-2023 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "uicontroller.h"

#include <cwchar>

#include <unistd.h>
#include <sys/select.h>

#include "uikeyinput.h"

UiController::UiController()
{
}

UiController::~UiController()
{
}

void UiController::Init()
{
}

void UiController::Cleanup()
{
}

wint_t UiController::GetKey(int p_TimeOutMs)
{
  // ncurses may already hold keys read together with an earlier one (e.g. mouse + Alt-u); select() cannot see those
  wint_t key = 0;
  if (UiKeyInput::GetWch(&key) != ERR)
  {
    return key;
  }

  fd_set fds;
  FD_ZERO(&fds);
  FD_SET(STDIN_FILENO, &fds);
  int maxfd = STDIN_FILENO;
  struct timeval tv = { (p_TimeOutMs / 1000), (p_TimeOutMs % 1000) * 1000 };
  select(maxfd + 1, &fds, NULL, NULL, &tv); // ignore select() rv to get resize events
  if (FD_ISSET(STDIN_FILENO, &fds) && (UiKeyInput::GetWch(&key) != ERR))
  {
    return key;
  }

  return WEOF; // 0 is a real key (Ctrl-@ / Ctrl-2), so it cannot mean "no key"
}
