// eventjson.cpp
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "eventjson.h"

#include "cereal/external/rapidjson/stringbuffer.h"
#include "cereal/external/rapidjson/writer.h"

namespace
{
typedef rapidjson::Writer<rapidjson::StringBuffer> JsonWriter;

void PutStr(JsonWriter& p_W, const char* p_Key, const std::string& p_Val)
{
  p_W.Key(p_Key);
  p_W.String(p_Val.c_str(), static_cast<rapidjson::SizeType>(p_Val.size()));
}

void PutBool(JsonWriter& p_W, const char* p_Key, bool p_Val)
{
  p_W.Key(p_Key);
  p_W.Bool(p_Val);
}

void PutInt(JsonWriter& p_W, const char* p_Key, int64_t p_Val)
{
  p_W.Key(p_Key);
  p_W.Int64(p_Val);
}

void PutMessage(JsonWriter& p_W, const ChatMessage& p_Msg)
{
  p_W.StartObject();
  PutStr(p_W, "id", p_Msg.id);
  PutStr(p_W, "sender_id", p_Msg.senderId);
  PutStr(p_W, "text", p_Msg.text);
  PutStr(p_W, "quoted_id", p_Msg.quotedId);
  PutStr(p_W, "quoted_text", p_Msg.quotedText);
  PutStr(p_W, "quoted_sender", p_Msg.quotedSender);
  PutStr(p_W, "file_info", p_Msg.fileInfo);
  PutStr(p_W, "link_preview", p_Msg.linkPreview);
  PutInt(p_W, "time_sent", p_Msg.timeSent);
  PutBool(p_W, "is_outgoing", p_Msg.isOutgoing);
  PutBool(p_W, "is_read", p_Msg.isRead);
  PutBool(p_W, "is_delivered", p_Msg.isDelivered);
  PutBool(p_W, "is_edited", p_Msg.isEdited);
  PutBool(p_W, "is_deleted", p_Msg.isDeleted);
  PutBool(p_W, "is_pinned", p_Msg.isPinned);
  PutBool(p_W, "has_mention", p_Msg.hasMention);
  p_W.EndObject();
}
}

std::string EventToJson(const std::shared_ptr<ServiceMessage>& p_Message)
{
  rapidjson::StringBuffer sb;
  JsonWriter w(sb);
  w.StartObject();

  const MessageType type = p_Message->GetMessageType();
  switch (type)
  {
    case ConnectNotifyType:
      {
        const ConnectNotify& n = static_cast<const ConnectNotify&>(*p_Message);
        PutStr(w, "type", "connect");
        PutStr(w, "profile", n.profileId);
        PutBool(w, "success", n.success);
      }
      break;

    case NewChatsNotifyType:
      {
        const NewChatsNotify& n = static_cast<const NewChatsNotify&>(*p_Message);
        PutStr(w, "type", "chats");
        PutStr(w, "profile", n.profileId);
        PutBool(w, "success", n.success);
        w.Key("chats");
        w.StartArray();
        for (const ChatInfo& c : n.chatInfos)
        {
          w.StartObject();
          PutStr(w, "id", c.id);
          PutBool(w, "is_unread", c.isUnread);
          PutBool(w, "is_unread_mention", c.isUnreadMention);
          PutBool(w, "is_muted", c.isMuted);
          PutBool(w, "is_pinned", c.isPinned);
          PutBool(w, "is_archived", c.isArchived);
          PutInt(w, "last_message_time", c.lastMessageTime);
          w.EndObject();
        }

        w.EndArray();
      }
      break;

    case NewContactsNotifyType:
      {
        const NewContactsNotify& n = static_cast<const NewContactsNotify&>(*p_Message);
        PutStr(w, "type", "contacts");
        PutStr(w, "profile", n.profileId);
        PutBool(w, "full_sync", n.fullSync);
        w.Key("contacts");
        w.StartArray();
        for (const ContactInfo& c : n.contactInfos)
        {
          w.StartObject();
          PutStr(w, "id", c.id);
          PutStr(w, "name", c.name);
          PutStr(w, "phone", c.phone);
          PutBool(w, "is_self", c.isSelf);
          PutBool(w, "is_alias", c.isAlias);
          w.EndObject();
        }

        w.EndArray();
      }
      break;

    case NewMessagesNotifyType:
      {
        const NewMessagesNotify& n = static_cast<const NewMessagesNotify&>(*p_Message);
        PutStr(w, "type", "messages");
        PutStr(w, "profile", n.profileId);
        PutBool(w, "success", n.success);
        PutStr(w, "chat_id", n.chatId);
        PutStr(w, "from_msg_id", n.fromMsgId);
        PutBool(w, "cached", n.cached);
        PutBool(w, "sequence", n.sequence);
        w.Key("messages");
        w.StartArray();
        for (const ChatMessage& m : n.chatMessages)
        {
          PutMessage(w, m);
        }

        w.EndArray();
      }
      break;

    case SendMessageNotifyType:
      {
        const SendMessageNotify& n = static_cast<const SendMessageNotify&>(*p_Message);
        PutStr(w, "type", "sent");
        PutStr(w, "profile", n.profileId);
        PutBool(w, "success", n.success);
        PutStr(w, "chat_id", n.chatId);
        w.Key("message");
        PutMessage(w, n.chatMessage);
      }
      break;

    default:
      PutStr(w, "type", "other");
      PutStr(w, "profile", p_Message->profileId);
      PutInt(w, "kind", static_cast<int64_t>(type));
      break;
  }

  w.EndObject();
  return sb.GetString();
}
