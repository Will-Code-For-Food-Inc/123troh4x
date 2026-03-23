# Lessons Learned - Pokemon Platinum ROM Hacking

## Character Encoding Issues

### Problem
When creating custom text strings in the Pokemon summary screen, raw ASCII characters like `'P'`, `'e'`, `'r'` were displaying as Japanese characters.

### Root Cause
Pokemon Platinum uses a custom character encoding system defined in `constants/charcode.h`, not standard ASCII.

### Solution
- Use the proper character constants: `CHAR_P`, `CHAR_e`, `CHAR_r`, etc.
- Terminate strings with `CHAR_EOS` (not `CHAR_NONE` or `0`)
- Include `#include "constants/charcode.h"` in source files

### Example
```c
// WRONG - displays Japanese
const charcode_t label[] = {'P', 'e', 'r', 'f', 'e', 'c', 't', 0};

// CORRECT - displays English
const charcode_t label[] = {CHAR_P, CHAR_e, CHAR_r, CHAR_f, CHAR_e, CHAR_c, CHAR_t, CHAR_EOS};
```

## Window Management

### Clearing Windows
- Use `Window_FillTilemap(window, 0)` to clear window contents
- Always call `Window_ScheduleCopyToVRAM(window)` after modifying to update display
- Clear windows BEFORE drawing new text to avoid leftover characters

### Pattern
```c
// Clear the window
Window_FillTilemap(&window, 0);

// Draw new content
Text_AddPrinterWithParamsAndColor(&window, ...);

// Schedule VRAM update
Window_ScheduleCopyToVRAM(&window);
```

## String Buffer Management

### Key Points
- `Strbuf` objects can be reused across multiple operations
- `Strbuf_CopyChars()` copies character arrays into string buffers
- The game uses a custom `charcode_t` type (u16) for characters, not `char`

## File Organization

### Important Files for Pokemon Summary Screen
- `src/applications/pokemon_summary_screen/main.c` - Button input handling, state management
- `src/applications/pokemon_summary_screen/window.c` - Display/rendering logic
- `include/applications/pokemon_summary_screen/main.h` - Data structures, enums
- `build/res/text/bank/pokemon_summary_screen.h` - Text constant definitions (auto-generated)
- `include/constants/charcode.h` - Character encoding constants

## Button Remapping Pattern

### Location
Button input is typically handled in `main.c` files with `JOY_NEW()` checks.

### Example
```c
if (JOY_NEW(PAD_BUTTON_L)) {
    // L button pressed
}
else if (JOY_NEW(PAD_BUTTON_R)) {
    // R button pressed
}
else if (JOY_NEW(PAD_BUTTON_SELECT)) {
    // SELECT button pressed
}
```

## Building

### Docker Build Command
```bash
docker run --rm --userns=keep-id:uid=1001,gid=1001 \
  -v /path/to/pokeplatinum:/dshax/projects/pokeplatinum \
  -w /dshax/projects/pokeplatinum \
  dshax:latest make
```

### Expected Behavior
- Checksum tests will FAIL (this is normal when you modify code)
- Look for successful compilation messages:
  - `Compiling C object`
  - `Linking target main.nef`
  - `Generating pokeplatinum.us.nds`

## Backup Strategy

Before making changes to critical files:
```bash
cp file.c file.c.bak
```

This allows easy comparison and reversion if needed.

## Common Pitfalls

1. **Forgetting VRAM updates** - Windows won't display without `Window_ScheduleCopyToVRAM()`
2. **Using wrong character encoding** - Always use `CHAR_*` constants
3. **Not clearing windows** - Old text will remain visible
4. **Wrong string terminator** - Must use `CHAR_EOS`, not `0` or `CHAR_NONE`
5. **Modifying auto-generated files** - Files in `build/` are regenerated, modify source files instead
