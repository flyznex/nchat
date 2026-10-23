// test012.cpp - message delivery state selection
//
// Copyright (c) 2026 Kristofer Berggren
// All rights reserved.
//
// nchat is distributed under the MIT license, see LICENSE for details.

#include <iostream>

#include "deliverystate.h"

#include "unittest.h"

static void TestOutgoing()
{
  // GetDeliveryState(isOutgoing, isPending, isRead, isDelivered)
  unittest::ExpectTrue(GetDeliveryState(true, false, false, false) == DeliveryState::Sent);
  unittest::ExpectTrue(GetDeliveryState(true, false, false, true) == DeliveryState::Delivered);
  unittest::ExpectTrue(GetDeliveryState(true, false, true, false) == DeliveryState::Read);
  // read implies delivered
  unittest::ExpectTrue(GetDeliveryState(true, false, true, true) == DeliveryState::Read);
  // pending messages show nothing
  unittest::ExpectTrue(GetDeliveryState(true, true, false, false) == DeliveryState::None);
  unittest::ExpectTrue(GetDeliveryState(true, true, false, true) == DeliveryState::None);
}

static void TestIncoming()
{
  // incoming: read indicator as before, otherwise nothing
  unittest::ExpectTrue(GetDeliveryState(false, false, true, false) == DeliveryState::Read);
  unittest::ExpectTrue(GetDeliveryState(false, false, false, false) == DeliveryState::None);
  unittest::ExpectTrue(GetDeliveryState(false, false, false, true) == DeliveryState::None);
}

int main()
{
  int rv = 0;
  try
  {
    TestOutgoing();
    TestIncoming();
  }
  catch (const std::exception& ex)
  {
    std::cout << "exception: " << ex.what() << std::endl;
    rv = 1;
  }

  return rv;
}
