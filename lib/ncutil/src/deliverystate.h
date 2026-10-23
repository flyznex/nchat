// deliverystate.h
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#pragma once

enum class DeliveryState { None, Sent, Delivered, Read };

inline DeliveryState GetDeliveryState(bool p_IsOutgoing, bool p_IsPending, bool p_IsRead, bool p_IsDelivered)
{
  if (p_IsRead) return DeliveryState::Read;
  if (!p_IsOutgoing || p_IsPending) return DeliveryState::None;
  return p_IsDelivered ? DeliveryState::Delivered : DeliveryState::Sent;
}
