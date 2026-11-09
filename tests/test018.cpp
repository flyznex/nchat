// test018.cpp - ServiceMessage to JSON event conversion for the Rust bridge
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <iostream>
#include <memory>
#include <string>

#include "eventjson.h"
#include "protocol.h"

#include "unittest.h"

// Same literal is used as fixture in rust/nchat-ui/src/protocol.rs tests; keep in sync.
static const std::string s_MessageJson =
  R"j({"id":"m1","sender_id":"u1","text":"hi","quoted_id":"","quoted_text":"","quoted_sender":"","file_info":"","link_preview":"","time_sent":5,"is_outgoing":false,"is_read":true,"is_delivered":false,"is_edited":false,"is_deleted":false,"is_pinned":false,"has_mention":false})j";

static ChatMessage MakeMessage()
{
  ChatMessage m;
  m.id = "m1";
  m.senderId = "u1";
  m.text = "hi";
  m.timeSent = 5;
  m.isOutgoing = false;
  m.isRead = true;
  return m;
}

static void TestConnect()
{
  std::shared_ptr<ConnectNotify> n = std::make_shared<ConnectNotify>("Dummy_1");
  n->success = true;
  unittest::ExpectEqual(std::string, EventToJson(n),
                        std::string(R"j({"type":"connect","profile":"Dummy_1","success":true})j"));
}

static void TestChats()
{
  std::shared_ptr<NewChatsNotify> n = std::make_shared<NewChatsNotify>("P");
  n->success = true;
  ChatInfo c;
  c.id = "c1";
  c.isUnread = true;
  c.isPinned = true;
  c.lastMessageTime = 1700000000123LL;
  n->chatInfos.push_back(c);
  unittest::ExpectEqual(std::string, EventToJson(n), std::string(
    R"j({"type":"chats","profile":"P","success":true,"chats":[{"id":"c1","is_unread":true,"is_unread_mention":false,"is_muted":false,"is_pinned":true,"is_archived":false,"last_message_time":1700000000123}]})j"));
}

static void TestContactsEscaping()
{
  std::shared_ptr<NewContactsNotify> n = std::make_shared<NewContactsNotify>("P");
  n->fullSync = true;
  ContactInfo c;
  c.id = "u1";
  c.name = "Ann \"A\"\n";
  c.phone = "123";
  c.isSelf = true;
  n->contactInfos.push_back(c);
  unittest::ExpectEqual(std::string, EventToJson(n), std::string(
    R"j({"type":"contacts","profile":"P","full_sync":true,"contacts":[{"id":"u1","name":"Ann \"A\"\n","phone":"123","is_self":true,"is_alias":false}]})j"));
}

static void TestMessages()
{
  std::shared_ptr<NewMessagesNotify> n = std::make_shared<NewMessagesNotify>("P");
  n->success = true;
  n->chatId = "c1";
  n->sequence = true;
  n->chatMessages.push_back(MakeMessage());
  unittest::ExpectEqual(std::string, EventToJson(n), std::string(
    R"j({"type":"messages","profile":"P","success":true,"chat_id":"c1","from_msg_id":"","cached":false,"sequence":true,"messages":[)j") +
    s_MessageJson + "]}");
}

static void TestSent()
{
  std::shared_ptr<SendMessageNotify> n = std::make_shared<SendMessageNotify>("P");
  n->success = true;
  n->chatId = "c1";
  n->chatMessage = MakeMessage();
  unittest::ExpectEqual(std::string, EventToJson(n), std::string(
    R"j({"type":"sent","profile":"P","success":true,"chat_id":"c1","message":)j") + s_MessageJson + "}");
}

static void TestFile()
{
  std::shared_ptr<NewMessageFileNotify> n = std::make_shared<NewMessageFileNotify>("P");
  n->chatId = "c1";
  n->msgId = "m1";
  n->fileInfo = "info1";
  unittest::ExpectEqual(std::string, EventToJson(n),
                        std::string(R"j({"type":"file","profile":"P","chat_id":"c1","msg_id":"m1","file_info":"info1"})j"));
}

static void TestOther()
{
  std::shared_ptr<SendTypingNotify> n = std::make_shared<SendTypingNotify>("P");
  unittest::ExpectEqual(std::string, EventToJson(n),
                        std::string(R"j({"type":"other","profile":"P","kind":)j") +
                        std::to_string(static_cast<int>(SendTypingNotifyType)) + "}");
}

static void TestStickerSets()
{
  std::shared_ptr<NewStickerSetsNotify> n = std::make_shared<NewStickerSetsNotify>("P");
  StickerSetInfo s;
  s.id = 123456789012LL;
  s.title = "Cats";
  n->stickerSetInfos.push_back(s);
  unittest::ExpectEqual(std::string, EventToJson(n), std::string(
    R"j({"type":"sticker_sets","profile":"P","sticker_sets":[{"id":123456789012,"title":"Cats"}]})j"));
}

static void TestStickers()
{
  std::shared_ptr<NewStickersNotify> n = std::make_shared<NewStickersNotify>("P");
  n->key = "recent";
  StickerInfo s;
  s.fileId = 42;
  s.emoji = "😀";
  s.thumbnailPath = "/tmp/thumb.webp";
  n->stickerInfos.push_back(s);
  unittest::ExpectEqual(std::string, EventToJson(n), std::string(
    R"j({"type":"stickers","profile":"P","key":"recent","stickers":[{"file_id":42,"emoji":"😀","thumbnail_path":"/tmp/thumb.webp"}]})j"));
}

static void TestChatFolders()
{
  std::shared_ptr<NewChatFoldersNotify> n = std::make_shared<NewChatFoldersNotify>("P");
  n->folders.push_back(std::make_pair(1, "Work"));
  n->folders.push_back(std::make_pair(2, "Personal"));
  unittest::ExpectEqual(std::string, EventToJson(n), std::string(
    R"j({"type":"chat_folders","profile":"P","folders":[{"id":1,"title":"Work"},{"id":2,"title":"Personal"}]})j"));
}

static void TestUpdateChatFolder()
{
  std::shared_ptr<UpdateChatFolderNotify> n = std::make_shared<UpdateChatFolderNotify>("P");
  n->folderId = 42;
  n->chatIds.push_back("c1");
  n->chatIds.push_back("c2");
  n->isMember = true;
  unittest::ExpectEqual(std::string, EventToJson(n), std::string(
    R"j({"type":"update_chat_folder","profile":"P","folder_id":42,"is_member":true,"chat_ids":["c1","c2"]})j"));
}

static void TestMessageStatus()
{
  std::shared_ptr<NewMessageStatusNotify> n = std::make_shared<NewMessageStatusNotify>("P");
  n->chatId = "c1";
  n->msgId = "m1";
  n->isRead = true;
  n->isDelivered = false;
  unittest::ExpectEqual(std::string, EventToJson(n), std::string(
    R"j({"type":"message_status","profile":"P","chat_id":"c1","msg_id":"m1","is_read":true,"is_delivered":false})j"));
}

int main()
{
  int rv = 0;
  try
  {
    TestConnect();
    TestChats();
    TestContactsEscaping();
    TestMessages();
    TestSent();
    TestFile();
    TestOther();
    TestStickerSets();
    TestStickers();
    TestChatFolders();
    TestUpdateChatFolder();
    TestMessageStatus();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
