# OpenCode Agent Instructions

This file contains custom agent acronyms and instructions for the rhxd project.

## RDS (Refactor, Despaghettify, Simplify)

When you see "RDS" in a task or instruction, you are to:

1. **Refactor**: Improve code structure and organization
2. **Despaghettify**: Untangle complex, convoluted code into cleaner logic
3. **Simplify**: Reduce complexity and improve readability

### Specific Actions:
- Split large monolithic modules into smaller, more specific modules
- Merge small modules that are too specific into appropriate parent modules
- Clean up any deprecated code
- Remove dead code and unused functions
- Improve naming consistency
- Reduce code duplication
- Simplify control flow and reduce nesting

### Notes:
- We're pre-release, so it's okay to change the API on a whim right now
- Focus on maintainability and clarity over backwards compatibility
- Break things if it makes them better - we can fix consumers later

---

## DICK (Decompilation Identification and Clarification of Knowns)

**Context**: This applies when analyzing code in Ghidra reverse engineering sessions.

When you see "DICK" in a task or instruction, you are to rename **EVERY** unnamed element you come across using the tools provided by the MCP server:

### Rename Everything:
1. **Unnamed functions** → Descriptive function names based on behavior
2. **Unnamed variables** → Meaningful variable names based on usage
3. **Unnamed globals** → Clear global names based on purpose
4. **Unnamed function parameters** → Descriptive parameter names based on role

### Guidelines:
- Use clear, descriptive names that indicate purpose
- Follow consistent naming conventions
- Provide context through names (e.g., `user_count` not just `count`)
- Use the MCP server tools for renaming operations
- Document your naming decisions if non-obvious

---

## Usage Examples

```bash
# Trigger RDS refactoring
"RDS the handlers module - it's getting messy"

# Trigger DICK renaming in Ghidra
"DICK this function - analyze and rename everything"

# Combine both
"After you DICK the binary, RDS the resulting decompiled code"
```

---

## Notes

These are project-specific conventions. Standard OpenCode functionality and commands still work as normal.
