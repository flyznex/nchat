// uihistoryview.cpp
//
// Copyright (c) 2019-2025 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include "uihistoryview.h"

#include <algorithm>
#include <vector>

#include "appconfig.h"
#include "apputil.h"
#include "deliverystate.h"
#include "fileutil.h"
#include "imagecache.h"
#include "imagegeom.h"
#include "kittyencoder.h"
#include "log.h"
#include "protocolutil.h"
#include "strutil.h"
#include "termgfx.h"
#include "timeutil.h"
#include "uicolorconfig.h"
#include "uiconfig.h"
#include "uiimagerow.h"
#include "uimodel.h"

UiHistoryView::UiHistoryView(const UiViewParams& p_Params)
  : UiViewBase(p_Params)
{
  if (m_Enabled)
  {
    int hpad = (m_X == 0) ? 0 : 1;
    int vpad = 1;
    int paddedY = m_Y + vpad;
    int paddedX = m_X + hpad;
    m_PaddedH = m_H - (vpad * 2);
    m_PaddedW = m_W - (hpad * 2);
    m_PaddedWin = newwin(m_PaddedH, m_PaddedW, paddedY, paddedX);

    static int attributeTextNormal = UiColorConfig::GetAttribute("history_text_attr");
    static int colorPairTextRecv = UiColorConfig::GetColorPair("history_text_recv_color");
    werase(m_Win);
    wbkgd(m_Win, attributeTextNormal | colorPairTextRecv | ' ');
    wrefresh(m_Win);
  }
}

UiHistoryView::~UiHistoryView()
{
  if (m_PaddedWin != nullptr)
  {
    delwin(m_PaddedWin);
    m_PaddedWin = nullptr;
  }
}

