// bridge.h - C ABI between the Rust UI and the protocol core
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#define NC_ABI_VERSION 1

// Contract:
// - Single control thread: nc_init, nc_set_callback, nc_start, nc_send, nc_profiles and
//   nc_stop must not be called concurrently with each other.
// - The callback runs on a core-owned thread. Its json pointer is valid only for the
//   duration of the call (copy it). The callback must NOT call nc_stop or nc_set_callback
//   (deadlock on the callback mutex).
// - nc_init calls umask(077) on the whole host process.

enum
{
  NC_OK = 0,
  NC_ERR_STATE = 1,
  NC_ERR_REQUEST = 2,
  NC_ERR_NO_PROFILE = 3,
  NC_ERR_CONFDIR = 4,
  NC_ERR_LOCKED = 5,
};

typedef void (*NcEventCallback)(const char* p_Json, void* p_User);

int nc_abi_version(void);
// An empty p_ConfigDir selects nchat's default dir (~/.nchat, else $XDG_CONFIG_HOME/nchat, else ~/.config/nchat).
int nc_init(const char* p_ConfigDir);
void nc_set_callback(NcEventCallback p_Callback, void* p_User);
int nc_start(void);
int nc_send(const char* p_Json);
char* nc_profiles(void);
void nc_free_string(char* p_Str);
void nc_stop(void);

#ifdef __cplusplus
}
#endif
