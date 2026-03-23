# Feature 02: Fast Pokemon Center Healing System

## Current Behavior
Every time the player heals at a Pokemon Center:
1. Full dialogue with Nurse Joy
2. Pokemon placement animation
3. Healing jingle plays
4. Pokemon retrieval animation
5. Full closing dialogue

This takes approximately 10-15 seconds every time.

## Target Behavior (Emerald Legacy Style)
- **First heal ever:** Full dialogue and animations (classic experience)
- **Subsequent heals:** Instant heal with just confirmation sound
- **Player auto-faces down** after healing completes

## Technical Implementation

### Files to Modify

#### 1. `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/include/vars_flags.h`

**Add new flag constant:**
```c
#define FLAG_POKEMON_CENTER_HEALED_BEFORE 0x500  // Use an unused flag number
```

Note: There are 2912 available flags (364 bytes). Choose an unused flag ID.

#### 2. `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/src/scrcmd.c`

**Location:** Lines 5358-5364 (ScrCmd_HealParty function)

**Current Code:**
```c
BOOL ScrCmd_HealParty(ScriptContext *ctx)
{
    SaveData *saveData = FieldSystem_SaveData(ctx->fieldSystem);
    Party *party = SaveData_Party(saveData);

    Party_HealAllMembers(party);
    return FALSE;
}
```

**Modified Code:**
```c
BOOL ScrCmd_HealParty(ScriptContext *ctx)
{
    SaveData *saveData = FieldSystem_SaveData(ctx->fieldSystem);
    Party *party = SaveData_Party(saveData);
    VarsFlags *varsFlags = SaveData_GetVarsFlags(saveData);

    // Heal the party
    Party_HealAllMembers(party);

    // Check if this is the first heal
    if (!VarsFlags_CheckFlag(varsFlags, FLAG_POKEMON_CENTER_HEALED_BEFORE)) {
        // First heal - set flag for future
        VarsFlags_SetFlag(varsFlags, FLAG_POKEMON_CENTER_HEALED_BEFORE);

        // Play full healing fanfare
        ScriptContext_PlayFanfare(ctx, SEQ_ME_ASA);  // Full healing jingle

        // Return TRUE to wait for fanfare
        return TRUE;
    } else {
        // Subsequent heals - quick sound only
        ScriptContext_PlaySound(ctx, SEQ_SE_RECOVERY);  // Quick confirmation sound

        // Return FALSE for instant completion
        return FALSE;
    }
}
```

#### 3. Pokemon Center Script Modification

**File:** `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/scripts/scripts_pokemon_center.s` (hypothetical)

**Add auto-facing after heal:**
```assembly
.macro PokemonCenterHeal
    CheckFlag FLAG_POKEMON_CENTER_HEALED_BEFORE
    GoToIfSet _QuickHeal

_FullHeal:
    Message msg_full_welcome
    WaitABPress
    FadeScreen
    HealParty
    FadeScreen
    Message msg_full_restored
    WaitABPress
    Jump _EndHeal

_QuickHeal:
    HealParty

_EndHeal:
    ApplyMovement PLAYER, mov_face_down
    WaitMovement
    ReleaseAll
    End

mov_face_down:
    FaceDown
    EndMovement
.endm
```

### Core Functions

#### Party Healing Function
**Location:** `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/src/item_use_pokemon.c` (lines 603-628)

```c
void Party_HealAllMembers(Party *party)
{
    int i, j;
    Pokemon *mon;

    for (i = 0; i < Party_GetMemberCount(party); i++) {
        mon = Party_GetMemberPointer(party, i);

        if (Pokemon_GetValue(mon, MON_DATA_CURRENT_HP, NULL) == 0) {
            continue;
        }

        // Restore HP to max
        u16 maxHP = Pokemon_GetValue(mon, MON_DATA_MAX_HP, NULL);
        Pokemon_SetValue(mon, MON_DATA_CURRENT_HP, &maxHP);

        // Clear status conditions
        u32 status = 0;
        Pokemon_SetValue(mon, MON_DATA_STATUS_COND, &status);

        // Restore all move PP
        for (j = 0; j < 4; j++) {
            if (Pokemon_GetMoveAtIndex(mon, j) != 0) {
                u8 maxPP = Pokemon_GetValue(mon, MON_DATA_MOVE1_MAXPP + j, NULL);
                Pokemon_SetValue(mon, MON_DATA_MOVE1_PP + j, &maxPP);
            }
        }
    }
}
```

### Sound Constants

**File:** `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/include/scrcmd_sound.h`

```c
#define SEQ_ME_ASA          0x123  // Full healing fanfare (find actual value)
#define SEQ_SE_RECOVERY     0x124  // Quick recovery sound (find actual value)
```

### Flag System

**VarsFlags Structure:**
- Total flags: 2912 (364 bytes)
- Stored in save data
- Persists across game sessions

**Flag Functions:**
```c
BOOL VarsFlags_CheckFlag(VarsFlags *varsFlags, u16 flagID);
void VarsFlags_SetFlag(VarsFlags *varsFlags, u16 flagID);
void VarsFlags_ClearFlag(VarsFlags *varsFlags, u16 flagID);
```

## Testing Requirements

1. **First Heal Test**
   - New save file
   - Heal at Pokemon Center for first time
   - Verify full dialogue and animations play
   - Verify flag gets set

2. **Subsequent Heal Test**
   - Heal again at any Pokemon Center
   - Verify instant heal with sound only
   - Verify no dialogue or animations

3. **Different Centers Test**
   - Test multiple Pokemon Centers
   - Verify fast heal works at all locations

4. **Save Persistence Test**
   - Save after first heal
   - Reload game
   - Verify fast heal still works

5. **Player Direction Test**
   - Verify player faces down after healing
   - Test from different approach angles

## Implementation Options

### Option A: Script-Based (Recommended)
- Modify Pokemon Center NPC scripts
- Check flag in script
- Branch to different dialogue based on flag

### Option B: Code-Based
- Modify `ScrCmd_HealParty` as shown above
- Handle everything in C code
- Simpler but less flexible

### Option C: Hybrid
- Use flag check in script for dialogue
- Use code modification for sound effects
- Most flexible approach

## Risk Assessment

**Low-Medium Risk** - Modifying script commands requires careful testing to ensure no side effects.

## User Impact

**Very High Impact** - Saves 10+ seconds per heal, dramatically improving game flow for players who heal frequently.

## Emerald Legacy Reference

In Emerald Legacy:
- First heal shows full experience
- Subsequent heals are nearly instant
- With 4 trainer stars, dialogue is reduced even more
- This creates a sense of progression and respects player time

## Notes

- The healing function restores HP, status, and PP
- Script commands return FALSE for instant, TRUE for wait
- Flag persists in save data
- Consider adding similar system for PC healing