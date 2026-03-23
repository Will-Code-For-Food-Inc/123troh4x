# Complete File Modification List

This document lists all files that need to be modified for the Pokemon Platinum QoL enhancement project, organized by feature.

## Feature 01: Remove Double Save Confirmation

### C Source Files
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/src/start_menu.c`
  - Lines 1406-1423: Modify `StartMenu_SaveWait()` function

### Message Files (Optional)
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/text/unk_0010.json`
  - Message 00000213_00016: No longer displayed

---

## Feature 02: Fast Pokemon Center Healing

### C Source Files
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/src/scrcmd.c`
  - Line 5358: Modify `ScrCmd_HealParty()` function

### Header Files
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/include/vars_flags.h`
  - Add new flag: `FLAG_POKEMON_CENTER_HEALED_BEFORE`

### Script Files (If using script-based approach)
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/scripts/scripts_pokemon_center_*.s`
  - All Pokemon Center scripts need modification

---

## Feature 03: HM Move Deletion

### C Source Files
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/src/applications/pokemon_summary_screen/main.c`
  - Lines 841-847: Modify HM deletion check
  - Add new function: `CanSafelyDeleteHMMove()`
  - Add new function: `PartyHasOtherPokemonWithMove()`

---

## Feature 04: Trade Evolution Removal

### Pokemon Data Files (JSON)

#### Pure Trade Evolutions (4 files)
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/kadabra/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/machoke/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/graveler/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/haunter/data.json`

#### Trade with Item Evolutions (15+ files)
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/scyther/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/onix/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/porygon/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/porygon2/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/seadra/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/poliwhirl/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/slowpoke/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/clamperl/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/rhydon/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/electabuzz/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/magmar/data.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/dusclops/data.json`

---

## Feature 05: National Dex Completion

### Encounter Table Files (JSON)

#### Post-Game Routes (7 files)
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/encounters/encounters_route_224.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/encounters/encounters_route_225.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/encounters/encounters_route_226.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/encounters/encounters_route_227.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/encounters/encounters_route_228.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/encounters/encounters_route_229.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/encounters/encounters_route_230.json`

#### Special Areas (4+ files)
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/encounters/encounters_trophy_garden.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/encounters/encounters_stark_mountain_outside.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/encounters/encounters_stark_mountain_room_1.json`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/encounters/encounters_stark_mountain_room_2.json`

### Script Files (New)
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/scripts/scripts_sandgem_lab_postgame.s` (NEW)
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/field/scripts/scripts_fight_area_starters.s` (NEW)

---

## Feature 06: IV/EV Display

**STATUS: Already Implemented!**
- No files need modification
- Verify working: L for IVs, R for Stats, SELECT for EVs

---

## Additional QoL Features

### Fast Surf Speed
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/src/player_avatar.c`
  - Add B button check for surf speed boost

### Low HP Beep Reduction
- File TBD - Need to locate beep trigger code
  - Limit beep counter to 3

### Bag Improvements
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/src/bag.c` (presumed)
  - Increase capacity constants
  - Add sorting function
  - Add auto-PC storage

### Berry System
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/src/berry.c` (presumed)
  - Increase yield values
  - Remove decay timer
  - Speed up animations

### Persistent World States
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/include/vars_flags.h`
  - Add flags for smashed rocks, cut trees
- Map object handlers (various files)
  - Check flags before respawning objects

---

## Summary Statistics

### Total Files to Modify
- **C Source Files:** 4-6
- **Header Files:** 1-2
- **Pokemon Data JSON:** 19+
- **Encounter Table JSON:** 15+
- **Script Files:** 2+ (new)
- **Message Files:** 0-1 (optional)

### Total Estimated: ~45 files

### Modification Complexity
- **Simple (JSON edits):** 35+ files
- **Moderate (C code):** 5-7 files
- **Complex (new scripts):** 2-3 files

---

## Build System Files

### May Need Review
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/meson.build`
- `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/meson_options.txt`

---

## Testing Checklist

Before modifying each file:
1. Create backup
2. Note original behavior
3. Make minimal change
4. Test specific feature
5. Verify no side effects
6. Document change

---

## Notes

- All paths are absolute as requested
- JSON files are straightforward text edits
- C files require careful modification
- New script files need proper assembly syntax
- Consider using version control for all changes