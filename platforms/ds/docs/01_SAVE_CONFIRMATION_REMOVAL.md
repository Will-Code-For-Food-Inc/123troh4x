# Feature 01: Remove Double Save Confirmation

## Current Behavior
When saving the game in Pokemon Platinum:
1. Player selects SAVE from menu
2. Game asks "Would you like to save the game?"
3. Player confirms YES
4. Game shows "Saving... Don't turn off the power."
5. Game saves
6. Game shows "Player saved the game!" (redundant second confirmation)
7. Player must press A to dismiss

## Target Behavior
1. Player selects SAVE from menu
2. Game asks "Would you like to save the game?"
3. Player confirms YES
4. Game shows "Saving... Don't turn off the power."
5. Game saves
6. Immediately return to game (no second confirmation)

## Technical Implementation

### Files to Modify

#### 1. `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/src/start_menu.c`

**Location:** Lines 1406-1423 (StartMenu_SaveWait function)

**Current Code:**
```c
static void StartMenu_SaveWait(FieldTask *taskMan)
{
    StartMenu *menu = FieldTask_GetEnv(taskMan);
    SaveMenu *saveMenu = menu->taskData;

    if (FieldSystem_IsScriptRunning(menu->fieldSystem) == FALSE) {
        if (saveMenu->unk_04 == 0) {
            menu->state = START_MENU_STATE_INIT;  // User said NO
        } else {
            menu->state = START_MENU_STATE_15;    // User said YES - show success message
        }

        Heap_FreeToHeap(menu->taskData);
        menu->taskData = NULL;
    }
}
```

**Modified Code:**
```c
static void StartMenu_SaveWait(FieldTask *taskMan)
{
    StartMenu *menu = FieldTask_GetEnv(taskMan);
    SaveMenu *saveMenu = menu->taskData;

    if (FieldSystem_IsScriptRunning(menu->fieldSystem) == FALSE) {
        // Always return to init state after save completes
        menu->state = START_MENU_STATE_INIT;

        Heap_FreeToHeap(menu->taskData);
        menu->taskData = NULL;
    }
}
```

#### 2. Message File (Optional)

**File:** `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/text/unk_0010.json`

**Message ID:** 00000213_00016 ("{STRVAR_1 3, 0, 0} saved the game.")

This message will no longer be displayed, but we don't need to remove it from the file.

### State Machine Flow

**Original Flow:**
```
START_MENU_STATE_SAVE → Script 2005/2034 → START_MENU_STATE_SAVE_WAIT →
  ├─ (if saved) → START_MENU_STATE_15 (success message) → START_MENU_STATE_INIT
  └─ (if cancelled) → START_MENU_STATE_INIT
```

**New Flow:**
```
START_MENU_STATE_SAVE → Script 2005/2034 → START_MENU_STATE_SAVE_WAIT → START_MENU_STATE_INIT
```

### Script Information

- **Script 2005:** New save (no existing save file)
- **Script 2034:** Overwrite existing save

These scripts handle the actual save operation and first confirmation dialog. We're only removing the post-save success message.

## Testing Requirements

1. **New Save Test**
   - Start new game
   - Save for first time
   - Verify no double confirmation
   - Verify save completes correctly

2. **Overwrite Save Test**
   - Load existing save
   - Save again (overwrite)
   - Verify overwrite warning still appears
   - Verify no success message after save

3. **Cancel Save Test**
   - Start save process
   - Select NO when prompted
   - Verify returns to menu correctly

4. **Save Integrity Test**
   - Save game with modification
   - Power off/on
   - Verify save loads correctly
   - Verify no corruption

## Risk Assessment

**Low Risk** - This is a simple UI flow change that doesn't affect the actual save mechanism.

## User Impact

**High Impact** - Saves 1-2 seconds per save, reducing frustration for frequent savers.

## Notes

- The `saveMenu->unk_04` flag tracks whether the user confirmed the save
- State 15 (`START_MENU_STATE_15`) is specifically for the success message
- The actual save operation happens in the script, not in this C code
- This change preserves the important "Are you sure?" confirmation