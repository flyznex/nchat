// imagecache.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "imagecache.h"

#include <atomic>
#include <condition_variable>
#include <cstdint>
#include <cstdio>
#include <deque>
#include <fstream>
#include <functional>
#include <mutex>
#include <thread>
#include <unordered_map>
#include <vector>

#include <sys/stat.h>

#include "fileutil.h"
#include "imageconverter.h"
#include "imagegeom.h"
#include "log.h"
#include "termgfx.h"

namespace
{
  struct Entry
  {
    ImageCache::State state = ImageCache::State::Pending;
    ImageInfo info;
  };

  struct Placement
  {
    int id = 0;
    uint64_t lastPass = 0;
  };

  const int s_MaxId = 255;

  std::mutex s_Mutex;
  std::condition_variable s_Cv;
  std::unordered_map<std::string, Entry> s_Entries;
  std::deque<std::string> s_Queue;
  std::thread s_Worker;
  bool s_Stop = false;
  std::atomic<bool> s_Completed{ false };

  // UI thread only
  std::unordered_map<std::string, Placement> s_Placements;
  std::vector<int> s_FreeIds;
  uint64_t s_Pass = 0;

  std::string GetImagesDir()
  {
    return FileUtil::GetApplicationDir() + "/images";
  }

  std::string CacheName(const std::string& p_SrcPath)
  {
    // size and mtime are part of the key so a reused path (e.g. temp/clipboard-1.png) never shows a stale preview
    std::string key = p_SrcPath;
    struct stat st;
    if (stat(p_SrcPath.c_str(), &st) == 0)
    {
      key += "|" + std::to_string(static_cast<long long>(st.st_size)) + "|" +
             std::to_string(static_cast<long long>(st.st_mtime));
    }

    char buf[40];
    snprintf(buf, sizeof(buf), "%016zx.png", std::hash<std::string>{}(key));
    return buf;
  }

  std::string ReadHeader(const std::string& p_Path)
  {
    std::ifstream file(p_Path, std::ios::binary);
    char buf[24] = { 0 };
    file.read(buf, sizeof(buf));
    return std::string(buf, static_cast<size_t>(file.gcount()));
  }

  void ResetFreeIds()
  {
    s_FreeIds.clear();
    for (int id = s_MaxId; id >= 1; --id) s_FreeIds.push_back(id);
  }

  void WorkerMain()
  {
    while (true)
    {
      std::string src;
      {
        std::unique_lock<std::mutex> lock(s_Mutex);
        s_Cv.wait(lock, [] { return s_Stop || !s_Queue.empty(); });
        if (s_Stop) return;
        src = s_Queue.front();
        s_Queue.pop_front();
      }

      ImageInfo info;
      info.pngPath = GetImagesDir() + "/" + CacheName(src);
      bool ok = FileUtil::Exists(info.pngPath) || ImageConverter::ToPng(src, info.pngPath);
      ok = ok && ImageGeom::ParsePngSize(ReadHeader(info.pngPath), info.pxWidth, info.pxHeight);
      if (!ok)
      {
        LOG_WARNING("inline image conversion failed for %s", src.c_str());
      }

      {
        std::unique_lock<std::mutex> lock(s_Mutex);
        Entry& entry = s_Entries[src];
        entry.state = ok ? ImageCache::State::Ready : ImageCache::State::Failed;
        entry.info = info;
      }

      s_Completed = true;
    }
  }
}

ImageCache::State ImageCache::Request(const std::string& p_SrcPath, ImageInfo* p_Info)
{
  std::unique_lock<std::mutex> lock(s_Mutex);
  if (s_Stop) return State::Failed;

  auto it = s_Entries.find(p_SrcPath);
  if (it == s_Entries.end())
  {
    s_Entries[p_SrcPath] = Entry();
    s_Queue.push_back(p_SrcPath);
    if (!s_Worker.joinable())
    {
      FileUtil::MkDir(GetImagesDir());
      s_Worker = std::thread(WorkerMain);
    }

    s_Cv.notify_one();
    return State::Pending;
  }

  if ((it->second.state == State::Ready) && (p_Info != nullptr))
  {
    *p_Info = it->second.info;
  }

  return it->second.state;
}

bool ImageCache::TakeCompleted()
{
  return s_Completed.exchange(false);
}

void ImageCache::BeginDrawPass()
{
  ++s_Pass;
}

int ImageCache::Acquire(const std::string& p_SrcPath, const ImageInfo& p_Info, int p_Cols, int p_Rows)
{
  if (s_FreeIds.empty() && s_Placements.empty()) ResetFreeIds();

  const std::string key = p_SrcPath + "|" + std::to_string(p_Cols) + "x" + std::to_string(p_Rows);
  auto it = s_Placements.find(key);
  if (it != s_Placements.end())
  {
    it->second.lastPass = s_Pass;
    return it->second.id;
  }

  int id = 0;
  if (!s_FreeIds.empty())
  {
    id = s_FreeIds.back();
    s_FreeIds.pop_back();
  }
  else
  {
    auto victim = s_Placements.end();
    for (auto p = s_Placements.begin(); p != s_Placements.end(); ++p)
    {
      if ((p->second.lastPass < s_Pass) &&
          ((victim == s_Placements.end()) || (p->second.lastPass < victim->second.lastPass)))
      {
        victim = p;
      }
    }

    if (victim == s_Placements.end()) return 0;

    id = victim->second.id;
    TermGfx::Delete(id);
    s_Placements.erase(victim);
  }

  if (!TermGfx::Upload(id, p_Cols, p_Rows, p_Info.pngPath))
  {
    s_FreeIds.push_back(id);
    return 0;
  }

  Placement placement;
  placement.id = id;
  placement.lastPass = s_Pass;
  s_Placements[key] = placement;
  return id;
}

void ImageCache::InvalidateAll()
{
  for (const auto& placement : s_Placements)
  {
    TermGfx::Delete(placement.second.id);
  }

  s_Placements.clear();
  ResetFreeIds();
}

void ImageCache::Shutdown()
{
  {
    std::unique_lock<std::mutex> lock(s_Mutex);
    s_Stop = true;
  }

  s_Cv.notify_all();
  if (s_Worker.joinable()) s_Worker.join();

  InvalidateAll();
}
