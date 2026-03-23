# Feature 04: Remove Trade Evolution Requirements

## Overview
Convert all trade evolutions to alternative methods that work in single-player:
- **Pure trade evolutions** → Level-up evolutions
- **Trade with held item** → Use item like evolution stone

## Important Discovery
**Many trade evolutions ALREADY have level-up alternatives defined in the data files!**
- Kadabra, Machoke, Graveler, Haunter already evolve at level 40
- Trade-with-item Pokemon have `EVO_LEVEL_WITH_HELD_ITEM` alternatives

## Implementation Strategy

### Pure Trade Evolutions

These Pokemon already have both trade AND level evolution methods defined:

| Pokemon | Current Methods | Action Needed |
|---------|----------------|---------------|
| Kadabra → Alakazam | EVO_TRADE + EVO_LEVEL (40) | Remove EVO_TRADE entry |
| Machoke → Machamp | EVO_TRADE + EVO_LEVEL (40) | Remove EVO_TRADE entry |
| Graveler → Golem | EVO_TRADE + EVO_LEVEL (40) | Remove EVO_TRADE entry |
| Haunter → Gengar | EVO_TRADE + EVO_LEVEL (40) | Remove EVO_TRADE entry |

**Optional Level Adjustments** (per Emerald Legacy):
- Haunter → Gengar: Change to level 42
- Machoke → Machamp: Change to level 38
- Graveler → Golem: Keep at level 38
- Kadabra → Alakazam: Change to level 42

### Trade with Held Item Evolutions

Convert these to use items directly (like evolution stones):

| Pokemon | Current Method | New Method |
|---------|---------------|------------|
| Scyther + Metal Coat → Scizor | EVO_TRADE_WITH_HELD_ITEM | EVO_USE_ITEM |
| Onix + Metal Coat → Steelix | EVO_TRADE_WITH_HELD_ITEM | EVO_USE_ITEM |
| Porygon + Upgrade → Porygon2 | EVO_TRADE_WITH_HELD_ITEM | EVO_USE_ITEM |
| Porygon2 + Dubious Disc → Porygon-Z | EVO_TRADE_WITH_HELD_ITEM | EVO_USE_ITEM |
| Seadra + Dragon Scale → Kingdra | EVO_TRADE_WITH_HELD_ITEM | EVO_USE_ITEM |
| Poliwhirl + King's Rock → Politoed | EVO_TRADE_WITH_HELD_ITEM | EVO_USE_ITEM |
| Slowpoke + King's Rock → Slowking | EVO_TRADE_WITH_HELD_ITEM | EVO_USE_ITEM |
| Clamperl + DeepSeaTooth → Huntail | EVO_TRADE_WITH_HELD_ITEM | EVO_USE_ITEM |
| Clamperl + DeepSeaScale → Gorebyss | EVO_TRADE_WITH_HELD_ITEM | EVO_USE_ITEM |
| Rhydon + Protector → Rhyperior | EVO_TRADE_WITH_HELD_ITEM | EVO_USE_ITEM |
| Electabuzz + Electirizer → Electivire | EVO_TRADE_WITH_HELD_ITEM | EVO_USE_ITEM |
| Magmar + Magmarizer → Magmortar | EVO_TRADE_WITH_HELD_ITEM | EVO_USE_ITEM |
| Dusclops + Reaper Cloth → Dusknoir | EVO_TRADE_WITH_HELD_ITEM | EVO_USE_ITEM |
| Gligar + Razor Fang → Gliscor | EVO_LEVEL_WITH_HELD_ITEM_NIGHT | Keep (already works!) |
| Sneasel + Razor Claw → Weavile | EVO_LEVEL_WITH_HELD_ITEM_NIGHT | Keep (already works!) |

## File Modifications

### Pure Trade Evolution Files

#### 1. Kadabra
**File:** `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/kadabra/data.json`

**Current:**
```json
"evolutions": [
    [ "EVO_TRADE", "SPECIES_ALAKAZAM" ],
    [ "EVO_LEVEL", 40, "SPECIES_ALAKAZAM" ]
]
```

**Modified:**
```json
"evolutions": [
    [ "EVO_LEVEL", 42, "SPECIES_ALAKAZAM" ]
]
```

#### 2. Machoke
**File:** `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/machoke/data.json`

**Current:**
```json
"evolutions": [
    [ "EVO_TRADE", "SPECIES_MACHAMP" ],
    [ "EVO_LEVEL", 40, "SPECIES_MACHAMP" ]
]
```

