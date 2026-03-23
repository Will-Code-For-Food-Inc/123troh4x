# Feature 05: National Dex Completion & Starter Pokemon Distribution

## Goal
Enable players to complete the National Dex (483/493 Pokemon) in single-player, including all starter Pokemon from Gens 1-4.

## National Dex Statistics
- **Total Pokemon:** 493 (Gens 1-4)
- **Event-Only Excluded:** 11 (Mew, Lugia, Ho-Oh, Celebi, Jirachi, Deoxys, Phione, Manaphy, Darkrai, Shaymin, Arceus)
- **Completion Target:** 483 Pokemon
- **Starters to Add:** 12 lines (36 Pokemon including evolutions)

## Part A: Starter Pokemon Distribution

### Starter Species IDs
```
Sinnoh: Turtwig(387), Chimchar(390), Piplup(393)
Kanto: Bulbasaur(1), Charmander(4), Squirtle(7)
Johto: Chikorita(152), Cyndaquil(155), Totodile(158)
Hoenn: Treecko(252), Torchic(255), Mudkip(258)
```

### Distribution Strategy

#### 1. Unchosen Sinnoh Starters - Gift NPCs
**Location:** Professor Rowan's Lab (post-Elite Four)

**New Script File:** `/res/field/scripts/scripts_sandgem_lab_postgame.s`

```assembly
.macro GiveSinnohStarter
    GetNationalDexEnabled VAR_RESULT
    GoToIfEq VAR_RESULT, 0, _NoNationalDex

    GoToIfSet FLAG_GOT_SECOND_STARTER, _AlreadyGiven

    Message msg_assistant_offer_starter
    YesNoMenu VAR_RESULT
    GoToIfEq VAR_RESULT, 0, _GiveStarter
    Jump _Declined

_GiveStarter:
    GetPartyCount VAR_RESULT
    GoToIfEq VAR_RESULT, 6, _PartyFull

    # Logic to determine which starter based on player's original choice
    CheckPlayerStarter VAR_RESULT
    GoToIfEq VAR_RESULT, SPECIES_TURTWIG, _GiveChimchar
    GoToIfEq VAR_RESULT, SPECIES_CHIMCHAR, _GivePiplup
    GoToIfEq VAR_RESULT, SPECIES_PIPLUP, _GiveTurtwig

_GiveChimchar:
    GivePokemon SPECIES_CHIMCHAR, 5, ITEM_NONE, VAR_RESULT
    Jump _Complete

_GivePiplup:
    GivePokemon SPECIES_PIPLUP, 5, ITEM_NONE, VAR_RESULT
    Jump _Complete

_GiveTurtwig:
    GivePokemon SPECIES_TURTWIG, 5, ITEM_NONE, VAR_RESULT

_Complete:
    SetFlag FLAG_GOT_SECOND_STARTER
    Message msg_received_pokemon
    PlayFanfare SEQ_ME_ITEM
    WaitFanfare
.endm
```

#### 2. Other Generation Starters - Trophy Garden Rotation

**File:** `/res/field/encounters/encounters_trophy_garden.json`

**Current Daily Pool (16 Pokemon):**
```json
"daily_encounters": [
    "SPECIES_EEVEE", "SPECIES_BONSLY", "SPECIES_HAPPINY", "SPECIES_MEOWTH",
    "SPECIES_CLEFFA", "SPECIES_CLEFAIRY", "SPECIES_IGGLYBUFF", "SPECIES_PLUSLE",
    "SPECIES_JIGGLYPUFF", "SPECIES_DITTO", "SPECIES_CASTFORM", "SPECIES_MINUN",
    "SPECIES_MIME_JR", "SPECIES_MARILL", "SPECIES_CHANSEY", "SPECIES_AZURILL"
]
```

