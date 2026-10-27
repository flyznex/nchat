// test014.cpp - link preview formatting
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <iostream>

#include "protocolutil.h"

#include "unittest.h"

static void TestFormatLinkPreview()
{
  unittest::ExpectEqual(std::string, ProtocolUtil::FormatLinkPreview("GitHub", "flyznex/nchat", "A chat client", "github.com"),
                        "GitHub: flyznex/nchat\nA chat client");
  unittest::ExpectEqual(std::string, ProtocolUtil::FormatLinkPreview("GitHub", "flyznex/nchat", "", "github.com"),
                        "GitHub: flyznex/nchat");
  unittest::ExpectEqual(std::string, ProtocolUtil::FormatLinkPreview("", "Only title", "", "x.com"), "Only title");
  unittest::ExpectEqual(std::string, ProtocolUtil::FormatLinkPreview("Only site", "", "", "x.com"), "Only site");
  unittest::ExpectEqual(std::string, ProtocolUtil::FormatLinkPreview("", "", "", "x.com/page"), "x.com/page");
  unittest::ExpectEqual(std::string, ProtocolUtil::FormatLinkPreview("", "", "desc only", ""), "desc only");
  unittest::ExpectEqual(std::string, ProtocolUtil::FormatLinkPreview("", "", "", ""), "");
  unittest::ExpectEqual(std::string, ProtocolUtil::FormatLinkPreview("A\n", " B\tC ", "x\r\ny", ""), "A: B C\nx  y");
}

int main()
{
  int rv = 0;
  try
  {
    TestFormatLinkPreview();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