**Modified:**
```json
"evolutions": [
    [ "EVO_LEVEL", 38, "SPECIES_MACHAMP" ]
]
```

#### 3. Graveler
**File:** `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/graveler/data.json`

**Modified:**
```json
"evolutions": [
    [ "EVO_LEVEL", 38, "SPECIES_GOLEM" ]
]
```

#### 4. Haunter
**File:** `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/haunter/data.json`

**Modified:**
```json
"evolutions": [
    [ "EVO_LEVEL", 42, "SPECIES_GENGAR" ]
]
```

### Trade with Item Evolution Files

#### Example: Scyther
**File:** `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/res/pokemon/scyther/data.json`

**Current:**
```json
"evolutions": [
    [ "EVO_TRADE_WITH_HELD_ITEM", "ITEM_METAL_COAT", "SPECIES_SCIZOR" ],
    [ "EVO_LEVEL_WITH_HELD_ITEM", "ITEM_METAL_COAT", "SPECIES_SCIZOR" ]
]
```

**Modified:**
```json
"evolutions": [
    [ "EVO_USE_ITEM", "ITEM_METAL_COAT", "SPECIES_SCIZOR" ]
]
```

### Evolution Method Constants

**File:** `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/generated/evolution_methods.txt`

```
EVO_NONE                        = 0
EVO_LEVEL_HAPPINESS            = 1
EVO_LEVEL_HAPPINESS_DAY        = 2
EVO_LEVEL_HAPPINESS_NIGHT      = 3
EVO_LEVEL                      = 4
EVO_TRADE                      = 5    // Remove these entries
EVO_TRADE_WITH_HELD_ITEM       = 6    // Convert to EVO_USE_ITEM
EVO_USE_ITEM                   = 7    // Use this instead
```

### Evolution Processing Code

**File:** `/home/alex/projects/reveng/ds-reveng/projects/pokeplatinum/src/pokemon.c`

**Function:** `Pokemon_GetEvolutionTargetSpecies()` (line 3526)

The evolution checking already supports all these methods:
- `EVO_CLASS_BY_LEVEL` - Checks level-up conditions
- `EVO_CLASS_BY_TRADE` - Checks trade conditions (will be unused)
- `EVO_CLASS_BY_ITEM` - Checks item use (already works!)

## Testing Requirements

### Level Evolution Tests
1. Level up Kadabra to 42 → Should evolve to Alakazam
2. Level up Machoke to 38 → Should evolve to Machamp
3. Level up Graveler to 38 → Should evolve to Golem
4. Level up Haunter to 42 → Should evolve to Gengar

### Item Evolution Tests
1. Use Metal Coat on Scyther → Should evolve to Scizor
2. Use Metal Coat on Onix → Should evolve to Steelix
3. Use Upgrade on Porygon → Should evolve to Porygon2
4. Use Dragon Scale on Seadra → Should evolve to Kingdra
5. Test all other item evolutions

### Trade Still Works Test
1. Verify trading still functions normally
2. Pokemon should NOT evolve via trade anymore

## Complete List of Affected Pokemon

### Pure Trade (4 lines, 12 Pokemon total)
1. Kadabra → Alakazam → (no further)
2. Machoke → Machamp → (no further)
3. Graveler → Golem → (no further)
4. Haunter → Gengar → (no further)

### Trade with Item (15+ lines)
1. Scyther + Metal Coat → Scizor
2. Onix + Metal Coat → Steelix
3. Porygon + Upgrade → Porygon2 → Porygon-Z + Dubious Disc
4. Seadra + Dragon Scale → Kingdra
5. Poliwhirl + King's Rock → Politoed
6. Slowpoke + King's Rock → Slowking
7. Clamperl + DeepSeaTooth → Huntail
8. Clamperl + DeepSeaScale → Gorebyss
9. Rhydon + Protector → Rhyperior
10. Electabuzz + Electirizer → Electivire
11. Magmar + Magmarizer → Magmortar
12. Dusclops + Reaper Cloth → Dusknoir
13. Gligar + Razor Fang → Gliscor (already level+item at night)
14. Sneasel + Razor Claw → Weavile (already level+item at night)

## Impact Analysis

**Positive:**
- All Pokemon obtainable in single-player
- No need for trading partners
- Items become more valuable
- Natural progression feels better

**Considerations:**
- Trading loses some significance
- May affect game balance slightly
- Some players prefer original mechanics

## Notes

- JSON files are easy to edit
- No code changes needed for basic functionality
- Evolution animations and mechanics remain unchanged
- Items are consumed when used for evolution (existing behavior)
- Consider adding NPCs that hint at new evolution methods