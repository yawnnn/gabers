# LD
```rust
// LD r8,r8
0x6D => Instruction::LD(Reg8(L), Reg8(L)),
// LD r8,n8
0x3E => Instruction::LD(Reg8(A), Imm8),
// LD r16,n16
0x01 => Instruction::LD(Reg16(BC), Imm16),
// LD [HL],r8
0x75 => Instruction::LD(Addr16(Reg16(HL)), Reg8(L)),
// LD [HL],n8
0x36 => Instruction::LD(Addr16(Reg16(HL)), Imm8),
// LD r8,[HL]
0x4E => Instruction::LD(Reg8(C), Addr16(Reg16(HL))),
// LD [r16],A
0x12 => Instruction::LD(Addr16(Reg16(DE)), Reg8(A)),
// LD [n16],A
0xEA => Instruction::LD(Addr16(Imm16), Reg8(A)),
// LD A,[r16]
0x1A => Instruction::LD(Reg8(A), Addr16(Reg16(DE))),
// LD A,[n16]
0xFA => Instruction::LD(Reg8(A), Addr16(Imm16)),
// LD [HLI],A
0x22 => Instruction::LD(Addr8(HLI), Reg8(A)),
// LD [HLD],A
0x32 => Instruction::LD(Addr8(HLD), Reg8(A)),
// LD A,[HLI]
0x2A => Instruction::LD(Reg8(A), Addr8(HLI)),
// LD A,[HLD]
0x3A => Instruction::LD(Reg8(A), Addr8(HLD)),

// LDH [n16],A
// LDH [C],A
0xE2 => Instruction::LDH(Addr8(Reg8(C)), Reg8(A)),
// LDH A,[n16]
// LDH A,[C]
0xF2 => Instruction::LDH(Reg8(A), Addr8(Reg8(C))),

// remaining 
0xF9 => Instruction::LD(Reg16(SP), Reg16(HL)),
0x08 => Instruction::LD(Addr16(Imm16), Reg16(SP)),
0xF8 => Instruction::LD(Reg16(HL), SPMod(Some(Imm8S))),
```

# ADC
```rust
// ADC A, r8
0x8A => Instruction::ADC(Reg8(A), Reg8(D)),
// ADC A, [HL]
0x8E => Instruction::ADC(Reg8(A), Addr16(Reg16(HL))),
// ADC A, n8
0xCE => Instruction::ADC(Reg8(A), Imm8),
```

# ADD
```rust
// ADD A, r8
0x80 => Instruction::ADD(Reg8(A), Reg8(B)),
// ADD A, [HL]
0x86 => Instruction::ADD(Reg8(A), Addr16(Reg16(HL))),
// ADD A, n8
0xC6 => Instruction::ADD(Reg8(A), Imm8),
// ADD HL, r16
0x29 => Instruction::ADD(Reg16(HL), Reg16(HL)),
// ADD HL, SP
0x39 => Instruction::ADD(Reg16(HL), Reg16(SP)),
// ADD SP, e8
0xE8 => Instruction::ADD(Reg16(SP), Imm8S),
```

# AND
```rust
// AND A, r8
0xA1 => Instruction::AND(Reg8(A), Reg8(C)),
// AND A, [HL] 
0xA6 => Instruction::AND(Reg8(A), Addr16(Reg16(HL))),
// AND A, n8
0xE6 => Instruction::AND(Reg8(A), Imm8),
```

# CALL
```rust
// CALL n16
0xCD => Instruction::CALL(Addr16(Imm16)),
// CALL cc, n16
0xDC => Instruction::CALL(Flag(Carry), Addr16(Imm16)),
```

# DEC
```rust
// DEC r8
0x15 => Instruction::DEC(Reg8(D)),
// DEC [HL]
0x35 => Instruction::DEC(Addr16(Reg16(HL))),
// DEC r16
0x0B => Instruction::DEC(Reg16(BC)),
// DEC SP
0x3B => Instruction::DEC(Reg16(SP)),
```

# INC
```rust
// INC r8
0x0C => Instruction::INC(Reg8(C)),
// INC [HL]
0x34 => Instruction::INC(Addr16(Reg16(HL))),
// INC r16
0x03 => Instruction::INC(Reg16(BC)),
// INC SP
0x33 => Instruction::INC(Reg16(SP)),
```

# JP
```rust
// JP HL
0xE9 => Instruction::JP(Reg16(HL)),
// JP n16
0xC3 => Instruction::JP(Addr16(Imm16)),
// JP cc, n16
0xDA => Instruction::JP(Flag(Carry), Addr16(Imm16)),
```

# OR
```rust
// OR A, r8
0xB2 => Instruction::OR(Reg8(A), Reg8(D)),
// OR A, [HL]
0xB6 => Instruction::OR(Reg8(A), Addr16(Reg16(HL))),
// OR A, n8
0xF6 => Instruction::OR(Reg8(A), Imm8),
```

# POP
```rust
// POP AF
0xF1 => Instruction::POP(Reg16(AF)),
// POP r16
0xD1 => Instruction::POP(Reg16(DE)),
```

# PUSH
```rust
// PUSH AF
0xF5 => Instruction::PUSH(Reg16(AF)),
// PUSH r16
0xE5 => Instruction::PUSH(Reg16(HL)),
```

# RET
```rust
// RET
0xC9 => Instruction::RET(),
// RET cc
0xD8 => Instruction::RET(Flag(Carry)),
```

# SUB
```rust
// SUB A, r8
0x90 => Instruction::SUB(Reg8(A), Reg8(B)),
// SUB A, [HL]
0x96 => Instruction::SUB(Reg8(A), Addr16(Reg16(HL))),
// SUB A, n8
0xD6 => Instruction::SUB(Reg8(A), Imm8)
```

# XOR
```rust
// XOR A, r8
0xA8 => Instruction::XOR(Reg8(A), Reg8(B)),
// XOR A, [HL]
0xAE => Instruction::XOR(Reg8(A), Addr16(Reg16(HL))),
// XOR A, n8
0xEE => Instruction::XOR(Reg8(A), Imm8),
```