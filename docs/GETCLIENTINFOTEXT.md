# GetClientInfoText Transaction (303) Implementation

This document describes the GetClientInfoText transaction based on the GLoarbLine reference implementation.

## Overview

**Transaction ID:** 303  
**Transaction Name:** GetClientInfoText  
**Direction:** Client → Server (request), Server → Client (response)  
**Access Required:** GET_USER_INFO privilege (bit 22)  
**Purpose:** Retrieve detailed information about a connected user

## Request Format

Client sends a transaction with one field:

| Field ID | Name   | Type   | Description                         |
|----------|--------|--------|-------------------------------------|
| 103      | UserId | Uint16 | The user ID to get information about|

## Response Format

Server responds with three fields:

| Field ID | Name       | Type   | Description                    |
|----------|------------|--------|--------------------------------|
| 101      | Data       | Binary | Formatted user info text       |
| 102      | UserName   | String | User's nickname                |
| 104      | UserIconId | String | User's icon ID (as text)       |

## Data Field Format (Field 101)

The Data field contains multi-line formatted text with user details. GLoarbLine has two formats depending on server mode, but the standard format is:

```
Nickname:   <userName>
UserId:     <userid>
Icon:       <usericon>
Away:       <minutes> min <seconds> sec
Client:     <client_version>
Name:       <accountName>
Account:    <accountLogin>
Address:    <ip>
Connected:  <connect_time>
```

### Field Descriptions

- **Nickname:** User's display name
- **UserId:** Protocol user ID (session ID)
- **Icon:** Icon ID number
- **Away:** Time since last activity (calculated from last_activity timestamp)
- **Client:** Client version string (from initial handshake)
- **Name:** Full account name from database
- **Account:** Login name from database
- **Address:** Client IP address
- **Connected:** Connection timestamp

### Transfer Information (Future)

GLoarbLine includes detailed transfer information:

```
-------- File Downloads <count>x -----
<transfer details>

------- Folder Downloads <count>x -----
<transfer details>

--------- File Uploads <count>x ------
<transfer details>

-------- Folder Uploads <count>x -----
<transfer details>
```

Each transfer line format:
```
<filename (21 chars)> <percent> <transferred>/<totalSize> Speed: <speed>/s ETA: <hh:mm:ss>
```

For rhxd MVP, we'll omit transfer information and implement it when file transfers are added.

## Access Control

The requesting user must have the GET_USER_INFO privilege (bit 22).

**Error Response:**
- **Error Code:** 2 (PermissionDenied)
- **Message:** "You are not allowed to get client information."

## Edge Cases

### Client Not Found

If the requested user ID doesn't exist:

**Error Response:**
- **Error Code:** 3 (NotFound)
- **Message:** "Cannot get info for the specified client because it does not exist."

### Ghost Users

According to Virtual1's protocol guide:
> "Note: reply to #303 (get info) will be missing the Nick object if you're getting info on a 'ghost'."

A "ghost" user is one who has disconnected but may still appear in some contexts. In rhxd, we simply return NotFound for disconnected users.

## Implementation Notes

### Session Structure Requirements

The Session struct needs these fields:
- `user_id` - Protocol user ID
- `nickname` - Display name
- `icon_id` - Icon identifier
- `last_activity` - For "Away" calculation
- `connected_at` - Connection timestamp
- `address` - Client IP address
- `account_id` - For looking up account details

### Text Formatting

Use consistent field alignment:
- Field names: Left-aligned, followed by colon and spaces
- Values: Align at character position 12 (after "Nickname:   ")

Example:
```rust
let info_text = format!(
    "Nickname:   {}\n\
     UserId:     {}\n\
     Icon:       {}\n\
     Away:       {} min {} sec\n\
     Client:     {}\n\
     Name:       {}\n\
     Account:    {}\n\
     Address:    {}\n\
     Connected:  {}",
    nickname, user_id, icon_id, minutes, seconds, client_version,
    account_name, account_login, ip, connect_time
);
```

### Time Calculations

**Away Time:**
```rust
let away_duration = SystemTime::now().duration_since(session.last_activity).unwrap_or_default();
let minutes = away_duration.as_secs() / 60;
let seconds = away_duration.as_secs() % 60;
```

**Connected Time:**
```rust
let connected_duration = SystemTime::now().duration_since(session.connected_at).unwrap_or_default();
// Format as timestamp or duration as preferred
```

## References

- **GLoarbLine Implementation:** `/tmp/gloarbline/Apps/Server/Source/HotlineServTrans.cpp:3865-4302`
- **Transaction Definition:** `myTran_GetClientInfoText = 303`
- **Access Privilege:** `myAcc_GetClientInfo` (bit 22)
- **Virtual1's Protocol Guide:** Section on transaction 303

## Related Transactions

- **SetClientUserInfo (304):** Allows users to set custom info text (not implemented in MVP)
- **GetUserNameList (300):** Returns list of all connected users
- **NotifyChangeUser (301):** Broadcasts user changes

## Testing

Test with Hotline client:
1. Connect with two users
2. Right-click on user in list → "Get Info"
3. Verify all fields display correctly
4. Test with user lacking GET_USER_INFO privilege (should fail with error)
5. Test with invalid user ID (should fail with NotFound error)
