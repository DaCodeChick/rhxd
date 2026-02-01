# Access Privilege Bits - Endianness and Bit Reversal

## Overview

The Hotline protocol's 64-bit access privilege structure has an unusual bit encoding scheme that **reverses bits within each byte** on little-endian systems. This document explains this phenomenon based on the mhxd reference implementation.

## The Bit Reversal Phenomenon

### Network Wire Format

The Hotline protocol transmits access privileges as an 8-byte (64-bit) value in the **system's native byte order** (little-endian on x86/x86_64, big-endian on PowerPC). However, there's a critical detail: **on little-endian systems, the bit positions within each byte are reversed**.

This means:
- On **big-endian** systems: bits are transmitted as-is in natural byte order
- On **little-endian** systems: bytes are in little-endian order, but bits within each byte are reversed (bit 0 becomes bit 7, bit 1 becomes bit 6, etc.)

### Little-Endian System Compensation

On little-endian systems, C compilers allocate bitfield members from the **least significant bit (LSB)** to the **most significant bit (MSB)** within each storage unit (byte). To maintain protocol compatibility with the expected bit positions, mhxd implements **bit reversal within each byte** using the formula:

```c
physical_bit_position = 7 - logical_bit_position
```

This reversal happens within each byte boundary (0-7, 8-15, 16-23, etc.).

## Source Code Evidence

### 1. Bitfield Structure (mhxd/src/common/hotline.h:169-236)

The `struct hl_access_bits` is defined with **different member orders** for big-endian vs little-endian:

```c
struct hl_access_bits {
#if WORDS_BIGENDIAN
    u_int32_t delete_files:1,      // Bit 0 (natural order)
              upload_files:1,       // Bit 1
              download_files:1,     // Bit 2
              rename_files:1,       // Bit 3
              move_files:1,         // Bit 4
              create_folders:1,     // Bit 5
              delete_folders:1,     // Bit 6
              rename_folders:1,     // Bit 7
              // ... continues naturally
#else /* assumes little endian */
    u_int32_t rename_folders:1,    // Bit 7 (REVERSED!)
              delete_folders:1,     // Bit 6
              create_folders:1,     // Bit 5
              move_files:1,         // Bit 4
              rename_files:1,       // Bit 3
              download_files:1,     // Bit 2
              upload_files:1,       // Bit 1
              delete_files:1,       // Bit 0
              // ... continues with reversed order within each byte
#endif
};
```

### 2. Bit Manipulation Functions (mhxd/src/acctedit/acctedit.c:143-175)

The `test_bit()` and `inverse_bit()` functions explicitly reverse bit positions:

```c
int
test_bit (void *bufp, int bitno)
{
    unsigned char *buf = (unsigned char *)bufp;
    unsigned char c, m;
    
    c = buf[bitno / 8];        // Get the byte containing the bit
    bitno = bitno % 8;         // Get bit position within byte (0-7)
    bitno = 7 - bitno;         // *** REVERSE THE BIT POSITION ***
    
    // Create mask: m = 1 << bitno
    if (!bitno)
        m = 1;
    else {
        m = 2;
        while (--bitno)
            m *= 2;
    }
    
    return c & m;
}
```

The critical line is `bitno = 7 - bitno;` which implements the bit reversal.

## Bit Mapping Example

The 64-bit value is transmitted in **little-endian byte order** on x86/x86_64 systems, with bits reversed within each byte.

For bits 0-7 (which go into byte 0 in little-endian order):

| Logical Bit | Privilege Name     | Before Reversal | After reverse_bits() |
|-------------|-------------------|-----------------|---------------------|
| 0           | delete_files      | bit 0 (0x01)   | bit 7 (0x80)        |
| 1           | upload_files      | bit 1 (0x02)   | bit 6 (0x40)        |
| 2           | download_files    | bit 2 (0x04)   | bit 5 (0x20)        |
| 3           | rename_files      | bit 3 (0x08)   | bit 4 (0x10)        |
| 4           | move_files        | bit 4 (0x10)   | bit 3 (0x08)        |
| 5           | create_folders    | bit 5 (0x20)   | bit 2 (0x04)        |
| 6           | delete_folders    | bit 6 (0x40)   | bit 1 (0x02)        |
| 7           | rename_folders    | bit 7 (0x80)   | bit 0 (0x01)        |

**Example**: On a little-endian system, the value `0x0000000000000007` (DELETE_FILES | UPLOAD_FILES | DOWNLOAD_FILES) becomes:
1. Little-endian bytes: `[0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]`
2. After bit reversal: `[0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]`

The first byte `0x07` (binary `00000111`) becomes `0xE0` (binary `11100000`) after bit reversal.

This pattern applies to each byte of the 64-bit value.

## Current rhxd Implementation

**Important**: The current rhxd implementation (Rust) does **NOT** implement this bit reversal. It uses:

1. Standard bitflags with normal bit positions (1 << n)
2. Big-endian encoding via `to_be_bytes()`
3. No per-byte bit reversal

### Implications

This means rhxd's access privilege encoding may be **incompatible** with:
- Original C-based Hotline clients
- mhxd servers
- Any implementation that expects the bit-reversed format

### Testing Required

To verify compatibility, we need to:

1. Test rhxd against actual Hotline clients (Frogblast, Nostalgia, etc.)
2. Compare wire format byte-by-byte with mhxd
3. Determine if the bit reversal is actually required or if mhxd's implementation is overcomplicated

## Why This Exists

This bit reversal scheme likely originated from:

1. **Historical C bitfield behavior**: Different compilers on different architectures allocate bitfields differently
2. **Protocol specification ambiguity**: The original Hotline protocol spec may not have been clear about bit ordering within bytes
3. **Mac OS Classic heritage**: The protocol was designed on big-endian PowerPC Macs, where bitfields naturally matched the wire format

## Recommendations

### Option 1: Implement Bit Reversal (Maximum Compatibility)

Implement the same `7 - bitno` reversal for little-endian systems to match mhxd exactly.

```rust
fn encode_access_le(access: u64) -> [u8; 8] {
    let mut bytes = access.to_le_bytes();
    // Reverse bits within each byte
    for byte in &mut bytes {
        *byte = byte.reverse_bits();
    }
    bytes
}
```

### Option 2: Keep Current Implementation (Simplified)

If testing shows that modern clients don't actually require bit reversal, keep the current simple big-endian encoding.

### Option 3: Auto-Detect (Complex)

Detect the client version and apply bit reversal only for clients that need it.

## References

- mhxd source: https://github.com/Schala/mhxd
- Key file: `src/common/hotline.h` (lines 169-236)
- Key file: `src/acctedit/acctedit.c` (lines 143-175)
- Bit mapping: `src/acctedit/acctedit.c` (lines 91-131)
