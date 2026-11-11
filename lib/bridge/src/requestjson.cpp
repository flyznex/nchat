// requestjson.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "requestjson.h"

#include "fileutil.h"
#include "protocolutil.h"

#include "cereal/external/rapidjson/document.h"

namespace
{
bool GetStr(const rapidjson::Document& p_Doc, const char* p_Key, std::string& p_Out)
{
  rapidjson::Value::ConstMemberIterator it = p_Doc.FindMember(p_Key);
  if ((it == p_Doc.MemberEnd()) || !it->value.IsString()) return false;

  p_Out.assign(it->value.GetString(), it->value.GetStringLength());
  return true;
}

bool GetInt(const rapidjson::Document& p_Doc, const char* p_Key, int& p_Out)
{
  rapidjson::Value::ConstMemberIterator it = p_Doc.FindMember(p_Key);
  if ((it == p_Doc.MemberEnd()) || !it->value.IsInt()) return false;

  p_Out = it->value.GetInt();
  return true;
}

bool GetInt64(const rapidjson::Document& p_Doc, const char* p_Key, int64_t& p_Out)
{
  rapidjson::Value::ConstMemberIterator it = p_Doc.FindMember(p_Key);
  if ((it == p_Doc.MemberEnd()) || !it->value.IsInt64()) return false;

  p_Out = it->value.GetInt64();
  return true;
}

bool Missing(const char* p_Key, std::string& p_Error)
{
  p_Error = std::string("missing field: ") + p_Key;
  return false;
}
}

