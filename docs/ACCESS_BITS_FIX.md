# Access Privilege Encoding - Before and After

## The Problem

The original implementation used **big-endian byte order** for access privileges, which is incorrect. The Hotline protocol uses **little-endian byte order** with bit reversal on little-endian systems.

## Before (Incorrect)

```rust
// Old implementation
let access_bytes = guest_access.bits().to_be_bytes().to_vec();
```

For `DELETE_FILES | UPLOAD_FILES | DOWNLOAD_FILES` (bits 0, 1, 2 = `0x0000000000000007`):
- Wire format: `[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0]`
- Bits are in the **last byte** (byte 7)
- **WRONG** - mhxd expects bits in **first byte** (byte 0)

## After (Correct)

```rust
// New implementation  
let access_bytes = guest_access.to_wire_format().to_vec();

// Where to_wire_format() does:
#[cfg(target_endian = "little")]
{
    let mut bytes = self.bits().to_le_bytes();  // Little-endian!
    for byte in &mut bytes {
        *byte = byte.reverse_bits();  // Reverse bits within each byte
    }
    bytes
}
```

For `DELETE_FILES | UPLOAD_FILES | DOWNLOAD_FILES` (bits 0, 1, 2 = `0x0000000000000007`):
1. Little-endian bytes: `[0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]`
2. After bit reversal: `[0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]`
3. Bits are in the **first byte** (byte 0)
4. **CORRECT** - matches mhxd expectations!

## Wire Format Breakdown

Byte 0 before reversal: `0x07` = `00000111` (bits 0, 1, 2 set)
Byte 0 after reversal:  `0xE0` = `11100000` (bits 7, 6, 5 set)

This is exactly what mhxd produces on little-endian systems.

## Test Results

```
✓ test_wire_format_roundtrip ... ok
✓ test_guest_access ... ok  
✓ test_bit_reversal_little_endian ... ok
✓ test_multiple_bits_little_endian ... ok
```

All tests pass with the correct little-endian + bit reversal encoding.