void UiHistoryView::Draw()
{
  if (!m_Enabled || !m_Dirty) return;
  m_Dirty = false;

  curs_set(0);
  ImageCache::BeginDrawPass();

  static int colorPairTextSent = UiColorConfig::GetColorPair("history_text_sent_color");
  static int colorPairTextRecv = UiColorConfig::GetColorPair("history_text_recv_color");
  static int colorPairTextQuoted = UiColorConfig::GetColorPair("history_text_quoted_color");
  static int colorPairTextReaction = UiColorConfig::GetColorPair("history_text_reaction_color");
  static int colorPairTextAttachment = UiColorConfig::GetColorPair("history_text_attachment_color");
  static int attributeTextNormal = UiColorConfig::GetAttribute("history_text_attr");
  static int attributeTextSelected = UiColorConfig::GetAttribute("history_text_attr_selected");

  static int colorPairNameSent = UiColorConfig::GetColorPair("history_name_sent_color");
  static int colorPairNameRecv = UiColorConfig::GetColorPair("history_name_recv_color");
  static int attributeNameNormal = UiColorConfig::GetAttribute("history_name_attr");
  static int attributeNameSelected = UiColorConfig::GetAttribute("history_name_attr_selected");

  static std::wstring attachmentIndicator =
    StrUtil::ToWString(UiConfig::GetStr("attachment_indicator") + " ");
  static std::wstring quoteIndicator = L"> ";
  static const wchar_t imageRowMark = L'\x01';
  static const int maxPhotoRows =
    std::max(1, std::min(UiConfig::GetNum("inline_image_max_rows"), KittyEncoder::GetDiacriticCount()));
  static const int maxStickerRows =
    std::max(1, std::min(UiConfig::GetNum("inline_sticker_max_rows"), KittyEncoder::GetDiacriticCount()));

  std::pair<std::string, std::string>& currentChat = m_Model->GetCurrentChatLocked();
  const bool emojiEnabled = m_Model->GetEmojiEnabledLocked();
  static const bool developerMode = AppUtil::GetDeveloperMode();
  static const bool alignSent = UiConfig::GetBool("sent_align_right");
  // ponytail: fixed 75% cap when aligned; make it a config key if it needs tuning
  const int maxW = alignSent ? std::max(1, m_PaddedW * 3 / 4) : m_PaddedW;

  std::vector<std::string>& messageVec =
    m_Model->GetMessageVecLocked(currentChat.first, currentChat.second);
  std::unordered_map<std::string, ChatMessage>& messages =
    m_Model->GetMessagesLocked(currentChat.first, currentChat.second);
  int messageOffset = std::max(m_Model->GetMessageOffsetLocked(currentChat.first, currentChat.second), 0);

  werase(m_PaddedWin);
  wbkgd(m_PaddedWin, attributeTextNormal | colorPairTextRecv | ' ');

  m_HistoryShowCount = 0;

  bool firstMessage = true;
  int y = m_PaddedH - 1;
  for (auto it = std::next(messageVec.begin(), messageOffset); it != messageVec.end(); ++it)
  {
    bool isSelectedMessage = firstMessage && m_Model->GetSelectMessageActiveLocked();

    auto msgIt = messages.find(*it);
    if (msgIt == messages.end())
    {
      LOG_WARNING("message %s missing", it->c_str());
      continue;
    }

    ChatMessage& msg = msgIt->second;

    int attributeText = isSelectedMessage ? attributeTextSelected : attributeTextNormal;
    int colorPairText = [&]()
    {
      if (msg.isOutgoing) return colorPairTextSent;

      if (msg.senderId == currentChat.second) return colorPairTextRecv;

      static bool isUserColor = UiColorConfig::IsUserColor("history_text_recv_group_color");
      if (!isUserColor)
      {
        static int colorPairGroup = UiColorConfig::GetColorPair("history_text_recv_group_color");
        return colorPairGroup;
      }

      int colorPairGroup = UiColorConfig::GetUserColorPair("history_text_recv_group_color",
                                                           msg.senderId);
      return colorPairGroup;
    }();

    std::vector<std::wstring> wlines;
    if (!msg.text.empty())
    {
      std::string text = msg.text;
      StrUtil::SanitizeMessageStr(text);
      if (!emojiEnabled)
      {
        text = StrUtil::Textize(text);
      }

      wlines = StrUtil::WordWrap(StrUtil::ToWString(text), maxW, false, false, false, 2);
    }

    // Quoted message
    if (!msg.quotedId.empty())
    {
      std::string quotedText;
      auto quotedIt = messages.find(msg.quotedId);
      if (quotedIt != messages.end())
      {
        if (!quotedIt->second.text.empty())
        {
          quotedText = StrUtil::Split(quotedIt->second.text, '\n').at(0);
          if (!emojiEnabled)
          {
            quotedText = StrUtil::Textize(quotedText);
          }
        }
        else if (!quotedIt->second.fileInfo.empty())
        {
          FileInfo fileInfo = ProtocolUtil::FileInfoFromHex(quotedIt->second.fileInfo);
          quotedText = FileUtil::BaseName(fileInfo.filePath);
        }
      }
      else
      {
        m_Model->FetchCachedMessageLocked(currentChat.first, currentChat.second, msg.quotedId);
        quotedText = "";
      }

      int maxQuoteLen = maxW - 3;
      std::wstring quote = quoteIndicator + StrUtil::ToWString(quotedText);
      if (StrUtil::WStringWidth(quote) > maxQuoteLen)
      {
        quote = StrUtil::TrimPadWString(quote, maxQuoteLen) + L"...";
      }

      wlines.insert(wlines.begin(), quote);
    }

    // File attachment
    int imageId = 0;
    CellGrid imageGrid;
    if (!msg.fileInfo.empty())
    {
      FileInfo fileInfo = ProtocolUtil::FileInfoFromHex(msg.fileInfo);

      // special case handling selection-triggered download, and handling cache's old setting
      static const bool isAttachmentPrefetchAll =
        (AppConfig::GetNum("attachment_prefetch") == AttachmentPrefetchAll);
      static const bool isAttachmentPrefetchSelected =
        (AppConfig::GetNum("attachment_prefetch") == AttachmentPrefetchSelected);
      if (isAttachmentPrefetchAll || (isSelectedMessage && isAttachmentPrefetchSelected))
      {
        if (!UiModel::IsAttachmentDownloaded(fileInfo) && UiModel::IsAttachmentDownloadable(fileInfo))
        {
          m_Model->DownloadAttachmentLocked(currentChat.first, currentChat.second, *it,
                                            fileInfo.fileId, DownloadFileActionNone);
          fileInfo = ProtocolUtil::FileInfoFromHex(msg.fileInfo);
        }
      }

      const ImageKind imageKind =
        TermGfx::IsEnabled() ? ImageGeom::DetectKind(fileInfo.fileType, fileInfo.filePath) : ImageKind::None;
      if (imageKind != ImageKind::None)
      {
        if (!UiModel::IsAttachmentDownloaded(fileInfo) && UiModel::IsAttachmentDownloadable(fileInfo))
        {
          m_Model->DownloadAttachmentLocked(currentChat.first, currentChat.second, *it,
                                            fileInfo.fileId, DownloadFileActionNone);
          fileInfo = ProtocolUtil::FileInfoFromHex(msg.fileInfo);
        }

        ImageInfo imageInfo;
        if (UiModel::IsAttachmentDownloaded(fileInfo) &&
            (ImageCache::Request(fileInfo.filePath, &imageInfo) == ImageCache::State::Ready))
        {
          int cellW = 0;
          int cellH = 0;
          TermGfx::GetCellPixelSize(cellW, cellH);
          const int maxRows = std::min((imageKind == ImageKind::Sticker) ? maxStickerRows : maxPhotoRows,
                                       std::max(1, m_PaddedH / 2));
          const int maxCols = std::min(maxW, KittyEncoder::GetDiacriticCount());
          imageGrid = ImageGeom::FitToCells(imageInfo.pxWidth, imageInfo.pxHeight, cellW, cellH,
                                            maxCols, maxRows);
          if (imageGrid.cols > 0)
          {
            imageId = ImageCache::Acquire(fileInfo.filePath, imageInfo, imageGrid.cols, imageGrid.rows);
          }
        }
      }

      std::string fileName = FileUtil::BaseName(fileInfo.filePath);
      std::string fileStatus;
      if (fileInfo.fileStatus == FileStatusNone)
      {
        // should not happen
        static const std::string statusNone = " -";
        fileStatus = statusNone;
      }
      else if (fileInfo.fileStatus == FileStatusNotDownloaded)
      {
        static const std::string statusNotDownloaded = " " + UiConfig::GetStr("downloadable_indicator");
        fileStatus = statusNotDownloaded;
      }
      else if (fileInfo.fileStatus == FileStatusDownloaded)
      {
        static const std::string statusDownloaded = "";
        fileStatus = statusDownloaded;
      }
      else if (fileInfo.fileStatus == FileStatusDownloading)
      {
        static const std::string statusDownloading = " " + UiConfig::GetStr("syncing_indicator");
        fileStatus = statusDownloading;
      }
      else if (fileInfo.fileStatus == FileStatusDownloadFailed)
      {
        static const std::string statusDownloadFailed = " " + UiConfig::GetStr("failed_indicator");
        fileStatus = statusDownloadFailed;
      }

      if (imageId != 0)
      {
        for (int row = imageGrid.rows - 1; row >= 0; --row)
        {
          wlines.insert(wlines.begin(), std::wstring{ imageRowMark, static_cast<wchar_t>(row) });
        }
      }
      else
      {
        std::wstring fileStr = attachmentIndicator + StrUtil::ToWString(fileName + fileStatus);
        wlines.insert(wlines.begin(), fileStr);
      }
    }

    // Reactions
    int reactionLines = 0;
    static bool reactionsEnabled = UiConfig::GetBool("reactions_enabled");
    if (reactionsEnabled)
    {
      std::string selfEmoji;
      auto sit = msg.reactions.senderEmojis.find(s_ReactionsSelfId);
      if (sit != msg.reactions.senderEmojis.end())
      {
        selfEmoji = sit->second;
      }

      // Allow also if we have self emoji, even if not yet consolidated into count
      if (!msg.reactions.emojiCounts.empty() || !selfEmoji.empty())
      {
        bool foundSelf = false;
        std::string reactionsText;
        std::multimap<float, std::string> emojiCountsSorted;
        for (const auto& emojiCount : msg.reactions.emojiCounts)
        {
          float count = emojiCount.second;
          if (emojiCount.first == selfEmoji)
          {
            count += 0.1; // for equal count, prioritize own selected reaction
            foundSelf = true;
          }

          emojiCountsSorted.insert(std::make_pair(count, emojiCount.first));
        }

        if (!foundSelf && !selfEmoji.empty())
        {
          LOG_DEBUG("insert missing reaction for self");
          emojiCountsSorted.insert(std::make_pair(1.1, selfEmoji));
        }

        bool firstReaction = true;
        for (auto emojiCount = emojiCountsSorted.rbegin(); emojiCount != emojiCountsSorted.rend(); ++emojiCount)
        {
          reactionsText += (firstReaction ? " " : "  ");
          if (emojiCount->second == selfEmoji)
          {
            // Highlight own reaction emoji
            reactionsText += "" + emojiCount->second + "*";
          }
          else
          {
            reactionsText += emojiCount->second;
          }

          if (emojiCount->first > 1.5)
          {
            reactionsText += " " + FileUtil::GetSuffixedCount(static_cast<ssize_t>(emojiCount->first));
          }

          firstReaction = false;
        }

        if (!reactionsText.empty())
        {
          if (!emojiEnabled)
          {
            reactionsText = StrUtil::Textize(reactionsText);
          }

          const int maxReactionsLen = maxW - 4;
          std::wstring reactions = StrUtil::ToWString(reactionsText);
          if (StrUtil::WStringWidth(reactions) > maxReactionsLen)
          {
            reactions = StrUtil::TrimPadWString(reactions, maxReactionsLen) + L"... ";
          }
          else
          {
            reactions += L" ";
          }

          wlines.insert(wlines.end(), reactions);
          reactionLines = 1;
        }
      }
    }

    const int maxMessageLines = (m_PaddedH - 1);
    if (firstMessage && ((int)wlines.size() > maxMessageLines))
    {
      wlines.resize(std::max(0, maxMessageLines - 1)); // tiny terminals: pane height can be <= 1
      wlines.push_back(L"[...]");
      reactionLines = 0;
    }

    std::string name = m_Model->GetContactNameLocked(currentChat.first, msg.senderId);
    if (!emojiEnabled)
    {
      name = StrUtil::Textize(name);
    }

    std::wstring wsender = StrUtil::ToWString(name);
    std::wstring wtime;
    if (developerMode)
    {
      wtime = L" (" + StrUtil::ToWString(std::to_string(msg.timeSent)) + L")";
    }
    else
    {
      if (msg.timeSent != std::numeric_limits<int64_t>::max())
      {
        wtime = L" (" + StrUtil::ToWString(TimeUtil::GetTimeString(msg.timeSent, false /* p_IsExport */)) + L")";
      }
    }

    static const std::string readIndicator = " " + UiConfig::GetStr("read_indicator");
    static const std::string sentIndicator = " " + UiConfig::GetStr("sent_indicator");
    static const std::string deliveredIndicator = " " + UiConfig::GetStr("delivered_indicator");
    static const std::string editedIndicator = " " + UiConfig::GetStr("edited_indicator");
    static const std::string pinnedIndicator = " " + UiConfig::GetStr("pinned_indicator");
    const bool isPending = (msg.timeSent == std::numeric_limits<int64_t>::max());
    std::string receipt;
    switch (GetDeliveryState(msg.isOutgoing, isPending, msg.isRead, msg.isDelivered))
    {
      case DeliveryState::Read: receipt = readIndicator; break;
      case DeliveryState::Delivered: receipt = deliveredIndicator; break;
      case DeliveryState::Sent: receipt = sentIndicator; break;
      case DeliveryState::None:
      default: break;
    }
    std::wstring wreceipt = StrUtil::ToWString(receipt);
    std::wstring wedited = StrUtil::ToWString(msg.isEdited ? editedIndicator : "");
    std::wstring wpinned = StrUtil::ToWString(msg.isPinned ? pinnedIndicator : "");
    std::wstring wheader = wsender + wtime + wreceipt + wedited + wpinned;

    if (developerMode)
    {
      wheader = wheader +
        L" msg " + StrUtil::ToWString(msg.id) +
        L" user " + StrUtil::ToWString(msg.senderId);
    }

    // block = widest line of this message; aligned mode pads and offsets by it instead of the full pane
    int blockW = m_PaddedW;
    if (alignSent)
    {
      blockW = StrUtil::WStringWidth(wheader);
      for (const std::wstring& wl : wlines)
      {
        if ((imageId != 0) && (wl.size() == 2) && (wl[0] == imageRowMark)) continue;
        blockW = std::max(blockW, StrUtil::WStringWidth(wl));
      }
      if (imageId != 0) blockW = std::max(blockW, imageGrid.cols);
      blockW = std::min(blockW, maxW);
    }
    const int x0 = (alignSent && msg.isOutgoing) ? (m_PaddedW - blockW) : 0;

    for (auto wline = wlines.rbegin(); wline != wlines.rend(); ++wline)
    {
      if ((imageId != 0) && (wline->size() == 2) && ((*wline)[0] == imageRowMark))
      {
        UiImageRow::Draw(m_PaddedWin, y, x0, static_cast<int>((*wline)[1]), imageId, imageGrid.cols);
        if (--y < 0) break;
        continue;
      }

      bool isAttachment = (wline->rfind(attachmentIndicator, 0) == 0);
      bool isQuote = (wline->rfind(quoteIndicator, 0) == 0);
      bool isReaction = (reactionLines == 1) && (std::distance(wline, wlines.rbegin()) == 0);

      if (isAttachment)
      {
        wattron(m_PaddedWin, attributeText | colorPairTextAttachment);
      }
      else if (isQuote)
      {
        wattron(m_PaddedWin, attributeText | colorPairTextQuoted);
      }
      else if (isReaction)
      {
        wattron(m_PaddedWin, attributeTextNormal | colorPairTextReaction);
      }
      else
      {
        wattron(m_PaddedWin, attributeText | colorPairText);
      }

      const std::wstring wdisp = isReaction ? *wline : StrUtil::TrimPadWString(*wline, blockW);
      mvwaddnwstr(m_PaddedWin, y, x0, wdisp.c_str(), std::min((int)wdisp.size(), m_PaddedW));

      if (isAttachment)
      {
        wattroff(m_PaddedWin, attributeText | colorPairTextAttachment);
      }
      else if (isQuote)
      {
        wattroff(m_PaddedWin, attributeText | colorPairTextQuoted);
      }
      else if (isReaction)
      {
        wattroff(m_PaddedWin, attributeTextNormal | colorPairTextReaction);
      }
      else
      {
        wattroff(m_PaddedWin, attributeText | colorPairText);
      }

      if (--y < 0) break;
    }

    if (y < 0) break;

    int attributeName = isSelectedMessage ? attributeNameSelected : attributeNameNormal;
    int colorPairName = [&]()
    {
      if (msg.isOutgoing) return colorPairNameSent;

      if (msg.senderId == currentChat.second) return colorPairNameRecv;

      static bool isUserColor = UiColorConfig::IsUserColor("history_name_recv_group_color");
      if (!isUserColor)
      {
        static int colorPairGroup = UiColorConfig::GetColorPair("history_name_recv_group_color");
        return colorPairGroup;
      }

      int colorPairGroup = UiColorConfig::GetUserColorPair("history_name_recv_group_color",
                                                           msg.senderId);
      return colorPairGroup;
    }();

    // selected rows keep a reverse bar over the whole block; otherwise the attribute covers the name only
    const int headerAttr = isSelectedMessage ? attributeName : 0;
    wattron(m_PaddedWin, headerAttr | colorPairName);

    m_Model->MarkReadLocked(currentChat.first, currentChat.second, *it, (!msg.isOutgoing && !msg.isRead));

    std::wstring wdisp = StrUtil::TrimPadWString(wheader, blockW);
    if (isSelectedMessage)
    {
      mvwaddnwstr(m_PaddedWin, y, x0, wdisp.c_str(), std::min((int)wdisp.size(), m_PaddedW));
    }
    else
    {
      // wdisp starts with the name unless the name alone overflows, then it is all name
      const std::wstring wname = (StrUtil::WStringWidth(wsender) >= blockW) ? wdisp : wsender;
      const std::wstring wrest = wdisp.substr(wname.size());
      wattron(m_PaddedWin, attributeName);
      mvwaddnwstr(m_PaddedWin, y, x0, wname.c_str(), wname.size());
      wattroff(m_PaddedWin, attributeName);
      mvwaddnwstr(m_PaddedWin, y, x0 + StrUtil::WStringWidth(wname), wrest.c_str(), wrest.size());
    }

    wattroff(m_PaddedWin, headerAttr | colorPairName);

    ++m_HistoryShowCount;

    if (--y < 0) break;

    if (--y < 0) break;

    firstMessage = false;
  }

  wrefresh(m_PaddedWin);
}

int UiHistoryView::GetHistoryShowCount()
{
  return m_HistoryShowCount;
}
