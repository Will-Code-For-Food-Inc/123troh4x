# Pokemon Platinum QoL Enhancement - Quick Reference Guide

## Project Structure
```
/home/alex/projects/reveng/ds-reveng/
├── projects/pokeplatinum/       # Main decomp project
│   ├── src/                     # C source files
│   ├── include/                 # Header files
│   ├── res/                     # Resources
│   │   ├── pokemon/            # Pokemon data JSONs
│   │   ├── field/
│   │   │   ├── encounters/     # Encounter table JSONs
│   │   │   └── scripts/        # Script files
│   │   └── text/               # Message files
│   └── build/                  # Build output
└── Documentation (this folder)
    ├── IMPLEMENTATION_PLAN.md
    ├── 01_SAVE_CONFIRMATION_REMOVAL.md
    ├── 02_FAST_POKEMON_CENTER_HEAL.md
    ├── 03_HM_MOVE_DELETION.md
    ├── 04_TRADE_EVOLUTION_REMOVAL.md
    ├── 05_NATIONAL_DEX_COMPLETION.md
    ├── FILES_TO_MODIFY.md
    └── QUICK_REFERENCE.md (this file)
```

## Priority Implementation Order

### Phase 1: Core QoL (Week 1)
| Priority | Feature | Complexity | Files | Status |
|----------|---------|------------|-------|--------|
| 1 | Remove save confirmation | Low | 1 | Research complete |
| 2 | Trade evolution removal | Low | 19+ JSON | Research complete |
| 3 | HM deletion | Medium | 1 | Research complete |
| 4 | Fast Pokemon Center | Medium | 2-3 | Research complete |
| 5 | IV/EV Display | None | 0 | Already done! |
| 6 | Indoor running | Low | 0-1 | May already work |

### Phase 2: National Dex (Week 2)
| Priority | Feature | Complexity | Files | Status |
|----------|---------|------------|-------|--------|
| 7 | Starter distribution | Medium | 2+ scripts | Research complete |
| 8 | Post-game encounters | Low | 15+ JSON | Research complete |
| 9 | Version exclusives | Low | 10+ JSON | Research complete |

### Phase 3: Polish (Week 3)
| Priority | Feature | Complexity | Files | Status |
|----------|---------|------------|-------|--------|
| 10 | Fast surf | Low | 1 | Research needed |
| 11 | Low HP beep | Low | 1 | Research needed |
| 12 | Bag improvements | Medium | 2-3 | Research needed |
| 13 | Berry system | Medium | 2-3 | Research needed |
| 14 | Persistent world | Medium | 3-4 | Research needed |

## Key Functions & Locations

### Save System
- **Function:** `StartMenu_SaveWait()`
- **File:** `src/start_menu.c:1406`
- **Scripts:** 2005 (new save), 2034 (overwrite)

### Pokemon Center Healing
- **Function:** `ScrCmd_HealParty()`
- **File:** `src/scrcmd.c:5358`
- **Party Heal:** `Party_HealAllMembers()`

### HM System
- **HM Check:** `Item_IsHMMove()`
- **Summary Screen:** `src/applications/pokemon_summary_screen/main.c:841`
- **Field Moves:** `src/field_move_tasks.c`

### Evolution System
- **Method Check:** `Pokemon_GetEvolutionTargetSpecies()`
- **File:** `src/pokemon.c:3526`
- **Data:** `res/pokemon/[species]/data.json`

### Encounter System
- **Tables:** `res/field/encounters/encounters_*.json`
- **Handler:** `src/overlay006/wild_encounters.c`
- **Trophy Garden:** `src/overlay006/trophy_garden_daily_encounters.c`

### Gift Pokemon
- **Script Command:** `GivePokemon`
- **Function:** `ScrCmd_GivePokemon()`
- **File:** `src/scrcmd_party.c:33`

## Important Constants

### Evolution Methods
```c
EVO_LEVEL = 4              // Level up
EVO_TRADE = 5              // Remove these
EVO_TRADE_WITH_HELD_ITEM = 6  // Convert to EVO_USE_ITEM
EVO_USE_ITEM = 7           // Use item directly
```

### HM Move IDs
```c
MOVE_CUT = 15
MOVE_FLY = 19
MOVE_SURF = 57
MOVE_STRENGTH = 70
MOVE_WATERFALL = 127
MOVE_FLASH = 148
MOVE_ROCK_SMASH = 249
MOVE_ROCK_CLIMB = 431
MOVE_DEFOG = 432
```

### Starter Species IDs
```c
// Sinnoh
SPECIES_TURTWIG = 387
SPECIES_CHIMCHAR = 390
SPECIES_PIPLUP = 393

// Kanto
SPECIES_BULBASAUR = 1
SPECIES_CHARMANDER = 4
SPECIES_SQUIRTLE = 7

// Johto
SPECIES_CHIKORITA = 152
SPECIES_CYNDAQUIL = 155
SPECIES_TOTODILE = 158

// Hoenn
SPECIES_TREECKO = 252
SPECIES_TORCHIC = 255
SPECIES_MUDKIP = 258
```

## National Dex Statistics
- **Total Pokemon:** 493 (Gen 1-4)
- **Excluded (Mythical):** 11
- **Completion Goal:** 483
- **Currently Unavailable:** ~100+
- **Trade Evolutions:** 19 lines

## Build Commands
```bash
cd /home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/
meson setup build
cd build
ninja
```

## Testing Checklist

### Core Features
- [ ] Save without double confirmation
- [ ] Pokemon Center instant heal after first
- [ ] Delete HM moves (with softlock prevention)
- [ ] Trade evolutions work via level/item
- [ ] Indoor running enabled
- [ ] IV/EV display cycles with L/R/SELECT

### National Dex
- [ ] All 12 starter lines obtainable
- [ ] Post-game routes have new Pokemon
- [ ] Trophy Garden has starters in rotation
- [ ] Version exclusives available
- [ ] 483 Pokemon obtainable total

### Polish
- [ ] Fast surf with B button
- [ ] Low HP beep limited to 3
- [ ] Bag sorting works
- [ ] Berries don't decay
- [ ] Rocks/trees stay cleared

## Common Pitfalls

1. **Save Corruption** - Always test save/load after changes
2. **Softlocks** - Test HM deletion in various locations
3. **Evolution Loops** - Ensure JSON syntax is correct
4. **Encounter Rates** - Balance new Pokemon appropriately
5. **Script Syntax** - Assembly scripts are sensitive to formatting

## Useful Debug Locations

### Testing Areas
- **Pokemon Center:** Any city
- **Water:** Route 219 (early Surf area)
- **Cliffs:** Route 217 (Rock Climb)
- **Post-Game:** Routes 224-230
- **Trophy Garden:** Route 212

### Key NPCs
- **Move Deleter:** Canalave City
- **Professor Rowan:** Sandgem Town
- **Trophy Garden Owner:** Route 212

## Git Commands for Tracking Changes
```bash
git init
git add -A
git commit -m "Initial state before QoL modifications"
git branch qol-enhancements
git checkout qol-enhancements
# Make changes
git add [modified files]
git commit -m "Feature: Remove double save confirmation"
```

## Notes

- JSON files are easy to revert if issues arise
- Keep backups of all original files
- Test each feature in isolation first
- Document any unexpected behavior
- Consider creating save states at key points

## Success Metrics

1. All critical features implemented
2. No save corruption or crashes
3. 483 Pokemon obtainable
4. Performance remains stable
5. Features feel natural to players

---

This quick reference provides all essential information for implementing the Pokemon Platinum QoL enhancements at a glance.