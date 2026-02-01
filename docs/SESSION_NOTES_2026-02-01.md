# Development Session Notes - February 1, 2026

## Access Privilege Bit Reversal Implementation

### Problem Identified
The Hotline protocol uses an unusual encoding for access privileges on little-endian systems:
- 64-bit value transmitted in **little-endian byte order** (not big-endian as initially thought)
- **Bits within each byte are reversed** on little-endian systems
- This matches the mhxd reference implementation's bitfield layout

### Root Cause
The original implementation used `to_be_bytes()` which:
- Put bits 0-7 in byte 7 (last byte)
- mhxd expects bits 0-7 in byte 0 (first byte)
- Result: Access privileges were in the wrong byte positions

### Solution Implemented
Changed to use **little-endian byte order with bit reversal**:

```rust
// Before (WRONG):
let access_bytes = guest_access.bits().to_be_bytes().to_vec();
// Result for bits 0,1,2: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0]

// After (CORRECT):
let access_bytes = guest_access.to_wire_format().to_vec();
// Result for bits 0,1,2: [0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```

Added methods to `AccessPrivileges`:
- `to_wire_format()` - Encodes with bit reversal on little-endian systems
- `from_wire_format()` - Decodes with bit reversal on little-endian systems

### Files Modified
- `crates/rhxcore/src/types/access.rs` - Added wire format methods and tests
- `crates/rhxd/src/handlers/login.rs` - Use `to_wire_format()` for guest and user logins
- `crates/rhxd/src/connection/handler.rs` - Use `to_wire_format()` for UserAccess transaction (354)
- `crates/rhxcore/src/codec/field_codec.rs` - Updated UserAccess field decoding
- `docs/ACCESS_BITS.md` - Comprehensive documentation of the bit reversal phenomenon
- `docs/ACCESS_BITS_FIX.md` - Before/after comparison

### Test Results
All tests pass:
- ✓ test_wire_format_roundtrip
- ✓ test_guest_access
- ✓ test_bit_reversal_little_endian
- ✓ test_multiple_bits_little_endian

Verified encoding matches mhxd exactly for all test cases.

## Ghost User Bug Fix

### Problem
Users were seeing duplicate entries in the user list:
1. Their own entry (from GetUserNameList response)
2. A "ghost" entry (from UserJoined broadcast sent to themselves)

Result: User appeared twice in the client's user list, with the duplicate showing only as an icon without a name.

### Root Cause
When a user completed the agreed transaction:
1. Server broadcast `UserJoined` to **ALL** connected users
2. This included broadcasting to the user who just joined
3. User received NotifyChangeUser about themselves
4. When they requested GetUserNameList, they got themselves again
5. Client showed the user twice in the list

### Solution Implemented
Modified the broadcast handler to **skip sending UserJoined notifications to the user who just joined**:

```rust
BroadcastMessage::UserJoined { user_id: joined_user_id, nickname } => {
    // Don't send the notification to the user who just joined
    if joined_user_id == user_id {
        None  // Skip this broadcast for the joining user
    } else {
        // Send NotifyChangeUser to other users
        Some(Transaction { ... })
    }
}
```

This ensures:
- Users only see themselves once (from GetUserNameList response)
- Other users get notified when someone joins (via NotifyChangeUser)
- No duplicate entries in the user list

### Files Modified
- `crates/rhxd/src/connection/handler.rs` - Skip UserJoined broadcast to self

### Testing
Confirmed with Hotline client:
- ✓ User list works correctly
- ✓ No ghost/duplicate users
- ✓ Chat works (somewhat)

## Summary

Two major bugs fixed:
1. **Access privilege encoding** - Now uses correct little-endian + bit reversal format matching mhxd
2. **Ghost user bug** - Users no longer receive their own join notification

Both fixes improve protocol compatibility with standard Hotline clients.
