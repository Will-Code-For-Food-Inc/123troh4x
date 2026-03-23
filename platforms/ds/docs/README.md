# Pokemon Platinum Decomp - Emerald Legacy-Style QoL Enhancement Project

## Overview
This project implements comprehensive quality-of-life enhancements for Pokemon Platinum using the pokediamond/pokeplatinum decompilation, inspired by the acclaimed Pokemon Emerald Legacy ROM hack. The goal is to modernize the Gen 4 experience while preserving its core identity.

## Project Status
**Phase:** Planning & Documentation Complete ✅
**Next Step:** Implementation Phase 1

## Documentation Structure

| Document | Description |
|----------|-------------|
| `IMPLEMENTATION_PLAN.md` | High-level project overview and phases |
| `01_SAVE_CONFIRMATION_REMOVAL.md` | Detailed guide for streamlining save system |
| `02_FAST_POKEMON_CENTER_HEAL.md` | Implementation of instant healing after first use |
| `03_HM_MOVE_DELETION.md` | Allowing HM deletion with softlock prevention |
| `04_TRADE_EVOLUTION_REMOVAL.md` | Converting trade evolutions to single-player methods |
| `05_NATIONAL_DEX_COMPLETION.md` | Adding all starters and National Dex Pokemon |
| `FILES_TO_MODIFY.md` | Complete list of all files requiring changes |
| `QUICK_REFERENCE.md` | At-a-glance reference for implementation |

## Key Features

### ✅ Researched & Ready to Implement

#### Core QoL Features
1. **Single Save Confirmation** - Remove redundant "game saved" message
2. **Fast Pokemon Center** - Instant heal after first visit (Emerald Legacy style)
3. **HM Deletion Freedom** - Delete HMs from summary screen with safety checks
4. **Trade Evolution Removal** - All Pokemon obtainable in single-player
5. **Indoor Running** - May already work, needs testing
6. **IV/EV Display** - ALREADY IMPLEMENTED! (L/R/SELECT buttons)

#### National Dex Completion
7. **All Starters Available** - Gift NPCs and Trophy Garden encounters
8. **483 Pokemon Obtainable** - Full dex completion (minus 11 mythicals)
9. **Post-Game Variety** - Routes 224-230 populated with National Dex Pokemon
10. **Version Exclusives Resolved** - All Pokemon available in one game

### 🔄 Additional Features (Research Needed)
- Fast surf speed with B button
- Reduced low HP warning (3 beeps max)
- Bag improvements (sorting, auto-PC storage)
- Berry system overhaul
- Persistent world states (rocks/trees stay cleared)

## Technical Discoveries

### Major Findings
1. **IV/EV display already exists** in the codebase
2. **Trade evolutions already have level alternatives** defined in JSON
3. **No indoor running restriction** found in code
4. **Trophy Garden system** perfect for starter distribution
5. **Great Marsh** already has National Dex encounter support

### Project Statistics
- **Files to Modify:** ~45 total
  - JSON data files: 35+
  - C source files: 5-7
  - New script files: 2-3
- **Pokemon Data:** 493 species (Gen 1-4)
- **Obtainable Goal:** 483 (excludes 11 mythicals)
- **Encounter Tables:** 189 location files

## Implementation Timeline

### Week 1: Core Systems
- Days 1-2: Save confirmation + Pokemon Center
- Days 3-4: Trade evolution removal
- Days 5-6: HM deletion + testing
- Day 7: Integration testing

### Week 2: National Dex
- Days 8-10: Post-game encounters
- Days 11-12: Starter distribution
- Days 13-14: Testing & balancing

### Week 3: Polish
- Days 15-17: Additional QoL features
- Days 18-20: Bug fixes & optimization
- Day 21: Final testing & documentation

## Build Instructions

```bash
# Navigate to project
cd /home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/

# Set up build
meson setup build

# Compile
cd build
ninja

# Output: pl_plat_us.nds
```

## Design Philosophy

### Core Principles
1. **Preserve Gen 4 Identity** - This is Platinum with polish, not a new game
2. **Respect Player Time** - Remove repetitive animations and confirmations
3. **Enable Full Experience** - All content accessible in single-player
4. **Maintain Balance** - QoL improvements shouldn't trivialize gameplay
5. **Natural Integration** - Features should feel like they always belonged

### What We're NOT Changing
- No physical/special split reversal
- No type changes
- No Fairy type
- No new Pokemon or forms
- No new evolution methods beyond trade removal
- No difficulty modifications

## Testing Requirements

### Critical Tests
- Save system integrity
- Softlock prevention
- Evolution methods
- Encounter rates
- National Dex completion

### Regression Tests
- Existing save compatibility
- Performance stability
- Script execution
- Battle system unchanged

## Contributing Guidelines

### Before Making Changes
1. Read relevant documentation file
2. Create backup of original file
3. Test in isolation first
4. Document any discoveries

### Code Style
- Preserve existing formatting
- Comment significant changes
- Use existing function patterns
- Test thoroughly

## Resources

### Decomp Project
- Main Repository: `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/`
- Build System: Meson + Ninja
- Target Platform: Nintendo DS

### Reference
- Emerald Legacy for QoL inspiration
- Pokemon Platinum for base game
- Gen 4 mechanics documentation

## Current Focus

**Immediate Next Steps:**
1. Set up development environment
2. Create baseline test ROM
3. Implement save confirmation removal (simplest change)
4. Test and validate
5. Proceed with remaining Phase 1 features

## Contact & Support

For questions about implementation, refer to:
- Feature-specific documentation files
- `QUICK_REFERENCE.md` for common lookups
- `FILES_TO_MODIFY.md` for file locations

## Acknowledgments

- Pokemon Emerald Legacy team for QoL design inspiration
- pret/pokediamond team for the decompilation
- Nintendo/Game Freak for Pokemon Platinum

---

**Project Philosophy:** Make Pokemon Platinum the definitive Gen 4 experience by respecting player time while preserving the original game's charm and identity.