bool RequestFromJson(const std::string& p_Json, ParsedRequest& p_Out, std::string& p_Error)
{
  rapidjson::Document doc;
  doc.Parse(p_Json.c_str());
  if (doc.HasParseError() || !doc.IsObject())
  {
    p_Error = "invalid json";
    return false;
  }

  std::string type;
  if (!GetStr(doc, "type", type)) return Missing("type", p_Error);
  if (!GetStr(doc, "profile", p_Out.profileId)) return Missing("profile", p_Error);

  if (type == "get_chats")
  {
    p_Out.request = std::make_shared<GetChatsRequest>();
  }
  else if (type == "get_contacts")
  {
    p_Out.request = std::make_shared<GetContactsRequest>();
  }
  else if (type == "get_messages")
  {
    std::shared_ptr<GetMessagesRequest> r = std::make_shared<GetMessagesRequest>();
    int limit = 0;
    if (!GetStr(doc, "chat_id", r->chatId)) return Missing("chat_id", p_Error);
    if (!GetInt(doc, "limit", limit)) return Missing("limit", p_Error);
    GetStr(doc, "from_msg_id", r->fromMsgId);
    r->limit = limit;
    p_Out.request = r;
  }
  else if (type == "send_message")
  {
    std::shared_ptr<SendMessageRequest> r = std::make_shared<SendMessageRequest>();
    if (!GetStr(doc, "chat_id", r->chatId)) return Missing("chat_id", p_Error);
    if (!GetStr(doc, "text", r->chatMessage.text)) return Missing("text", p_Error);
    GetStr(doc, "quoted_id", r->chatMessage.quotedId);
    std::string filePath;
    if (GetStr(doc, "file_path", filePath) && !filePath.empty())
    {
      FileInfo fileInfo;
      fileInfo.filePath = filePath;
      fileInfo.fileStatus = FileStatusDownloaded;
      fileInfo.fileType = FileUtil::GetMimeType(filePath);
      r->chatMessage.fileInfo = ProtocolUtil::FileInfoToHex(fileInfo);
    }
    p_Out.request = r;
  }
  else if (type == "edit_message")
  {
    std::shared_ptr<EditMessageRequest> r = std::make_shared<EditMessageRequest>();
    if (!GetStr(doc, "chat_id", r->chatId)) return Missing("chat_id", p_Error);
    if (!GetStr(doc, "msg_id", r->msgId)) return Missing("msg_id", p_Error);
    if (!GetStr(doc, "text", r->chatMessage.text)) return Missing("text", p_Error);
    p_Out.request = r;
  }
  else if (type == "delete_message")
  {
    std::shared_ptr<DeleteMessageRequest> r = std::make_shared<DeleteMessageRequest>();
    if (!GetStr(doc, "chat_id", r->chatId)) return Missing("chat_id", p_Error);
    if (!GetStr(doc, "msg_id", r->msgId)) return Missing("msg_id", p_Error);
    p_Out.request = r;
  }
  else if (type == "send_reaction")
  {
    std::shared_ptr<SendReactionRequest> r = std::make_shared<SendReactionRequest>();
    if (!GetStr(doc, "chat_id", r->chatId)) return Missing("chat_id", p_Error);
    if (!GetStr(doc, "msg_id", r->msgId)) return Missing("msg_id", p_Error);
    if (!GetStr(doc, "emoji", r->emoji)) return Missing("emoji", p_Error);
    GetStr(doc, "prev_emoji", r->prevEmoji);
    p_Out.request = r;
  }
  else if (type == "download_file")
  {
    std::shared_ptr<DownloadFileRequest> r = std::make_shared<DownloadFileRequest>();
    if (!GetStr(doc, "chat_id", r->chatId)) return Missing("chat_id", p_Error);
    if (!GetStr(doc, "msg_id", r->msgId)) return Missing("msg_id", p_Error);
    if (!GetStr(doc, "file_id", r->fileId)) return Missing("file_id", p_Error);
    r->downloadFileAction = DownloadFileActionNone;
    p_Out.request = r;
  }
  else if (type == "mark_read")
  {
    std::shared_ptr<MarkMessageReadRequest> r = std::make_shared<MarkMessageReadRequest>();
    if (!GetStr(doc, "chat_id", r->chatId)) return Missing("chat_id", p_Error);
    if (!GetStr(doc, "msg_id", r->msgId)) return Missing("msg_id", p_Error);
    p_Out.request = r;
  }
  else if (type == "get_status")
  {
    std::shared_ptr<GetStatusRequest> r = std::make_shared<GetStatusRequest>();
    if (!GetStr(doc, "user_id", r->userId)) return Missing("user_id", p_Error);
    p_Out.request = r;
  }
  else if (type == "get_sticker_sets")
  {
    p_Out.request = std::make_shared<GetStickerSetsRequest>();
  }
  else if (type == "get_stickers")
  {
    std::shared_ptr<GetStickersRequest> r = std::make_shared<GetStickersRequest>();
    if (!GetStr(doc, "key", r->key)) return Missing("key", p_Error);
    int64_t setId = 0;
    GetInt64(doc, "set_id", setId);
    r->setId = setId;
    GetStr(doc, "query", r->query);
    p_Out.request = r;
  }
  else if (type == "send_sticker")
  {
    std::shared_ptr<SendStickerRequest> r = std::make_shared<SendStickerRequest>();
    if (!GetStr(doc, "chat_id", r->chatId)) return Missing("chat_id", p_Error);
    int stickerFileId = 0;
    if (!GetInt(doc, "sticker_file_id", stickerFileId)) return Missing("sticker_file_id", p_Error);
    r->stickerFileId = stickerFileId;
    p_Out.request = r;
  }
  else if (type == "set_poll_answer")
  {
    std::shared_ptr<SetPollAnswerRequest> r = std::make_shared<SetPollAnswerRequest>();
    if (!GetStr(doc, "chat_id", r->chatId)) return Missing("chat_id", p_Error);
    if (!GetStr(doc, "msg_id", r->msgId)) return Missing("msg_id", p_Error);
    if (doc.HasMember("option_ids") && doc["option_ids"].IsArray())
    {
      for (const auto& opt : doc["option_ids"].GetArray())
      {
        if (opt.IsInt()) r->optionIds.push_back(opt.GetInt());
      }
    }
    p_Out.request = r;
  }
  else if (type == "open_web_app")
  {
    std::shared_ptr<OpenWebAppRequest> r = std::make_shared<OpenWebAppRequest>();
    if (!GetStr(doc, "chat_id", r->chatId)) return Missing("chat_id", p_Error);
    if (!GetStr(doc, "bot_user_id", r->botUserId)) return Missing("bot_user_id", p_Error);
    if (!GetStr(doc, "url", r->url)) return Missing("url", p_Error);
    p_Out.request = r;
  }
  else
  {
    p_Error = "unknown request type: " + type;
    return false;
  }

  return true;
}
