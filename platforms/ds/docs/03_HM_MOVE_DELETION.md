# Feature 03: HM Move Deletion Without Move Deleter

## Current Behavior
- HM moves cannot be deleted from the Pokemon summary screen
- Players must visit the Move Deleter NPC in Canalave City
- Error message: "HM moves can't be forgotten!"

## Target Behavior
- HM moves CAN be deleted from summary screen
- Softlock prevention: Cannot delete if it would strand the player
- Move Deleter remains as convenience option

## Technical Implementation

### Files to Modify

#### 1. `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/src/applications/pokemon_summary_screen/main.c`

**Location:** Lines 841-847

**Current Code:**
```c
if (summaryScreen->cursor != LEARNED_MOVES_MAX) {
    if (Item_IsHMMove(summaryScreen->monData.moves[summaryScreen->cursor]) == TRUE &&
        summaryScreen->data->move != MOVE_NONE) {
        Sprite_SetDrawFlag2(summaryScreen->sprites[SUMMARY_SPRITE_MOVE_CATEGORY_ICON], FALSE);
        DrawEmptyHearts(summaryScreen);
        PokemonSummaryScreen_PrintHMMovesCantBeForgotten(summaryScreen);
        return SUMMARY_STATE_WAIT_HM_MSG_INPUT;
    }
}
```

**Modified Code:**
```c
if (summaryScreen->cursor != LEARNED_MOVES_MAX) {
    if (Item_IsHMMove(summaryScreen->monData.moves[summaryScreen->cursor]) == TRUE &&
        summaryScreen->data->move != MOVE_NONE) {

        // Check if deletion would cause softlock
        if (!CanSafelyDeleteHMMove(summaryScreen->fieldSystem,
                                    summaryScreen->monData.moves[summaryScreen->cursor],
                                    summaryScreen->pokemon)) {
            Sprite_SetDrawFlag2(summaryScreen->sprites[SUMMARY_SPRITE_MOVE_CATEGORY_ICON], FALSE);
            DrawEmptyHearts(summaryScreen);
            PokemonSummaryScreen_PrintHMMovesCantBeForgotten(summaryScreen);
            return SUMMARY_STATE_WAIT_HM_MSG_INPUT;
        }
        // Otherwise, allow deletion to proceed normally
    }
}
```

#### 2. Add Softlock Prevention Function

**New Function to Add:**
```c
static BOOL CanSafelyDeleteHMMove(FieldSystem *fieldSystem, u16 move, Pokemon *pokemon)
{
    // Get current player position and tile behavior
    int playerX = Player_GetXPos(fieldSystem->player);
    int playerY = Player_GetYPos(fieldSystem->player);
    int currentTile = GetTileBehavior(fieldSystem, playerX, playerY);

    // Check if currently on water and trying to delete Surf
    if (TileBehavior_IsWater(currentTile) || TileBehavior_IsSurfable(currentTile)) {
        if (move == MOVE_SURF || move == MOVE_WATERFALL) {
            // Check if any other party member has Surf/Waterfall
            if (!PartyHasOtherPokemonWithMove(fieldSystem->party, pokemon, move)) {
                return FALSE;  // Would strand player on water
            }
        }
    }

    // Check if on climbable surface and trying to delete Rock Climb
    if (TileBehavior_IsClimbable(currentTile)) {
        if (move == MOVE_ROCK_CLIMB) {
            if (!PartyHasOtherPokemonWithMove(fieldSystem->party, pokemon, move)) {
                return FALSE;  // Would strand player on cliff
            }
        }
    }

    // Check for dungeon-specific requirements
    if (IsInDungeon(fieldSystem)) {
        // Certain dungeons may require specific HMs to exit
        if (move == MOVE_ROCK_SMASH || move == MOVE_STRENGTH) {
            if (!PartyHasOtherPokemonWithMove(fieldSystem->party, pokemon, move)) {
                return FALSE;  // Might trap player in dungeon
            }
        }
    }

    return TRUE;  // Safe to delete
}

static BOOL PartyHasOtherPokemonWithMove(Party *party, Pokemon *excludePokemon, u16 move)
{
    int partyCount = Party_GetMemberCount(party);

    for (int i = 0; i < partyCount; i++) {
        Pokemon *mon = Party_GetMemberPointer(party, i);

        // Skip the Pokemon we're deleting from
        if (mon == excludePokemon) continue;

        // Check if this Pokemon has the move
        for (int j = 0; j < LEARNED_MOVES_MAX; j++) {
            if (Pokemon_GetMoveAtIndex(mon, j) == move) {
                return TRUE;
            }
        }
    }

    return FALSE;
}
```

