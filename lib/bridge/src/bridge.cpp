// bridge.cpp - C ABI between the Rust UI and the protocol core
//
// Copyright (c) 2019-2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "bridge.h"

#include <cstdlib>
#include <cstring>
#include <map>
#include <memory>
#include <mutex>
#include <sstream>
#include <thread>
#include <vector>

#include <sys/stat.h>

#include "appconfig.h"
#include "apputil.h"
#include "debuginfo.h"
#include "eventjson.h"
#include "fileutil.h"
#include "log.h"
#include "messagecache.h"
#include "profiles.h"
#include "protocolfactory.h"
#include "requestjson.h"
#include "scopeddirlock.h"
#include "status.h"

#include "cereal/external/rapidjson/stringbuffer.h"
#include "cereal/external/rapidjson/writer.h"

namespace
{
const int s_DirVersion = 1;

struct State
{
  std::mutex callbackMutex;
  NcEventCallback callback = nullptr;
  void* callbackUser = nullptr;

  bool inited = false;
  bool started = false;
  std::unique_ptr<ScopedDirLock> dirLock;
  std::vector<std::unique_ptr<ScopedDirLock>> profileDirLocks;
  std::map<std::string, std::shared_ptr<Protocol>> protocols;
  std::thread loginThread;
};

State& GetState()
{
  static State s_State;
  return s_State;
}

void HandleServiceMessage(std::shared_ptr<ServiceMessage> p_Message)
{
  const std::string json = EventToJson(p_Message);
  State& state = GetState();
  std::lock_guard<std::mutex> lock(state.callbackMutex);
  if (state.callback != nullptr)
  {
    state.callback(json.c_str(), state.callbackUser);
  }
}
}

extern "C" int nc_abi_version(void)
{
  return NC_ABI_VERSION;
}

extern "C" int nc_init(const char* p_ConfigDir)
{
  State& state = GetState();
  if ((p_ConfigDir == nullptr) || state.inited) return NC_ERR_STATE;

  umask(S_IRWXG | S_IRWXO);
  FileUtil::SetApplicationDir(p_ConfigDir);
  Log::SetVerboseLevel(Log::INFO_LEVEL);

  const std::string dir = FileUtil::GetApplicationDir();
  if (!FileUtil::Exists(dir))
  {
    FileUtil::InitDirVersion(dir, s_DirVersion);
  }
  else if (FileUtil::GetDirVersion(dir) != s_DirVersion)
  {
    return NC_ERR_CONFDIR;
  }

  std::unique_ptr<ScopedDirLock> dirLock = std::make_unique<ScopedDirLock>(dir);
  if (!dirLock->IsLocked()) return NC_ERR_LOCKED;

  Profiles::Init();
  Log::Init(dir + std::string("/log.txt"));
  AppUtil::InitSignalHandler();
  AppConfig::Init();
  FileUtil::SetDownloadsDir(AppConfig::GetStr("downloads_dir"));
  DebugInfo::Init();
  MessageCache::Init();
  FileUtil::InitTempDir();

  state.dirLock = std::move(dirLock);
  state.inited = true;
  return NC_OK;
}

extern "C" void nc_set_callback(NcEventCallback p_Callback, void* p_User)
{
  State& state = GetState();
  std::lock_guard<std::mutex> lock(state.callbackMutex);
  state.callback = p_Callback;
  state.callbackUser = p_User;
}

