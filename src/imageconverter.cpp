// imageconverter.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "imageconverter.h"

#include <vector>

#include "fileutil.h"
#include "strutil.h"
#include "sysutil.h"

bool ImageConverter::ToPng(const std::string& p_SrcPath, const std::string& p_DstPngPath)
{
  const std::string tmpPath = p_DstPngPath + ".tmp.png";
  const std::string src = StrUtil::EscapeSingleQuote(p_SrcPath);
  const std::string tmp = StrUtil::EscapeSingleQuote(tmpPath);

  const std::vector<std::string> cmds =
  {
    "magick '" + src + "[0]' -auto-orient -resize '800x800>' '" + tmp + "'",
    "sips -s format png -Z 800 '" + src + "' --out '" + tmp + "'",
    "dwebp '" + src + "' -o '" + tmp + "'",
    // video stickers: first frame only; libvpx-vp9 keeps the alpha channel, the default decoder drops it
    "ffmpeg -v error -y -c:v libvpx-vp9 -i '" + src + "' -frames:v 1 '" + tmp + "'",
    "ffmpeg -v error -y -i '" + src + "' -frames:v 1 '" + tmp + "'",
  };

  for (const std::string& cmd : cmds)
  {
    FileUtil::RmFile(tmpPath);
    if ((SysUtil::System(cmd + " >/dev/null 2>&1") == 0) && FileUtil::Exists(tmpPath) &&
        (FileUtil::ReadFile(tmpPath).size() >= 24))
    {
      FileUtil::Move(tmpPath, p_DstPngPath);
      return true;
    }
  }

  FileUtil::RmFile(tmpPath);
  return false;
}
