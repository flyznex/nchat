// protocolfactory.h - protocol creation, copied from src/main.cpp (removed with it in slice 6)
//
// Copyright (c) 2019-2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

#include <dlfcn.h>

#include <iostream>
#include <memory>
#include <string>
#include <vector>

#include "fileutil.h"
#include "log.h"
#include "protocol.h"

#ifdef HAS_DUMMY
#include "duchat.h"
#endif

#ifdef HAS_TELEGRAM
#include "tgchat.h"
#endif

#ifdef HAS_WHATSAPP
#include "wmchat.h"
#endif

#ifdef HAS_SIGNAL
#include "sgchat.h"
#endif

class ProtocolBaseFactory
{
public:
  ProtocolBaseFactory() { }
  virtual ~ProtocolBaseFactory() { }
  virtual std::string GetName() const = 0;
  virtual std::shared_ptr<Protocol> Create() const = 0;
};

template<typename T>
class ProtocolFactory : public ProtocolBaseFactory
{
public:
  virtual std::string GetName() const
  {
    return T::GetName();
  }

  virtual std::shared_ptr<Protocol> Create() const
  {
    std::shared_ptr<T> protocol;
#ifdef HAS_DYNAMICLOAD
    std::string libPath =
      FileUtil::DirName(FileUtil::GetSelfPath()) + "/../" CMAKE_INSTALL_LIBDIR "/" + T::GetLibName() +
      FileUtil::GetLibSuffix();
    std::string createFunc = T::GetCreateFunc();
    void* handle = dlopen(libPath.c_str(), RTLD_LAZY);
    if (handle == nullptr)
    {
      LOG_ERROR("failed dlopen %s", libPath.c_str());
      const char* dlerr = dlerror();
      if (dlerr != nullptr)
      {
        LOG_ERROR("dlerror %s", dlerr);
      }

      std::cout << "Failed to load " << libPath << ", skipping profile.\n";
      return protocol;
    }

    // intentionally leak 'handle' as dlclose may cause exit crash as go routines are not explicitly managed

    T* (* CreateFunc)() = (T * (*)())dlsym(handle, createFunc.c_str());
    if (CreateFunc == nullptr)
    {
      LOG_ERROR("failed dlsym %s", createFunc.c_str());
      return protocol;
    }

    protocol.reset(CreateFunc());
#else
    protocol = std::make_shared<T>();
#endif
    return protocol;
  }
};

inline std::vector<std::shared_ptr<ProtocolBaseFactory>> GetProtocolFactorys()
{
  std::vector<std::shared_ptr<ProtocolBaseFactory>> protocolFactorys =
  {
#ifdef HAS_DUMMY
    std::shared_ptr<ProtocolBaseFactory>(new ProtocolFactory<DuChat>()),
#endif
#ifdef HAS_TELEGRAM
    std::shared_ptr<ProtocolBaseFactory>(new ProtocolFactory<TgChat>()),
#endif
#ifdef HAS_WHATSAPP
    std::shared_ptr<ProtocolBaseFactory>(new ProtocolFactory<WmChat>()),
#endif
#ifdef HAS_SIGNAL
    std::shared_ptr<ProtocolBaseFactory>(new ProtocolFactory<SgChat>()),
#endif
  };

  return protocolFactorys;
}
