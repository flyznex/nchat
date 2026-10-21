// test005.cpp - kitty graphics protocol encoding for inline images
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <iostream>
#include <string>
#include <vector>

#include "kittyencoder.h"

#include "unittest.h"

static void TestDiacritics()
{
  unittest::ExpectEqual(int, KittyEncoder::GetDiacriticCount(), 297);
  unittest::ExpectEqual(int, static_cast<int>(KittyEncoder::Diacritic(0)), 0x305);
  unittest::ExpectEqual(int, static_cast<int>(KittyEncoder::Diacritic(1)), 0x30D);
  unittest::ExpectEqual(int, static_cast<int>(KittyEncoder::Diacritic(296)), 0x1D244);
  unittest::ExpectEqual(int, static_cast<int>(KittyEncoder::Diacritic(297)), 0);
  unittest::ExpectEqual(int, static_cast<int>(KittyEncoder::Diacritic(-1)), 0);
}

static void TestCellChars()
{
  wchar_t w[4];
  unittest::ExpectTrue(KittyEncoder::CellChars(1, 2, w));
  unittest::ExpectEqual(int, static_cast<int>(w[0]), 0x10EEEE);
  unittest::ExpectEqual(int, static_cast<int>(w[1]), 0x30D);
  unittest::ExpectEqual(int, static_cast<int>(w[2]), 0x30E);
  unittest::ExpectEqual(int, static_cast<int>(w[3]), 0);

  unittest::ExpectFalse(KittyEncoder::CellChars(0, 297, w));
  unittest::ExpectEqual(int, static_cast<int>(w[0]), static_cast<int>(L' '));
  unittest::ExpectEqual(int, static_cast<int>(w[1]), 0);
  unittest::ExpectFalse(KittyEncoder::CellChars(-1, 0, w));
}

static void TestBase64()
{
  unittest::ExpectEqual(std::string, KittyEncoder::Base64(""), "");
  unittest::ExpectEqual(std::string, KittyEncoder::Base64("f"), "Zg==");
  unittest::ExpectEqual(std::string, KittyEncoder::Base64("fo"), "Zm8=");
  unittest::ExpectEqual(std::string, KittyEncoder::Base64("foo"), "Zm9v");
  unittest::ExpectEqual(std::string, KittyEncoder::Base64("foob"), "Zm9vYg==");
  unittest::ExpectEqual(std::string, KittyEncoder::Base64(std::string("\xff\xfe\x00", 3)), "//4A");
}

static void TestEncodeUploadSingleChunk()
{
  const std::vector<std::string> seqs = KittyEncoder::EncodeUpload(7, 20, 5, "foo");
  unittest::ExpectEqual(size_t, seqs.size(), 1);
  unittest::ExpectEqual(std::string, seqs[0], "\033_Ga=T,U=1,f=100,i=7,c=20,r=5,q=2,m=0;Zm9v\033\\");
}

static void TestEncodeUploadChunked()
{
  // 3100 bytes -> 4136 base64 chars -> chunks of 4096 and 40
  const std::vector<std::string> seqs = KittyEncoder::EncodeUpload(1, 2, 2, std::string(3100, 'A'));
  unittest::ExpectEqual(size_t, seqs.size(), 2);

  const std::string prefix = "\033_Ga=T,U=1,f=100,i=1,c=2,r=2,q=2,m=1;";
  unittest::ExpectEqual(std::string, seqs[0].substr(0, prefix.size()), prefix);
  unittest::ExpectEqual(size_t, seqs[0].size(), prefix.size() + 4096 + 2);
  unittest::ExpectEqual(std::string, seqs[0].substr(seqs[0].size() - 2), "\033\\");

  unittest::ExpectEqual(std::string, seqs[1].substr(0, 7), "\033_Gm=0;");
  unittest::ExpectEqual(size_t, seqs[1].size(), 7 + 40 + 2);
  unittest::ExpectEqual(std::string, seqs[1].substr(seqs[1].size() - 2), "\033\\");
}

static void TestEncodeUploadExactChunkBoundary()
{
  // 3072 bytes -> exactly 4096 base64 chars -> one chunk, last
  const std::vector<std::string> seqs = KittyEncoder::EncodeUpload(1, 2, 2, std::string(3072, '\0'));
  unittest::ExpectEqual(size_t, seqs.size(), 1);
  unittest::ExpectEqual(std::string, seqs[0].substr(0, 40), "\033_Ga=T,U=1,f=100,i=1,c=2,r=2,q=2,m=0;AAA");
}

static void TestEncodeDelete()
{
  unittest::ExpectEqual(std::string, KittyEncoder::EncodeDelete(5), "\033_Ga=d,d=I,i=5,q=2\033\\");
}

static void TestWrapTmux()
{
  unittest::ExpectEqual(std::string, KittyEncoder::WrapTmux("\033_Gx\033\\"),
                        "\033Ptmux;\033\033_Gx\033\033\\\033\\");
  unittest::ExpectEqual(std::string, KittyEncoder::WrapTmux("abc"), "\033Ptmux;abc\033\\");
}

static void TestIsSupportedTerm()
{
  unittest::ExpectFalse(KittyEncoder::IsSupportedTerm(false, false, "xterm-256color"));
  unittest::ExpectFalse(KittyEncoder::IsSupportedTerm(false, false, "screen"));
  unittest::ExpectFalse(KittyEncoder::IsSupportedTerm(false, false, ""));
  unittest::ExpectTrue(KittyEncoder::IsSupportedTerm(true, false, "xterm-256color"));
  unittest::ExpectTrue(KittyEncoder::IsSupportedTerm(false, true, "tmux-256color"));
  unittest::ExpectTrue(KittyEncoder::IsSupportedTerm(false, false, "xterm-kitty"));
  unittest::ExpectTrue(KittyEncoder::IsSupportedTerm(false, false, "xterm-ghostty"));
}

int main()
{
  int rv = 0;

  try
  {
    TestDiacritics();
    TestCellChars();
    TestBase64();
    TestEncodeUploadSingleChunk();
    TestEncodeUploadChunked();
    TestEncodeUploadExactChunkBoundary();
    TestEncodeDelete();
    TestWrapTmux();
    TestIsSupportedTerm();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