extern "C" int nc_start(void)
{
  State& state = GetState();
  if (!state.inited || state.started) return NC_ERR_STATE;

  MessageCache::SetMessageHandler(HandleServiceMessage);

  const std::string profilesDir = FileUtil::GetApplicationDir() + "/profiles";
  for (const std::string& profileId : FileUtil::ListDirNames(profilesDir))
  {
    if (profileId == "version") continue;

    std::stringstream ss(profileId);
    std::string protocolName;
    if ((profileId.find("_") == std::string::npos) || !std::getline(ss, protocolName, '_'))
    {
      LOG_WARNING("invalid profile name, skipping %s", profileId.c_str());
      continue;
    }

#ifndef HAS_MULTIPROTOCOL
    if (!state.protocols.empty())
    {
      LOG_WARNING("multiple profile support not enabled, skipping %s", profileId.c_str());
      continue;
    }
#endif

    const std::string profileDir = profilesDir + "/" + profileId;
    std::unique_ptr<ScopedDirLock> profileLock = std::make_unique<ScopedDirLock>(profileDir);
    if (!profileLock->IsLocked())
    {
      LOG_WARNING("unable to acquire lock for %s, skipping", profileDir.c_str());
      continue;
    }

    bool found = false;
    for (const std::shared_ptr<ProtocolBaseFactory>& factory : GetProtocolFactorys())
    {
      if (factory->GetName() != protocolName) continue;

      std::shared_ptr<Protocol> protocol = factory->Create();
      if (!protocol) continue;

      protocol->LoadProfile(profilesDir, profileId);
      state.protocols[protocol->GetProfileId()] = protocol;
      found = true;
    }

    if (found)
    {
      state.profileDirLocks.push_back(std::move(profileLock));
    }
    else
    {
      LOG_WARNING("protocol %s not supported, skipping %s", protocolName.c_str(), profileId.c_str());
    }
  }

  for (auto& entry : state.protocols)
  {
    Status::Set(entry.first, Status::FlagConnecting);
  }

  state.loginThread = std::thread([&state]()
  {
    for (auto& entry : state.protocols)
    {
      entry.second->SetMessageHandler(HandleServiceMessage);
      entry.second->Login();
    }
  });

  state.started = true;
  return NC_OK;
}

extern "C" int nc_send(const char* p_Json)
{
  State& state = GetState();
  if ((p_Json == nullptr) || !state.started) return NC_ERR_STATE;

  ParsedRequest parsed;
  std::string error;
  if (!RequestFromJson(p_Json, parsed, error))
  {
    LOG_WARNING("bad request: %s", error.c_str());
    return NC_ERR_REQUEST;
  }

  auto it = state.protocols.find(parsed.profileId);
  if (it == state.protocols.end())
  {
    LOG_WARNING("no profile \"%s\"", parsed.profileId.c_str());
    return NC_ERR_NO_PROFILE;
  }

  it->second->SendRequest(parsed.request);
  return NC_OK;
}

extern "C" char* nc_profiles(void)
{
  State& state = GetState();
  rapidjson::StringBuffer sb;
  rapidjson::Writer<rapidjson::StringBuffer> w(sb);
  w.StartArray();
  for (auto& entry : state.protocols)
  {
    const std::shared_ptr<Protocol>& p = entry.second;
    const std::string id = p->GetProfileId();
    const std::string name = p->GetProfileDisplayName();
    w.StartObject();
    w.Key("id");
    w.String(id.c_str(), static_cast<rapidjson::SizeType>(id.size()));
    w.Key("display_name");
    w.String(name.c_str(), static_cast<rapidjson::SizeType>(name.size()));
    w.Key("auto_get_chats");
    w.Bool(p->HasFeature(FeatureAutoGetChatsOnLogin));
    w.Key("auto_get_contacts");
    w.Bool(p->HasFeature(FeatureAutoGetContactsOnLogin));
    w.EndObject();
  }

  w.EndArray();
  return strdup(sb.GetString());
}

extern "C" void nc_free_string(char* p_Str)
{
  free(p_Str);
}

extern "C" void nc_stop(void)
{
  State& state = GetState();
  if (!state.inited) return;

  if (state.loginThread.joinable())
  {
    state.loginThread.join();
  }

  for (auto& entry : state.protocols)
  {
    entry.second->Logout();
    entry.second->CloseProfile();
  }

  state.protocols.clear();
  state.profileDirLocks.clear();
  MessageCache::SetMessageHandler(nullptr);
  nc_set_callback(nullptr, nullptr);

  FileUtil::CleanupTempDir();
  MessageCache::Cleanup();
  DebugInfo::Cleanup();
  AppConfig::Cleanup();
  Profiles::Cleanup();
  LOG_INFO("exit");
  Log::Cleanup(false);

  state.dirLock.reset();
  state.started = false;
  state.inited = false;
}
