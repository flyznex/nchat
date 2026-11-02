// test019.cpp - JSON request parsing for the Rust bridge
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <iostream>
#include <memory>
#include <string>

#include "protocol.h"
#include "requestjson.h"

#include "unittest.h"

static void TestGetChatsAndContacts()
{
  ParsedRequest p;
  std::string err;
  unittest::ExpectTrue(RequestFromJson(R"j({"type":"get_chats","profile":"Dummy_1"})j", p, err));
  unittest::ExpectEqual(std::string, p.profileId, std::string("Dummy_1"));
  unittest::ExpectTrue(p.request->GetMessageType() == GetChatsRequestType);

  unittest::ExpectTrue(RequestFromJson(R"j({"type":"get_contacts","profile":"Dummy_1"})j", p, err));
  unittest::ExpectTrue(p.request->GetMessageType() == GetContactsRequestType);
}

static void TestGetMessages()
{
  ParsedRequest p;
  std::string err;
  unittest::ExpectTrue(RequestFromJson(
    R"j({"type":"get_messages","profile":"P","chat_id":"c1","from_msg_id":"m9","limit":12})j", p, err));
  unittest::ExpectTrue(p.request->GetMessageType() == GetMessagesRequestType);
  std::shared_ptr<GetMessagesRequest> r = std::static_pointer_cast<GetMessagesRequest>(p.request);
  unittest::ExpectEqual(std::string, r->chatId, std::string("c1"));
  unittest::ExpectEqual(std::string, r->fromMsgId, std::string("m9"));
  unittest::ExpectEqual(int, static_cast<int>(r->limit), 12);

  // from_msg_id is optional
  unittest::ExpectTrue(RequestFromJson(
    R"j({"type":"get_messages","profile":"P","chat_id":"c1","limit":3})j", p, err));
  r = std::static_pointer_cast<GetMessagesRequest>(p.request);
  unittest::ExpectEqual(std::string, r->fromMsgId, std::string(""));
}

static void TestSendMessage()
{
  ParsedRequest p;
  std::string err;
  unittest::ExpectTrue(RequestFromJson(
    R"j({"type":"send_message","profile":"P","chat_id":"c1","text":"hej \"du\"\n"})j", p, err));
  unittest::ExpectTrue(p.request->GetMessageType() == SendMessageRequestType);
  std::shared_ptr<SendMessageRequest> r = std::static_pointer_cast<SendMessageRequest>(p.request);
  unittest::ExpectEqual(std::string, r->chatId, std::string("c1"));
  unittest::ExpectEqual(std::string, r->chatMessage.text, std::string("hej \"du\"\n"));
}

static void TestErrors()
{
  ParsedRequest p;
  std::string err;
  unittest::ExpectFalse(RequestFromJson("not json", p, err));
  unittest::ExpectEqual(std::string, err, std::string("invalid json"));

  unittest::ExpectFalse(RequestFromJson(R"j({"type":"get_chats"})j", p, err));
  unittest::ExpectEqual(std::string, err, std::string("missing field: profile"));

  unittest::ExpectFalse(RequestFromJson(R"j({"type":"get_messages","profile":"P","chat_id":"c1"})j", p, err));
  unittest::ExpectEqual(std::string, err, std::string("missing field: limit"));

  unittest::ExpectFalse(RequestFromJson(R"j({"type":"explode","profile":"P"})j", p, err));
  unittest::ExpectEqual(std::string, err, std::string("unknown request type: explode"));
}

int main()
{
  int rv = 0;
  try
  {
    TestGetChatsAndContacts();
    TestGetMessages();
    TestSendMessage();
    TestErrors();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