### HM Move Detection

**File:** `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/src/item.c`

**Existing Function:**
```c
u8 Item_IsHMMove(u16 move)
{
    for (u8 i = 0; i < NUM_HMS; i++) {
        if (sTMHMMoves[NUM_TMS + i] == move) {
            return TRUE;
        }
    }
    return FALSE;
}
```

**HM Move IDs:**
```c
MOVE_CUT        = 15
MOVE_FLY        = 19
MOVE_SURF       = 57
MOVE_STRENGTH   = 70
MOVE_WATERFALL  = 127
MOVE_FLASH      = 148
MOVE_ROCK_SMASH = 249
MOVE_ROCK_CLIMB = 431
MOVE_DEFOG      = 432
```

### Tile Behavior Checks

**File:** `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/src/map_tile_behavior.c`

**Key Functions:**
```c
BOOL TileBehavior_IsWater(u16 behavior);
BOOL TileBehavior_IsSurfable(u16 behavior);
BOOL TileBehavior_IsWaterfall(u16 behavior);
BOOL TileBehavior_IsClimbable(u16 behavior);
```

### Field Move Requirements

**Critical Scenarios to Prevent:**
1. **Water Stranding**
   - Cannot delete Surf while on water
   - Cannot delete Waterfall while on/near waterfall

2. **Cliff Stranding**
   - Cannot delete Rock Climb while on climbable wall

3. **Dungeon Trapping**
   - Cannot delete required HMs in certain dungeons
   - Victory Road, Mt. Coronet, etc.

4. **Progress Blocking**
   - Warning when deleting HM needed for current area

## Testing Requirements

### Softlock Prevention Tests

1. **Surf Test**
   - Surf to middle of water
   - Try to delete Surf (should fail)
   - Have second Pokemon with Surf
   - Delete from first Pokemon (should succeed)

2. **Rock Climb Test**
   - Climb partway up cliff
   - Try to delete Rock Climb (should fail)
   - Climb to solid ground
   - Delete Rock Climb (should succeed)

3. **Dungeon Test**
   - Enter Victory Road
   - Try to delete required HMs
   - Verify appropriate blocking

4. **Party Test**
   - Multiple Pokemon with same HM
   - Delete from one (should work)
   - Try to delete from last one (context-dependent)

### General Tests

1. **Non-HM Moves**
   - Verify normal moves still delete properly

2. **Move Deleter**
   - Verify Move Deleter NPC still works

3. **Edge Cases**
   - PC vs Party
   - Different game states

## Alternative Implementations

### Option A: Warning System
Instead of blocking, show warning:
- "Warning: Deleting Surf may strand you!"
- Player can override at own risk

### Option B: Smart Detection
More sophisticated checks:
- Path-finding to nearest Pokemon Center
- Check if player can escape without HM

### Option C: Item-Based
Give player "Escape Rope Plus" that works anywhere
- Allows HM deletion without softlock risk

## Risk Assessment

**Medium Risk** - Softlock prevention is critical. Thorough testing required.

## User Impact

**High Impact** - Major convenience improvement, reduces tedious trips to Move Deleter.

## Notes

- Move Deleter bypasses all checks (summaryScreen->data->move == MOVE_NONE)
- Consider adding confirmation dialog for HM deletion
- May want to track "in dungeon" state more precisely
- Could expand to allow HM deletion in PC (safer context)