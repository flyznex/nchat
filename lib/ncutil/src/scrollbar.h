// scrollbar.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

// thumb of a vertical scroll bar; false when everything fits or there is no track to draw on
bool ScrollThumb(int p_Total, int p_First, int p_Visible, int p_Track, int& p_Pos, int& p_Len);
