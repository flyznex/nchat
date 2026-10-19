// test003.cpp - chat id parsing for telegram forum topics
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <iostream>
#include <string>

#include "chatid.h"
#include "strutil.h"

#include "unittest.h"

static void TestMakeChatId()
{
  const int64_t chatId = -1001234567890LL;
  const std::string plain = StrUtil::NumToHex(chatId);

  unittest::ExpectEqual(std::string, MakeChatId(chatId, 0), plain);
  unittest::ExpectEqual(std::string, MakeChatId(chatId, 1), plain);
  unittest::ExpectEqual(std::string, MakeChatId(chatId, 2), plain + ":2");
  unittest::ExpectEqual(std::string, MakeChatId(chatId, 4711), plain + ":4711");
}

static void TestParseChatId()
{
  const int64_t chatId = -1001234567890LL;
  const std::string plain = StrUtil::NumToHex(chatId);

  ChatRef ref = ParseChatId(plain);
  unittest::ExpectEqual(int64_t, ref.chatId, chatId);
  unittest::ExpectEqual(int32_t, ref.topicId, 0);

  ref = ParseChatId(plain + ":5");
  unittest::ExpectEqual(int64_t, ref.chatId, chatId);
  unittest::ExpectEqual(int32_t, ref.topicId, 5);

  ref = ParseChatId(StrUtil::NumToHex(int64_t(123456789)));
  unittest::ExpectEqual(int64_t, ref.chatId, 123456789);
  unittest::ExpectEqual(int32_t, ref.topicId, 0);

  ref = ParseChatId("");
  unittest::ExpectEqual(int64_t, ref.chatId, 0);
  unittest::ExpectEqual(int32_t, ref.topicId, 0);
}

static void TestRoundTrip()
{
  const int64_t chatIds[] = { 1, -1, 777000, -1009999999999LL };
  const int32_t topicIds[] = { 2, 3, 100, 2147483647 };
  for (const int64_t chatId : chatIds)
  {
    for (const int32_t topicId : topicIds)
    {
      const ChatRef ref = ParseChatId(MakeChatId(chatId, topicId));
      unittest::ExpectEqual(int64_t, ref.chatId, chatId);
      unittest::ExpectEqual(int32_t, ref.topicId, topicId);
    }
  }
}

int main()
{
  int rv = 0;

  try
  {
    TestMakeChatId();
    TestParseChatId();
    TestRoundTrip();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
