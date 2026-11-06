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

  unittest::ExpectTrue(RequestFromJson(
    R"j({"type":"download_file","profile":"P","chat_id":"c1","msg_id":"m1","file_id":"f1"})j", p, err));
  unittest::ExpectTrue(p.request->GetMessageType() == DownloadFileRequestType);
  std::shared_ptr<DownloadFileRequest> d = std::static_pointer_cast<DownloadFileRequest>(p.request);
  unittest::ExpectEqual(std::string, d->chatId, std::string("c1"));
  unittest::ExpectEqual(std::string, d->msgId, std::string("m1"));
  unittest::ExpectEqual(std::string, d->fileId, std::string("f1"));

  unittest::ExpectTrue(RequestFromJson(
    R"j({"type":"edit_message","profile":"P","chat_id":"c1","msg_id":"m1","text":"edited"})j", p, err));
  unittest::ExpectTrue(p.request->GetMessageType() == EditMessageRequestType);

  unittest::ExpectTrue(RequestFromJson(
    R"j({"type":"delete_message","profile":"P","chat_id":"c1","msg_id":"m1"})j", p, err));
  unittest::ExpectTrue(p.request->GetMessageType() == DeleteMessageRequestType);

  unittest::ExpectTrue(RequestFromJson(
    R"j({"type":"send_reaction","profile":"P","chat_id":"c1","msg_id":"m1","emoji":"👍"})j", p, err));
  unittest::ExpectTrue(p.request->GetMessageType() == SendReactionRequestType);

  unittest::ExpectTrue(RequestFromJson(
    R"j({"type":"get_sticker_sets","profile":"P"})j", p, err));
  unittest::ExpectTrue(p.request->GetMessageType() == GetStickerSetsRequestType);

  unittest::ExpectTrue(RequestFromJson(
    R"j({"type":"get_stickers","profile":"P","key":"recent","set_id":0,"query":""})j", p, err));
  unittest::ExpectTrue(p.request->GetMessageType() == GetStickersRequestType);
  std::shared_ptr<GetStickersRequest> gs = std::static_pointer_cast<GetStickersRequest>(p.request);
  unittest::ExpectEqual(std::string, gs->key, std::string("recent"));
  unittest::ExpectEqual(int64_t, gs->setId, (int64_t)0);

  unittest::ExpectTrue(RequestFromJson(
    R"j({"type":"get_stickers","profile":"P","key":"set:123456789012","set_id":123456789012,"query":""})j", p, err));
  gs = std::static_pointer_cast<GetStickersRequest>(p.request);
  unittest::ExpectEqual(int64_t, gs->setId, (int64_t)123456789012LL);

  unittest::ExpectTrue(RequestFromJson(
    R"j({"type":"send_sticker","profile":"P","chat_id":"c1","sticker_file_id":42})j", p, err));
  unittest::ExpectTrue(p.request->GetMessageType() == SendStickerRequestType);
  std::shared_ptr<SendStickerRequest> ss = std::static_pointer_cast<SendStickerRequest>(p.request);
  unittest::ExpectEqual(std::string, ss->chatId, std::string("c1"));
  unittest::ExpectEqual(int32_t, ss->stickerFileId, (int32_t)42);
}

static void TestSetPollAnswer()
{
  ParsedRequest p;
  std::string err;
  unittest::ExpectTrue(RequestFromJson(
    R"j({"type":"set_poll_answer","profile":"P","chat_id":"c1","msg_id":"m9","option_ids":[0,2]})j", p, err));
  unittest::ExpectTrue(p.request->GetMessageType() == SetPollAnswerRequestType);
  std::shared_ptr<SetPollAnswerRequest> r = std::static_pointer_cast<SetPollAnswerRequest>(p.request);
  unittest::ExpectEqual(std::string, r->chatId, std::string("c1"));
  unittest::ExpectEqual(std::string, r->msgId, std::string("m9"));
  unittest::ExpectEqual(size_t, r->optionIds.size(), (size_t)2);
  unittest::ExpectEqual(int32_t, r->optionIds[0], (int32_t)0);
  unittest::ExpectEqual(int32_t, r->optionIds[1], (int32_t)2);
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
    TestSetPollAnswer();
    TestErrors();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