**Expanded Pool (Add 9 starters):**
```json
"daily_encounters": [
    // Original 16 Pokemon
    "SPECIES_EEVEE", "SPECIES_BONSLY", "SPECIES_HAPPINY", "SPECIES_MEOWTH",
    "SPECIES_CLEFFA", "SPECIES_CLEFAIRY", "SPECIES_IGGLYBUFF", "SPECIES_PLUSLE",
    "SPECIES_JIGGLYPUFF", "SPECIES_DITTO", "SPECIES_CASTFORM", "SPECIES_MINUN",
    "SPECIES_MIME_JR", "SPECIES_MARILL", "SPECIES_CHANSEY", "SPECIES_AZURILL",
    // Add Kanto starters
    "SPECIES_BULBASAUR", "SPECIES_CHARMANDER", "SPECIES_SQUIRTLE",
    // Add Johto starters
    "SPECIES_CHIKORITA", "SPECIES_CYNDAQUIL", "SPECIES_TOTODILE",
    // Add Hoenn starters
    "SPECIES_TREECKO", "SPECIES_TORCHIC", "SPECIES_MUDKIP"
]
```

**Levels:** 16-18 (matches current Trophy Garden levels)

#### 3. Alternative: Battle Frontier Gift NPCs

**Location:** Battle Frontier (Fight Area)

Create three NPCs, each offering one starter from their generation:
- **Kanto Expert:** Choose Bulbasaur, Charmander, or Squirtle (Level 5)
- **Johto Expert:** Choose Chikorita, Cyndaquil, or Totodile (Level 5)
- **Hoenn Expert:** Choose Treecko, Torchic, or Mudkip (Level 5)

## Part B: Post-Game Area Pokemon Distribution

### Route 224 (Victory Road Exit)
**File:** `/res/field/encounters/encounters_route_224.json`

**Current Levels:** 50-52
**Theme:** Elite trainers' Pokemon

**Add to grass encounters:**
```json
"land_encounters": [
    {"level": 52, "species": "SPECIES_ALAKAZAM"},
    {"level": 52, "species": "SPECIES_MACHAMP"},
    {"level": 51, "species": "SPECIES_GOLEM"},
    {"level": 52, "species": "SPECIES_GENGAR"},
    {"level": 50, "species": "SPECIES_DRAGONITE"},
    {"level": 50, "species": "SPECIES_TYRANITAR"},
    {"level": 50, "species": "SPECIES_SALAMENCE"},
    {"level": 50, "species": "SPECIES_METAGROSS"},
    {"level": 51, "species": "SPECIES_GARCHOMP"},
    {"level": 51, "species": "SPECIES_LUCARIO"},
    {"level": 50, "species": "SPECIES_PIDGEOT"},
    {"level": 50, "species": "SPECIES_NOCTOWL"}
]
```

### Routes 225-230 (Battle Zone)

#### Route 225 - Kanto Focus
**Levels:** 47-50
```json
"land_encounters": [
    {"level": 48, "species": "SPECIES_VENUSAUR"},
    {"level": 48, "species": "SPECIES_CHARIZARD"},
    {"level": 48, "species": "SPECIES_BLASTOISE"},
    {"level": 47, "species": "SPECIES_PIDGEOT"},
    {"level": 47, "species": "SPECIES_RATICATE"},
    {"level": 47, "species": "SPECIES_FEAROW"},
    {"level": 48, "species": "SPECIES_ARBOK"},
    {"level": 48, "species": "SPECIES_SANDSLASH"},
    {"level": 49, "species": "SPECIES_NIDOKING"},
    {"level": 49, "species": "SPECIES_NIDOQUEEN"},
    {"level": 50, "species": "SPECIES_CLEFABLE"},
    {"level": 50, "species": "SPECIES_WIGGLYTUFF"}
]
```

#### Route 227 - Johto Focus
**Levels:** 46-48
```json
"land_encounters": [
    {"level": 47, "species": "SPECIES_MEGANIUM"},
    {"level": 47, "species": "SPECIES_TYPHLOSION"},
    {"level": 47, "species": "SPECIES_FERALIGATR"},
    {"level": 46, "species": "SPECIES_FURRET"},
    {"level": 46, "species": "SPECIES_NOCTOWL"},
    {"level": 46, "species": "SPECIES_LEDIAN"},
    {"level": 47, "species": "SPECIES_ARIADOS"},
    {"level": 47, "species": "SPECIES_CROBAT"},
    {"level": 48, "species": "SPECIES_AMPHAROS"},
    {"level": 48, "species": "SPECIES_BELLOSSOM"},
    {"level": 48, "species": "SPECIES_AZUMARILL"},
    {"level": 48, "species": "SPECIES_SUDOWOODO"}
]
```

