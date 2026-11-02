// requestjson.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "requestjson.h"

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
    p_Out.request = r;
  }
  else
  {
    p_Error = "unknown request type: " + type;
    return false;
  }

  return true;
}
