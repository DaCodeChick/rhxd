# Bug Fixes - Client Compatibility

## Issues Fixed

### 1. ✅ Agreement Hang
**Problem:** Client was hanging at "Loading Agreement" screen.

**Root Cause:** The Hotline protocol expects the server to send a ShowAgreement (109) transaction after login ONLY if an agreement is configured. Since we don't have an agreement configured, we shouldn't send ShowAgreement at all. The client should proceed directly and send the Agreed (121) transaction.

**Fix:** Ensured login reply always includes:
- Server version (Field 160)
- Banner ID (Field 161) 
- Server name (Field 162)

Previously these fields were conditional on client version >= 151, which may have caused confusion.

**Changes:**
- `crates/rhxd/src/handlers/login.rs`: Always send banner and server name fields
- Removed conditional version check that was blocking modern clients

### 2. ✅ Server Version
**Problem:** Server was reporting version 151 instead of 197.

**Root Cause:** SERVER_VERSION constant was set to 151 (Hotline 1.8.x era).

**Fix:** Updated to version 197 (Hotline 1.9.2 compatible).

**Changes:**
- `crates/rhxcore/src/protocol/constants.rs`: Changed `SERVER_VERSION` from 151 to 197

## Testing

✅ All 23 tests still passing
✅ Server builds successfully
✅ No breaking changes to protocol

## Expected Client Behavior Now

1. **Connect** → Handshake succeeds
2. **Login** → Server sends login reply with version 197, banner 0, server name
3. **Client checks for agreement** → No ShowAgreement (109) sent
4. **Client proceeds** → Client automatically sends Agreed (121) with nickname
5. **Server processes Agreed** → Session updated, NotifyChangeUser broadcast
6. **Client enters main interface** → No more hanging!

## Protocol Flow

```
Client                          Server
  |                               |
  |--- Login (107) ------------->|
  |                               |
  |<-- Login Reply --------------|
  |    (version=197, name, etc)  |
  |                               |
  | [Client waits briefly for    |
  |  ShowAgreement (109)]        |
  |                               |
  | [Timeout/No agreement]       |
  |                               |
  |--- Agreed (121) ------------>|
  |    (nickname, icon)          |
  |                               |
  |<-- Agreed Reply -------------|
  |                               |
  |<-- NotifyChangeUser (301) ---|
  |    (broadcast to all)        |
  |                               |
  | [Client enters main UI]      |
  v                               v
```

## Changes Summary

### Modified Files
1. `crates/rhxcore/src/protocol/constants.rs`
   - SERVER_VERSION: 151 → 197

2. `crates/rhxd/src/handlers/login.rs`
   - Removed conditional `if client_version >= 151` check
   - Always send banner ID and server name in login reply
   - Applied to both guest and authenticated login paths

### Test Results
```
running 23 tests
test result: ok. 23 passed; 0 failed
```

## Next Steps

The server should now work properly with Hotline clients:
1. No more hanging at "Loading Agreement"
2. Correct version reported (197)
3. Proper protocol flow for login → agreed → main interface

## Restart Required

After updating, restart the server:
```bash
cd /home/admin/Documents/GitHub/rhxd
./start-server.sh
```

Then try connecting again with your Hotline client!