#### Route 229 - Hoenn Focus
**Levels:** 48-52
```json
"land_encounters": [
    {"level": 50, "species": "SPECIES_SCEPTILE"},
    {"level": 50, "species": "SPECIES_BLAZIKEN"},
    {"level": 50, "species": "SPECIES_SWAMPERT"},
    {"level": 48, "species": "SPECIES_SWELLOW"},
    {"level": 48, "species": "SPECIES_PELIPPER"},
    {"level": 49, "species": "SPECIES_GARDEVOIR"},
    {"level": 49, "species": "SPECIES_BRELOOM"},
    {"level": 49, "species": "SPECIES_SLAKING"},
    {"level": 50, "species": "SPECIES_EXPLOUD"},
    {"level": 50, "species": "SPECIES_AGGRON"},
    {"level": 51, "species": "SPECIES_FLYGON"},
    {"level": 52, "species": "SPECIES_ALTARIA"}
]
```

### Stark Mountain
**Theme:** Fire and rare types
**Levels:** 45-48

```json
"land_encounters": [
    {"level": 47, "species": "SPECIES_ARCANINE"},
    {"level": 47, "species": "SPECIES_NINETALES"},
    {"level": 46, "species": "SPECIES_MAGMORTAR"},
    {"level": 46, "species": "SPECIES_FLAREON"},
    {"level": 48, "species": "SPECIES_HOUNDOOM"},
    {"level": 48, "species": "SPECIES_CAMERUPT"},
    {"level": 45, "species": "SPECIES_TORKOAL"},
    {"level": 47, "species": "SPECIES_INFERNAPE"},
    {"level": 47, "species": "SPECIES_RAPIDASH"},
    {"level": 46, "species": "SPECIES_MAGCARGO"},
    {"level": 48, "species": "SPECIES_TYPHLOSION"},
    {"level": 48, "species": "SPECIES_BLAZIKEN"}
]
```

### Great Marsh National Dex Mode
**Already implemented!** The game switches to National Dex encounters (NARC index 9) when National Dex is obtained.

### Trophy Garden Daily Rotation
Expanded pool includes starters and rare Pokemon from all regions.

## Part C: Version Exclusive Resolution

### Currently Version-Exclusive Pokemon

**Diamond Only:**
- Dialga (keep legendary exclusive)
- Stunky/Skuntank
- Murkrow/Honchkrow
- Others...

**Pearl Only:**
- Palkia (keep legendary exclusive)
- Glameow/Purugly
- Misdreavus/Mismagius
- Others...

**Solution:** Add version exclusives to encounter tables in both versions, using unused slots.

## Testing Requirements

### Starter Availability
1. Complete Elite Four
2. Get National Dex
3. Visit Professor Rowan's lab - receive unchosen starters
4. Visit Trophy Garden daily - find other starters
5. Verify all 12 starter lines obtainable

### Encounter Testing
1. Visit each post-game route
2. Verify new Pokemon appear
3. Check encounter rates are balanced
4. Confirm levels are appropriate

### National Dex Completion
1. Track all 483 obtainable Pokemon
2. Verify each can be found/received
3. Check evolution methods work
4. Confirm no Pokemon are unobtainable

## Implementation Priority

1. **High Priority:**
   - Sinnoh starter gifts
   - Trophy Garden starters
   - Route 224-230 encounters

2. **Medium Priority:**
   - Stark Mountain variety
   - Version exclusive resolution

3. **Low Priority:**
   - Fine-tuning encounter rates
   - Additional gift NPCs

## Notes

- Trophy Garden system already supports rotation
- Great Marsh already has National Dex support
- Encounter tables use JSON format (easy to edit)
- Gift Pokemon use script commands
- Consider adding hints/NPCs that mention where to find starters