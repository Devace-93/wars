# AoE2 content reference ("numbers bible") for Wars

> Scope: Age of Empires II: The Age of Kings (AoK, 1999) + The Conquerors (AoC/TC, 2000). The 2000 expansion is
> the baseline ("AoC values"). Where the HD edition (2013+) or the Definitive Edition (DE, 2019+) changed a number in a
> way that matters for design, the newer value is given in a "DE" note. Nothing is copied verbatim; all numbers were
> re-tabulated from the sources listed at the end (primarily the Age of Empires fandom wiki infoboxes/changelogs and the
> machine-readable game data published by aoe2techtree.net).
>
> Status: complete first pass. Items marked `(verify)` could not be confirmed against a second source.

## Conventions

- Costs: `F` food, `W` wood, `G` gold, `S` stone. "175W" = 175 wood.
- Times are seconds of game time at 1.0x speed (the engine's stored values). Classic multiplayer ran at 1.5x
  ("normal" in the lobby) and DE at 1.7x; all times here are the 1.0x base.
- Armour is written `melee/pierce`. Range, LOS (line of sight) and speed are in tiles and tiles/second.
- Age column: `D` Dark, `F` Feudal, `C` Castle, `I` Imperial.
- "Reload" = seconds between attacks (rate of fire). "Train" = creation time in seconds.
- Attack bonuses are listed as `+N <armour class>`; see section 8 for the class list and the formula.
- Sources are numbered `[Sn]` and listed in section 11.

## Table of contents

1. Ages
2. Resources and economy
3. Population
4. Civilisations
5. Units
6. Buildings
7. Technologies
8. Combat formulas
9. Unit abilities and controls
10. Simplifications Wars will make
11. Sources

---

## 1. Ages

### 1.1 Advancing

Researched at the Town Center. Only one age-up can be in progress per player (it occupies the TC queue; other TCs can
still train Villagers). The building requirement counts *completed* buildings of the listed types; walls, gates, houses,
farms, outposts and towers never count.

| Age | Cost | Research time | Building prerequisite (own, completed) | Notes |
|---|---|---|---|---|
| Dark Age | - | - | - | Starting age in standard games. |
| Feudal Age | 500F | 130 s | any **2** of: Barracks, Mill, Lumber Camp, Mining Camp, Dock | Only "real" Dark Age buildings count. |
| Castle Age | 800F 200G | 160 s | any **2** of: Archery Range, Stable, Market, Blacksmith | Feudal-only buildings. |
| Imperial Age | 1000F 800G | 190 s | **1** Castle, **or** any **2** of: Monastery, University, Siege Workshop | The Castle alone suffices. |

Civ modifiers in AoC: Byzantines Imperial Age -33% (667F 533G); Persians TCs work faster (which includes the age-up
research itself: 10/15/20% in Feudal/Castle/Imperial). DE adds Malay (-66% time), Italians (-15% cost), Khmer (no
building requirement), etc.

### 1.2 Starting state (standard random map)

| Item | Value |
|---|---|
| Town Center | 1 (cannot build another until Castle Age, unless you own none) |
| Villagers | 3 (Chinese 6, Mayans 4) |
| Scout | 1 Scout Cavalry (Aztecs/Mayans: 1 Eagle Warrior - called Eagle Scout in DE) |
| Resources | see 2.1 |
| Revealed map | none ("explored" / "all visible" are lobby options) |

### 1.3 Free per-age upgrades (no research needed)

Things the engine applies automatically when an age is reached [S-FeudalAge]:

| Reached | Effect |
|---|---|
| Feudal | Scout Cavalry: +2 attack (3 -> 5), speed 1.2 -> 1.55, +2 LOS. Infantry: +2 LOS (DE; AoC had the Tracking tech instead). Palisade Walls +66% HP (DE). Non-wall buildings +1/+1 armour. Barracks +300 HP; Mill/Lumber/Mining Camp/House +200 HP (DE staggered HP; in AoC these buildings had their full HP from the start). |
| Castle | Buildings +1/+1 armour again (so most buildings go 0/7 -> 1/8 -> 2/9 -> 3/10 across the four ages). Scout line +2 LOS. Outpost +2 LOS per age. |
| Imperial | Buildings +1/+1 armour. Scout line +2 LOS. |

### 1.4 What each age unlocks (summary)

| Age | Buildings | Units | Notable techs |
|---|---|---|---|
| Dark | House, Mill, Farm, Lumber Camp, Mining Camp, Dock, Barracks, Outpost, Palisade Wall, (Palisade Gate - HD/DE) | Villager, Militia, Fishing Ship, (Transport Ship - Feudal in AoC, Dark in DE) | Loom |
| Feudal | Archery Range, Stable, Blacksmith, Market, Watch Tower, Stone Wall, Gate, Fish Trap | Man-at-Arms, Spearman, Archer, Skirmisher, Scout Cavalry, Trade Cart, Trade Cog, Galley, Fire Ship (DE: Fire Galley), Demolition Ship (DE: Demolition Raft) | Town Watch, Wheelbarrow, Horse Collar, Double-Bit Axe, Gold/Stone Mining, Blacksmith tier 1, Bloodlines (TC) |
| Castle | Town Center (more), Castle, Siege Workshop, Monastery, University, Guard Tower (tech), Fortified Wall (tech) | Long Swordsman, Pikeman, Eagle Warrior, Crossbowman, Elite Skirmisher, Cavalry Archer, Light Cavalry, Knight, Camel, Monk, unique unit, Petard, Battering Ram, Mangonel, Scorpion, War Galley, Fire Ship, Demolition Ship | Hand Cart, Town Patrol, Heavy Plow, Bow Saw, Shaft Mining, Coinage/Caravan, Blacksmith tier 2, Ballistics, Murder Holes, Masonry, Monastery techs, Castle-Age UT (HD/DE only) |
| Imperial | Wonder, Keep (tech), Bombard Tower (tech) | Two-Handed Swordsman, Champion, Halberdier, Elite Eagle, Arbalester, Heavy CA, Hand Cannoneer, Hussar, Cavalier, Paladin, Heavy Camel, elite UU, Trebuchet, Capped/Siege Ram, Onager/Siege Onager, Heavy Scorpion, Bombard Cannon, Galleon, Fast Fire Ship, Heavy Demolition Ship, (Elite) Cannon Galleon | Crop Rotation, Two-Man Saw, Banking, Guilds, Blacksmith tier 3, Chemistry, Siege Engineers, Architecture, Conscription, Sappers, Hoardings, Spies, unique tech |

---

## 2. Resources and economy

### 2.1 Starting resources per mode [S-Deathmatch][S-Regicide]

| Mode / preset | Food | Wood | Gold | Stone | Notes |
|---|---|---|---|---|---|
| Random map, Standard | 200 | 200 | 100 | 200 | The classic default. |
| Random map, Low | 100 | 100 | 50 | 100 | DE preset (verify - AoC "Low" differed and is not documented on the wiki). |
| Random map, Medium | 500 | 500 | 300 | 400 | DE preset (verify). |
| Random map, High | 1000 | 1000 | 500 | 800 | DE preset (verify). |
| Deathmatch | 20000 | 20000 | 10000 | 5000 | Start age Dark in AoC; DE defaults to Post-Imperial. Spies disabled. |
| Regicide, Standard/Low | 500 | 500 | 0 | 150 | Plus a King, usually a Castle and extra Villagers (map-dependent). Spies replaced by Treason. |
| Regicide, Medium | 750 | 750 | 0 | 225 | |
| Regicide, High | 1000 | 1000 | 0 | 300 | |

Civ offsets (AoC): Chinese -200F -50W, Mayans -50F, Persians +50F +50W, Huns -100W, Aztecs +50G (DE; added in HD).

### 2.2 Villager gather rates [S-Villager]

Base rate is resource units per second of *gathering*, excluding walking. All Villager carry capacity is 10 unless
noted. Rates are DE's; the AoC values are the same for wood/gold/stone/berries/sheep/hunt (these were never
rebalanced) and the farm figure is within rounding.

| Source | Resource | Rate /s | Rate /min | Carry | Amount per node (AoC) | Notes |
|---|---|---|---|---|---|---|
| Berry bush (forage) | Food | 0.31 | 18.6 | 10 | 125 per bush, 6 bushes per patch | Drop at Mill/TC. |
| Sheep / Turkey / Goat (herdable) | Food | 0.33 | 19.8 | 10 | 100 per animal | Herdables convert to whoever has a unit closest; decay once killed. |
| Deer (hunt) | Food | 0.41 | 24.6 | 35 | 140 | Hunter has 3-range bow; animals decay (lose food) once dead. |
| Wild Boar (hunt) | Food | 0.41 | 24.6 | 35 | 340 | Boar attacks back (7 attack, 75 HP in AoC); lured to TC. |
| Farm | Food | 0.32 | 19.2 | 10 | 175 (+75 Horse Collar, +125 Heavy Plow, +175 Crop Rotation = 550) | Farms also have an internal production cap of 0.4 food/s. |
| Shore fish (Villager) | Food | 0.43 | 25.8 | 10 | 200 | Drop at Mill/TC (Dock drop-off only since DE). |
| Fish Trap (Fishing Ship) | Food | 0.35 | 21 | 15 | 715 per trap | Only Fishing Ships can gather from traps. |
| Deep fish (Fishing Ship) | Food | 0.49 (DE now 0.42) | 29.4 | 15 | 200-350 depending on species (Perch 200, Tuna/Snapper/Salmon 225-350) | Fishing Ship shore fish 0.28/s. |
| Tree | Wood | 0.39 | 23.4 | 10 | 100 per tree | Drop at Lumber Camp/TC. |
| Gold mine | Gold | 0.38 | 22.8 | 10 | 800 per tile | Drop at Mining Camp/TC. |
| Stone mine | Stone | 0.36 | 21.6 | 10 | 350 per tile | Drop at Mining Camp/TC. |
| Relic (in Monastery) | Gold | 0.5 | 30 | - | infinite | Not a villager; see 2.6. |

Economic tech effects on these rates are in section 7 (Wheelbarrow +25% carry/+10% speed, Hand Cart +50% carry/+10%
speed, Double-Bit Axe/Bow Saw +20% each, Two-Man Saw +10%, Gold/Stone Mining +15%, Shaft Mining +15%).

Walking: Villagers move 0.8 tiles/s (Wheelbarrow and Hand Cart each add +10% movement speed). Effective rate =
carry / (carry/rate + 2 * distance / speed); keep drop-off sites adjacent to the resource.

### 2.3 Drop-off buildings

| Building | Accepts |
|---|---|
| Town Center | food, wood, gold, stone |
| Mill | food (farm, forage, herd, hunt, villager-fishing) |
| Lumber Camp | wood |
| Mining Camp | gold, stone |
| Dock | food from Fishing Ships (and from Villager fishermen only since DE) |
| Market | gold brought by Trade Carts (home Market) |
| Dock | gold brought by Trade Cogs (home Dock) |
| Monastery | relics (garrisoned, generate gold) |

A Villager who finishes building a drop-off site or a Farm deposits whatever it carries. Mill is required to build Farms
and the Market.

### 2.4 Farms and Fish Traps [S-Farm][S-FishTrap]

| | Farm | Fish Trap |
|---|---|---|
| Cost | 60W | 100W |
| Build time | 15 s (one Villager) | 53 s in AoC (15 s in AoK; 40 s in DE); built by a Fishing Ship (a Villager can help) |
| Age / prereq | Dark, needs a Mill | Feudal, needs a Dock |
| Size | 3x3 | 1x1 (water) |
| HP / armour | 480, 0/0 | 50, 0/0 (DE 250, 1/1) |
| Food | 175 base; 250 / 375 / 550 with Horse Collar / Heavy Plow / Crop Rotation | 715 |
| Workers | 1 Villager | 1 Fishing Ship |
| Reseed | A depleted farm becomes a "dead farm"; re-building costs 60W again. Farms can be queued at the Mill for automatic reseeding (AoC: queue; DE: toggle). | Not reseedable in AoC (rebuild); DE adds auto-reseed. |
| Civ | Teutons -33% cost (DE -40%); Chinese team bonus +45 food per farm (DE +10%); Franks farm techs free; Mayans resources last 20% longer (DE 15%) | Japanese fishing ships gather faster |

Farm food upgrades researched after the farm was built apply only partially in DE (proportional to remaining food
squared); in AoC they applied to new farms only.

### 2.5 Market: buying, selling, tribute [S-Market][S-Tribute]

- The Market (Feudal, 175W, requires Mill) exchanges food, wood and stone for gold in lots of 100. Prices are global
  (shared by all players in the game) and move with supply and demand.
- Exchange rate per commodity starts at **100 gold per 100 units** (DE: food 115). Every player applies a **30%
  commodity fee** on top: sell 100 for `0.7 x rate` gold, buy 100 for `1.3 x rate` gold. At the start that is sell 70 /
  buy 130.
- Each 100-unit transaction shifts that commodity's rate by **2** (AoC; DE briefly used 3 and reverted): selling lowers
  the rate, buying raises it. Rates are clamped to `[20, 9999]`, i.e. the floor is sell 14 / buy 26.
- Holding Shift trades 500 at a time (five transactions).
- Fee reductions: **Guilds** (Imperial, 300F 200G) -> 15%; Saracens civ bonus -> 5%.
- **Tribute**: resources can be sent to any player once you own a Market. A 30% tax is charged to the sender (send
  100, lose 130). **Coinage** (200F 100G, 70 s) reduces the tax to 20%, **Banking** (300F 200G, 70 s) removes it.
  In AoC Coinage is Feudal and Banking is Castle Age; HD patch 5.8 moved them to Castle/Imperial respectively.
- Allied vision: in AoC, building a Market grants shared line of sight with allies (Cartography, 100F 100G hidden
  tech). HD 5.7+ makes it free/automatic.

### 2.6 Trade carts and cogs [S-TradeCart][S-TradeCog]

Trade units walk from your Market/Dock to any *other player's* Market/Dock (ally, neutral or enemy) and return with
gold; the destination player receives nothing (except with team bonuses). Gold per trip depends on the straight-line
distance between the two buildings and the map size:

```
gold = 0.46 * d * (d / map_size + 0.3)          (AoC/DE; the 0.46 is 2 * 0.345 / 1.5)
d    = max(0.1, sqrt(max(0, |dx| - 5)^2 + max(0, |dy| - 5)^2))     # centre-to-centre, minus a 5-tile dead zone
```

- `map_size` is the map edge in tiles (Tiny 120, Small 144, Medium 168, Normal 200, Large 220, Giant 240 in AoC).
- Below ~5 tiles of distance the trip yields 0 gold. The theoretical max (opposite corners of the largest map) is
  ~513-517 gold per trip.
- Docks yield about 25% less than Markets at equal distance (compensated in later versions).
- The return trip goes to the nearest own Market in AoC (DE: the farthest own Market, to maximise yield).
- Trade Cart: 100W 50G, 70 HP, 0/0, speed 1.0 (DE 1.25), LOS 7, 51 s, Feudal, trained at the Market.
- Trade Cog: 100W 50G, 80 HP, 0/6, speed 1.32 (DE 1.65), LOS 6, 36 s, Feudal, trained at the Dock.
- Caravan (Castle Age Market tech, 200F 200G, 40 s): trade units +50% speed (DE +20%). Gold per trip scales inversely
  with the unit's speed so that gold *per second* is what the speed buff actually raises.
- Spanish team bonus: +33% trade gold (DE +25%). Portuguese/Italians (DE) discounts.

### 2.7 Relics [S-Relic]

- 5 relics spawn per standard map (HD/DE scale with player count, minimum 5). Only Monks can pick them up (Spanish
  Missionaries cannot in AoC).
- A relic garrisoned in a Monastery yields **0.5 gold/s = 30 gold/min**, forever. Aztec team bonus +33% (40/min).
- A Monk carrying a relic loses 2 LOS and cannot convert or heal until he drops it.
- Holding **all** relics on the map starts the same victory countdown as a Wonder (section 6.9); losing one relic
  cancels it.
- Huns' Atheism (UT) adds +100 years to enemy Wonder/relic countdowns.

### 2.8 Repairing [S-Villager]

- Villagers repair buildings, siege and ships from up to 2 tiles away. Full repair from 1 HP costs 50% of the original
  cost, charged progressively (Town Center repair costs only wood).
- Building repair rate: first Villager 750 HP/min, each extra +375 HP/min. Siege/ships at 25% of that.

### 2.9 Construction time with several builders [S-Villager]

`time = 3t / (n + 2)` where `t` is the one-villager build time and `n` the number of builders. So 2 builders take 75%
of the time, 3 take 60%, 10 take 25%. Treadmill Crane +20% build speed; Spanish +30%.

---

## 3. Population

| Source | Population room | Notes |
|---|---|---|
| Town Center | +5 | Chinese TCs give 10 in AoC and 15 in DE. |
| House | +5 | 2x2, 30W in AoC (25W in HD/DE). Huns cannot build Houses and start at the cap. |
| Castle | +20 | |
| Cap | 25 to 200 in steps of 25 (lobby setting; 75/100/125/150/175/200 are the usual choices). AoK/AoC hard max 200; HD 500; DE 500 (cheat 1000). | Goths +10 cap in Imperial. |

- Every unit costs 1 population (Villagers, ships, siege and Monks included). Nothing costs 0 or 2 in AoC.
- Units can be queued when "housed"; production simply stalls until room appears. Converting units ignores the cap.
- Losing a House/TC/Castle removes its room; if you drop below current population nothing dies, you just cannot
  train until you are back under the cap.
- The Town Bell (TC) garrisons every Villager within 23 tiles into the nearest TC/tower/Castle; ringing again sends
  them back to their previous task.

---

## 4. Civilisations

All 18 AoK+AoC civilisations. Values are AoC (patch 1.0c); "DE:" notes list later changes. Unique units are trained at
the Castle; the Elite upgrade is Imperial Age. AoC had **one** unique technology per civ (Imperial Age, at the Castle;
Goths had two). HD's The Forgotten added a Castle-Age unique tech to every civ; those are listed as "DE Castle UT".
Full unit stats for every unique unit are in section 5.11; here only the headline numbers are repeated.

Tech-tree gaps were cross-checked against the DE tech tree data [S-TechTree] and corrected for things that did not
exist in AoC (regional units, DE-only techs). Techs that did not exist in AoC (Gambesons, Arson, Arrowslits, Supplies,
Carvel Hull, Clinker Construction, Devotion, Fishing Lines, Treadmill Crane availability changes) are omitted from the
gap lists.

Baseline every civ has unless listed as missing: full Militia line to Champion, Spearman line to Halberdier, Archer line
to Arbalester, Skirmisher + Elite, Cavalry Archer + Heavy, Hand Cannoneer, Scout line to Hussar, Knight line to
Paladin, Camel + Heavy Camel, Monk + all 10 Monastery techs, all siege incl. Siege Onager/Siege Ram/Heavy
Scorpion/Bombard Cannon, full Dock incl. Elite Cannon Galleon, all Blacksmith/University/economy techs, Keep,
Bombard Tower, Fortified Wall. Only 0 civs actually have all of it; the lists below are what each civ lacks.

### 4.1 Britons (archers)

| | |
|---|---|
| Civ bonuses | Town Centers cost -50% wood from the Castle Age (137W 100S). Foot archers +1 range in Castle Age, +1 more in Imperial (Skirmishers excluded). Shepherds work +25% faster. |
| Team bonus | Archery Ranges work 20% faster (DE 10%). |
| Unique unit | **Longbowman** (35W 40G): 35 HP, 6 pierce, 0/0, range 5 (+civ bonus = 6 Castle / 7 Imperial), 70% accuracy. **Elite**: 40 HP, 7 attack, 0/1, range 6 (8 with bonus), 80%. Upgrade 850F 850G, 60 s. |
| Unique tech (AoC, Imp) | **Yeomen** (750W 450G, 60 s): foot archers +1 range, towers +2 attack. |
| DE Castle UT | **Warwolf**: Trebuchets gain blast damage and 100% accuracy vs units. (Yeomen moved to Castle Age in HD, Warwolf is the Imperial one.) |
| Missing | Stable: Bloodlines, Hussar, Paladin (stops at Cavalier). Archery Range: Cavalry Archer line entirely, Hand Cannoneer, Thumb Ring, Parthian Tactics. Siege: Bombard Cannon, Siege Ram. Monastery: Redemption, Atonement, Heresy. University: Bombard Tower. Dock: Elite Cannon Galleon. Economy: Crop Rotation, Stone Shaft Mining. |

### 4.2 Byzantines (defence)

| | |
|---|---|
| Civ bonuses | Buildings +10/20/30/40% HP in Dark/Feudal/Castle/Imperial. Camel, Skirmisher and Spearman lines cost -25%. Fire Ships attack 20% faster (DE 25%). Advancing to Imperial Age costs -33%. Town Watch free (patch 1.0b; DE also Town Patrol). |
| Team bonus | Monks heal 50% faster (AoK: 200%; DE 100%). |
| Unique unit | **Cataphract** (70F 75G): 110 HP, 9 melee, 2/1, +9 vs infantry (DE +13), +12 cavalry-class armour (takes less anti-cavalry bonus), speed 1.35, reload 1.8. **Elite**: 150 HP, 12 attack, +12 vs infantry (DE +18), +16 cavalry armour, reload 1.7. Upgrade 1600F 800G (DE 1200F 800G), 50 s. |
| Unique tech (AoC, Imp) | **Logistica** (800F 600G, 50 s): Cataphracts deal trample (area) damage, +6 vs infantry (the +6 was dropped later). |
| DE Castle UT | **Greek Fire**: Fire Ships +1 range. |
| Missing | Blacksmith: Blast Furnace. Stable: Bloodlines. Archery: Parthian Tactics. Siege: Siege Onager, Heavy Scorpion, Siege Engineers. University: Masonry, Architecture, Sappers (Castle), Heated Shot. Monastery: Herbal Medicine. Dock: complete (incl. Elite Cannon Galleon in AoC; DE swapped it for the Dromon). |

### 4.3 Celts (infantry, siege)

| | |
|---|---|
| Civ bonuses | Infantry move +15% faster (DE: +5/10/15/20% per age). Lumberjacks work +15% faster. Siege weapons fire 20% faster (AoC; DE 25%). Sheep/herdables cannot be stolen while a Celt unit is near them. |
| Team bonus | Siege Workshops work 20% faster. |
| Unique unit | **Woad Raider** (65F 25G; DE 70F): 65 HP (DE 70), 8 melee (DE 11), 0/1 (AoK 0/0), speed 1.2 with civ bonus (base 1.03; DE 1.17), train 10 s (AoK 16). **Elite**: 80 HP (DE 85), 13 attack (DE 15). Upgrade 1000F 800G, 45 s. |
| Unique tech (AoC, Imp) | **Furor Celtica** (750F 450G, 50 s): Siege Workshop units +50% HP (DE +40%). |
| DE Castle UT | **Stronghold**: Castles and towers fire 25% faster (DE 33%). |
| Missing | Blacksmith: Bracer, Plate Barding Armor. Stable: Bloodlines (Hussar and Paladin available). Archery: Arbalester, Cavalry Archer line, Hand Cannoneer, Thumb Ring, Parthian Tactics. Siege: Bombard Cannon. Monastery: Redemption, Atonement, Illumination, Block Printing, Theocracy. University: Architecture, Bombard Tower, Treadmill Crane (DE). Barracks: Squires. Dock: Fast Fire Ship, Elite Cannon Galleon. Economy: Crop Rotation, Two-Man Saw. |

### 4.4 Chinese (archers)

| | |
|---|---|
| Civ bonuses | Start with +3 Villagers but -200F -50W (AoK -150F). Technologies cost -10/15/20% in Feudal/Castle/Imperial (DE 5/10/15%). Town Centers support +5 extra population (10 total; DE 15) and have +5 LOS (DE +7). Demolition Ships +50% HP (DE: Fire Galley line faster). |
| Team bonus | Farms +45 food (DE +10%). |
| Unique unit | **Chu Ko Nu** (40W 35G): 45 HP, 8 pierce + 2 extra arrows of 3, 0/0, range 4, reload 3, 85% acc, train 19 s (DE 16). **Elite**: 50 HP, 8 (DE 10) + 4 extra arrows, train 13 s. Upgrade 1300F 1300G (AoK 950/950), 50 s. |
| Unique tech (AoC, Imp) | **Rocketry** (750W 750G in AoC; DE 1100F 900G, 60 s): Chu Ko Nu +2 attack, Scorpions +4 attack (DE: Scorpions +25%). |
| DE Castle UT | **Great Wall**: walls and towers +30% HP. |
| Missing | Stable: Hussar, Paladin (has Camels, DE removed them). Archery: Hand Cannoneer, Heavy Cavalry Archer, Parthian Tactics. Siege: Siege Onager, Bombard Cannon, Siege Engineers (Mangonel/Onager available in AoC; DE replaced the line with the Rocket Cart). Monastery: Redemption (AoC had it; removed in DE), Heresy. University: Hoardings. Dock: Fast Fire Ship (AoC), Cannon Galleon line (AoC had Cannon Galleon, not Elite). Economy: Crop Rotation, Guilds. |

### 4.5 Franks (cavalry)

| | |
|---|---|
| Civ bonuses | Farm upgrades (Horse Collar, Heavy Plow, Crop Rotation) free. Castles cost -25% (488S; DE 15%/25% Castle/Imperial). Knights +20% HP (AoC: Knight line only; DE all mounted units from Feudal). DE adds Foragers +15%. |
| Team bonus | Knights +2 LOS. |
| Unique unit | **Throwing Axeman** (55F 25G): ranged melee attack, 50 HP (DE 60), 7 attack, range 3 (DE 5), 0/0, speed 0.9 (DE 1.0), train 17 s (DE 13). **Elite**: 60 HP (DE 70), 8 attack, range 4 (DE 6), 1/0. Upgrade 1000F 850G (DE 850F 550G), 45 s. |
| Unique tech (AoC, Imp) | **Bearded Axe** (400F 400G, 60 s): Throwing Axemen +1 range. |
| DE Castle UT | **Chivalry**: Stables work 40% faster (Bearded Axe became the Imperial one). |
| Missing | Blacksmith: Bracer, Ring Archer Armor. Stable: Bloodlines, Hussar. Archery: Arbalester, Cavalry Archer line, Thumb Ring, Parthian Tactics. Siege: Siege Onager, Siege Ram. Monastery: Redemption, Atonement. University: Keep, Bombard Tower, Heated Shot, Treadmill Crane (DE). Dock: Shipwright, Elite Cannon Galleon. Economy: Two-Man Saw, Stone Shaft Mining, Guilds. |

### 4.6 Goths (infantry)

| | |
|---|---|
| Civ bonuses | Infantry -35% cost from the Feudal Age (AoK -10/15/25%; DE -15/20/25/30% per age). Infantry +1 attack vs standard buildings (DE +1/+2/+3 per age). Villagers +5 attack vs Wild Boar. Hunters carry +15 food. +10 population cap in the Imperial Age. DE: Loom instant. |
| Team bonus | Barracks work 20% faster. |
| Unique unit | **Huskarl** (80F 40G; DE 75F 35G): 60 HP, 10 melee, 0/6 (AoK 0/4), +6 vs archers (AoK +3), speed 1.05 (AoK 0.9), train 16 s (DE 13; AoK 26). **Elite**: 70 HP, 12 attack, 0/8, +10 vs archers. Upgrade 1200F 550G, 40 s. |
| Unique techs (AoC) | **Anarchy** (Castle Age, 450F 250G, 40 s): Huskarls can be trained at the Barracks. **Perfusion** (Imperial, 400W 600G, 40 s): Barracks work 50% faster in AoC (DE +100%). |
| Missing | Blacksmith: Plate Mail Armor, Plate Barding Armor. Stable: Paladin. Archery: Arbalester, Thumb Ring, Parthian Tactics. Siege: Siege Ram, Siege Engineers. Monastery: Redemption, Atonement, Block Printing, Heresy. University: Guard Tower, Keep, Bombard Tower, Fortified Wall, Hoardings, Treadmill Crane. Walls: no Stone Walls, no Gates (palisades only). Dock: Dry Dock, Elite Cannon Galleon (Cannon Galleon available in AoC; DE swapped for the Dromon). Economy: Gold Shaft Mining. |

### 4.7 Japanese (infantry)

| | |
|---|---|
| Civ bonuses | Fishing Ships have 2x HP, +2 pierce armour and work +5/10/15/20% faster per age. Mill, Lumber Camp, Mining Camp cost -50%. Infantry attack 33% faster from the Feudal Age (AoK: +10/15/25% per age). DE adds mounted archers +2 vs archers. |
| Team bonus | Galleys +50% LOS (DE +4). |
| Unique unit | **Samurai** (60F 30G; DE 45F 30G): 60 HP (DE 70), 8 melee (DE 10), 1/1 (AoK 1/0), +10 vs unique units (AoK +5), reload 1.9 (1.43 with civ bonus), speed 1.0, train 9 s (AoK 16). **Elite**: 80 HP, 12 attack, +12 vs unique units. Upgrade 950F 875G (DE 750F 650G), 60 s. |
| Unique tech (AoC, Imp) | **Kataparuto** (750W 300G in AoC; DE 550W 300G, 60 s): Trebuchets fire and pack/unpack faster. |
| DE Castle UT | **Yasama**: towers fire 2 extra arrows. |
| Missing | Blacksmith: Plate Barding Armor. Stable: Hussar, Paladin, (Bloodlines - added in HD). Siege: Bombard Cannon, Siege Ram, Siege Onager. Monastery: Heresy. University: Architecture, Bombard Tower, Sappers, Heated Shot, Hoardings (added later). Dock: Heavy Demolition Ship. Economy: Crop Rotation, Stone Shaft Mining, (Gold Shaft Mining - added in HD). |

### 4.8 Mongols (cavalry archers)

| | |
|---|---|
| Civ bonuses | Cavalry Archers fire 20% faster (DE 25%). Light Cavalry/Hussar +30% HP (DE +20/30% Castle/Imperial, also Steppe Lancers). Hunters work +50% faster (DE 40%). |
| Team bonus | Scout Cavalry line +2 LOS. |
| Unique unit | **Mangudai** (55W 65G): 60 HP, 6 pierce, 0/0, range 4, +3 vs siege, +1 vs spearmen (TC), reload 2.1 (1.68 with bonus; AoK 2.0), 95% acc, speed 1.45 (DE 1.4), train 26 s (AoK 21). **Elite**: 60 HP, 8 attack, 1/0, +5 vs siege. Upgrade 1100F 675G, 50 s. |
| Unique tech (AoC, Imp) | **Drill** (500W 450G, 60 s): Siege Workshop units move +50% faster. |
| DE Castle UT | **Nomads**: destroyed Houses keep their population room. |
| Missing | Blacksmith: Plate Barding Armor, Ring Archer Armor. Barracks: Halberdier. Stable: Paladin. Archery: Hand Cannoneer. Siege: Bombard Cannon. Monastery: Redemption, Sanctity, Illumination, Block Printing, Theocracy. University: Architecture, Keep, Bombard Tower, Heated Shot, Treadmill Crane. Dock: Dry Dock, Elite Cannon Galleon. Economy: Crop Rotation, Two-Man Saw, Guilds. |

### 4.9 Persians (cavalry)

| | |
|---|---|
| Civ bonuses | Start with +50F +50W. Town Centers and Docks have 2x HP and work +10/15/20% faster in Feudal/Castle/Imperial (DE also 5% in Dark). DE: Parthian Tactics in Castle Age; Caravanserai. |
| Team bonus | Knight line +2 attack vs archers. |
| Unique unit | **War Elephant** (200F 75G; DE 170F 85G): 450 HP, 15 melee, 1/2, +7 vs buildings and stone defence (DE +30), speed 0.6 (DE 0.8), LOS 4 (DE 7), train 31 s (DE 25). **Elite**: 600 HP, 20 attack, 1/3, +10 vs buildings (DE +30), trample damage 0.5 tiles at 50% (DE: both tiers). Upgrade 1600F 1200G (DE 1350F 800G), 75 s. |
| Unique tech (AoC, Imp) | **Mahouts** (300F 300G, 50 s): War Elephants +30% speed (removed in DE, speed baked in). |
| DE Castle UT | **Kamandaran**: Archer line costs wood only. DE Imperial UT: **Citadels**. |
| Missing | Barracks: complete in AoC (Halberdier available; DE later removed Two-Handed Swordsman/Champion). Archery: Arbalester (Hand Cannoneer available). Stable: complete in AoC incl. Paladin (DE replaced Paladin with Savar). Monastery: Redemption, Atonement, Sanctity, Illumination, Heresy. University: Keep, Bombard Tower, Fortified Wall, Siege Engineers, Treadmill Crane. Blacksmith: Bracer. Dock: Shipwright. Economy: complete (one of the best eco trees). |

### 4.10 Saracens (camels, navy)

| | |
|---|---|
| Civ bonuses | Market commodity fee 5% instead of 30%. Transport Ships 2x HP, +5 carry capacity (DE +20). Galleys attack 20% faster (DE 25%). Camels +10 attack vs buildings (AoC; DE replaced by +25% HP). DE: Markets cost 75W. |
| Team bonus | Foot archers +1 attack vs standard buildings (shown +1, actually +2 in AoC; DE +3). |
| Unique unit | **Mameluke** (55F 85G): camel with a ranged (3) melee attack, 65 HP (DE 80 base, 100 with bonus), 7 attack (DE 8), 0/0, +9 vs cavalry, speed 1.4, train 23 s. **Elite**: 80 HP (DE 90), 10 attack, 1/0, +12 vs cavalry. Upgrade 600F 500G, 50 s. |
| Unique tech (AoC, Imp) | **Zealotry** (750F 800G, 45 s): camels and Mamelukes +30 HP (DE +20; later replaced). |
| DE Castle UT | **Madrasah** (Monk gold refund), later **Bimaristan**; DE Imperial: **Counterweights**. |
| Missing | Blacksmith: complete. Barracks: Halberdier (verify for AoC; missing in DE). Stable: Cavalier/Paladin (AoC: Cavalier yes, Paladin no; DE removed Cavalier too). Siege: Heavy Scorpion. University: Architecture, Bombard Tower, Heated Shot, Treadmill Crane (DE). Dock: Fast Fire Ship, Shipwright. Economy: Crop Rotation, Stone Shaft Mining, Guilds. Monastery: complete. |

### 4.11 Teutons (infantry, defence)

| | |
|---|---|
| Civ bonuses | Monks have 2x healing range. Towers garrison twice as many units (10). Murder Holes free (DE also Herbal Medicine). Farms cost -33% (40W; DE -40%). Town Centers +5 LOS (AoK also +5 range, removed in TC). DE: TC garrison +10; infantry/cavalry +1/+2 melee armour. |
| Team bonus | Units resist conversion (conversion takes longer: +1 min/+2 max intervals in AoC; DE +3/+1). |
| Unique unit | **Teutonic Knight** (85F 40G; DE 30G): 70 HP (DE 90), 12 melee (DE 14), 5/2 (DE 7/2), speed 0.7 (DE 0.8), train 12 s (AoK 19). **Elite**: 100 HP (DE 110), 17 attack, 10/2 (DE 8/2 base). Upgrade 1200F 600G (DE 950F 500G), 50 s. |
| Unique tech (AoC, Imp) | **Crenellations** (600F 400S, 60 s): Castles +3 range, garrisoned infantry fire arrows. |
| DE Castle UT | **Ironclad**: siege +4 melee armour. |
| Missing | Blacksmith: Bracer. Stable: Light Cavalry (Scout only), Hussar, Husbandry. Archery: Arbalester, Cavalry Archer line, Thumb Ring, Parthian Tactics. Siege: Siege Ram. University: Architecture, Treadmill Crane (DE). Dock: Shipwright, Dry Dock, Elite Cannon Galleon. Economy: Gold Shaft Mining. Monastery: complete. |

### 4.12 Turks (gunpowder)

| | |
|---|---|
| Civ bonuses | Gunpowder units +25% HP (AoK +50%). Chemistry free. Gunpowder techs cost -50% (Hand Cannoneer/Bombard Cannon/Cannon Galleon/Bombard Tower/Elite Janissary). Gold miners work +15% faster (DE 25%). Light Cavalry and Hussar upgrades free. DE: Scout line +1 pierce armour. |
| Team bonus | Gunpowder units created 20% faster (DE 25%). |
| Unique unit | **Janissary** (60F 55G): 35 HP (44 with bonus), 17 pierce (AoK 15 with +4 vs infantry), 1/0, range 8 (DE 7), reload 3.45, 50% acc, no minimum range, train 21 s. **Elite**: 40 HP (50), 22 attack (AoK 18), 2/0, range 8, 65% acc. Upgrade 850F 750G, 55 s. |
| Unique tech (AoC, Imp) | **Artillery** (500W 450G in AoC; DE 600F 650G, 40 s): +2 range for Bombard Towers, Bombard Cannons and Cannon Galleons. |
| DE Castle UT | **Sipahi**: mounted archers +20 HP. |
| Missing | Barracks: Pikeman/Halberdier (Spearman only). Archery: Elite Skirmisher, Arbalester. Stable: Paladin. Siege: Onager/Siege Onager, Siege Engineers. Monastery: Illumination, Block Printing, Herbal Medicine. Dock: Fast Fire Ship. Economy: Crop Rotation, Stone Shaft Mining. University: complete. |

### 4.13 Vikings (infantry, navy)

| | |
|---|---|
| Civ bonuses | Wheelbarrow and Hand Cart free (researched instantly when available). Warships cost -20% (DE -10/15/20% per age). Infantry +10/15/20% HP in Feudal/Castle/Imperial (DE +20% from Feudal). |
| Team bonus | Docks cost -25% (DE -15%). |
| Unique units | **Berserk** (65F 25G; DE 20G): 48 HP base (x civ bonus), 9 melee (DE 12), 0/1 (AoK 0/0), regenerates 20 HP/min (DE 40), speed 1.05 (AoK 0.9), train 16 s (DE 14). **Elite**: 60 HP base (DE 62), 14 attack, 2/1. Upgrade 1300F 550G (DE 1075F 475G), 45 s. **Longboat** (Dock, Castle Age, 100W 50G): 130 HP, 7 pierce x multiple arrows, 0/6, range 6, train 25 s (AoK 41). **Elite**: 160 HP, 8 attack, 0/8, range 7 (AoC; DE reworked to 130 HP/7 attack). Upgrade 750F 475G, 60 s. |
| Unique tech (AoC, Imp) | **Berserkergang** (500F 850G, 40 s): Berserks regenerate faster. |
| DE Castle UT | **Chieftains**: infantry +5 vs cavalry, +4 vs camels. DE Imperial: **Bogsveigar** (+1 attack archers/longboats). |
| Missing | Blacksmith: Plate Barding Armor. Barracks: Halberdier. Stable: Bloodlines, Husbandry, Hussar, Paladin. Archery: Hand Cannoneer, Cavalry Archer line (AoK had it), Thumb Ring, Parthian Tactics. Siege: Bombard Cannon. Monastery: Redemption, Sanctity, Illumination, Theocracy, Herbal Medicine. University: Keep, Bombard Tower. Dock: Fast Fire Ship (AoC Vikings had Fire Ship but not the Fast upgrade - verify), Cannon Galleon line, Shipwright (DE instead removed Demolition Ships). Economy: Stone Shaft Mining, Guilds. |

### 4.14 Aztecs (infantry, monks) - The Conquerors

| | |
|---|---|
| Civ bonuses | Villagers carry +5 (DE +3). All military units created 15% faster (DE 11% then 15%). Monks +5 HP per Monastery tech researched. Loom free (1.0b). Start with +50G (added in HD; DE). Start with an Eagle Scout, no Stable at all. |
| Team bonus | Relics generate +33% gold. |
| Unique unit | **Jaguar Warrior** (60F 30G): 50 HP (DE 65), 10 melee (DE 15), 1/0 (DE 1/2), +10 vs infantry (DE +5), speed 1.0, train 20 s (DE 12). **Elite**: 75 HP, 12 attack (DE 19), 2/0 (DE 2/2), +11 vs infantry (DE +6). Upgrade 1000F 500G, 45 s. |
| Unique tech (AoC, Imp) | **Garland Wars** (450F 750G, 60 s): infantry +4 attack. |
| DE Castle UT | **Atlatl**: Skirmishers +1 attack, +1 range. |
| Missing | No Stable (no cavalry at all, no Bloodlines/Husbandry). No gunpowder (Hand Cannoneer, Bombard Cannon, Bombard Tower, Cannon Galleon). Blacksmith: Ring Archer Armor (and no cavalry armour obviously). Archery: Thumb Ring, Parthian Tactics. Barracks: Halberdier. Siege: Heavy Scorpion. University: Masonry, Architecture, Keep, Hoardings. Dock: Galleon (verify), Heavy Demolition Ship, Dry Dock. Economy: Two-Man Saw, Guilds. Monastery: complete. |

### 4.15 Huns (cavalry) - The Conquerors

| | |
|---|---|
| Civ bonuses | Do not need Houses; start with the population cap maxed but -100W. Cavalry Archers cost -25% in Castle Age, -30% in Imperial (DE -10/20%). Trebuchets +30% accuracy (DE 35%). |
| Team bonus | Stables work 20% faster. |
| Unique unit | **Tarkan** (60F 60G): 90 HP (DE 100), 7 melee (DE 8), 1/2 (DE 1/3), +8 vs buildings (+12 vs stone defence, +10 vs castles in DE), speed 1.35 (DE 1.4), reload 2.1 (DE 1.9), train 14 s. **Elite**: 150 HP, 11 attack, 1/3 (DE 1/4), +10 vs buildings. Upgrade 1000F 500G, 45 s. |
| Unique tech (AoC, Imp) | **Atheism** (500F 500G in AoC; DE 500F 300W, 60 s): +100 years to Wonder and relic victory countdowns for everyone; Spies/Treason cost -50% (AoC; DE: enemy relic income -50%). |
| DE Castle UT | **Marauders**: Tarkans trainable at the Stable. |
| Missing | Blacksmith: Plate Mail Armor, Ring Archer Armor. Barracks: Champion. Archery: Arbalester, Hand Cannoneer. Siege: Onager/Siege Onager, Heavy Scorpion, Bombard Cannon, Siege Engineers. Monastery: Redemption, Block Printing, Theocracy, Herbal Medicine. University: Guard Tower, Keep, Bombard Tower, Fortified Wall, Architecture, Hoardings, Heated Shot, Treadmill Crane. Dock: Fast Fire Ship, Cannon Galleon line (verify for AoC), Shipwright. Economy: Crop Rotation, Stone Shaft Mining. |

### 4.16 Koreans (defence, navy) - The Conquerors

| | |
|---|---|
| Civ bonuses | Villagers +3 LOS (1.0b; DE made it the team bonus). Stone miners work +20% faster. Tower upgrades (Guard Tower, Keep) free; Bombard Tower free once Chemistry is known. Towers +1 range in Castle Age, +2 in Imperial (removed in DE, replaced by the Eupseong UT). DE: archers and infantry -50% wood (later), warships -20% wood, archer armour free. |
| Team bonus | Mangonel line minimum range -1 (DE: Villagers +3 LOS). |
| Unique units | **War Wagon** (120W 60G after 1.0b; DE 200W 60G with -50% wood): 150 HP, 9 pierce, 0/3 (DE 0/2 base), range 4 (5 before 1.0b), speed 1.2, reload 2.5, 100% acc, train 25 s (DE 21). **Elite**: 200 HP, 9 attack, 0/4, range 5. Upgrade 1000W 800G, 75 s. **Turtle Ship** (Dock, Castle Age, 200W 200G in AoC; DE 190W 180G -> 152/130): 200 HP, 50 melee (cannon), 6/5, range 6, speed 0.9, reload 6, train 50 s. **Elite**: 300 HP, 50 attack, 8/6 (DE reworked: 260/350 HP, 20/25 attack + rockets). Upgrade 1000F 800G (DE 650F 500G), 65 s. |
| Unique tech (AoC, Imp) | **Shinkichon** (800W 500G, 60 s): Mangonel line +1 range (+2 before 1.0b; DE +1 range for Rocket Carts/Turtle Ships). |
| DE Castle UT | **Eupseong**: towers +2 range (replaced the civ bonus). |
| Missing | Blacksmith: Blast Furnace, Plate Barding Armor. Stable: Bloodlines, Paladin. Archery: Parthian Tactics. Siege: Heavy Scorpion, Siege Ram (Mangonel line incl. Siege Onager available in AoC; DE replaced it with the Rocket Cart). Monastery: Redemption, Atonement, Illumination, Heresy. University: Sappers, Hoardings. Dock: Demolition line (verify), Heavy Demolition Ship. Economy: Crop Rotation. |

### 4.17 Mayans (archers) - The Conquerors

| | |
|---|---|
| Civ bonuses | Start with +1 Villager but -50F. Resources last 20% longer (DE 15%) - every node yields more. Archer line costs -10/20/30% in Feudal/Castle/Imperial. Start with an Eagle Scout, no Stable. |
| Team bonus | Walls and gates (palisade and stone) cost -50%. |
| Unique unit | **Plumed Archer** (46W 46G base in AoC; DE 55W 55G, reduced by the civ discount): 50 HP, 5 pierce, 0/1, range 4, speed 1.2, reload 1.9, 80% acc, +2 vs spearmen, +1 vs infantry, train 16 s. **Elite**: 65 HP, 5 attack, 0/2, range 5, 90% acc, +2 vs infantry. Upgrade 500F 1000W (DE 700F 1000W), 45 s. |
| Unique tech (AoC, Imp) | **El Dorado** (750F 450G, 70 s): Eagle Warriors +40 HP. |
| DE Castle UT | **Obsidian Arrows** (HD, archers +6 vs buildings), replaced by **Hul'che Javelineers** in DE (Skirmishers throw a second projectile). |
| Missing | No Stable, no gunpowder (as Aztecs). Blacksmith: Plate Barding etc. (no cavalry). Barracks: Champion. Stable techs: none. Archery: Parthian Tactics. Siege: Siege Engineers. Monastery: Redemption, Illumination. University: Bombard Tower. Dock: Dry Dock, Cannon Galleon line. Economy: Gold Shaft Mining. |

### 4.18 Spanish (gunpowder, monks) - The Conquerors

| | |
|---|---|
| Civ bonuses | Builders work +30% faster (DE: 20% for Wonders). Blacksmith upgrades cost no gold. Cannon Galleons benefit from Ballistics (track targets, faster projectiles). Gunpowder units fire 15% faster (DE 18%). DE: +20 gold per technology researched. |
| Team bonus | Trade units generate +33% gold (DE +25%). |
| Unique units | **Conquistador** (60F 70G; DE 75F 60G): mounted gunpowder, 55 HP, 16 pierce, 2/2 (DE 2/1), range 6, speed 1.3, reload 2.9, 65% acc, train 24 s (DE 26); not in the Cavalry Archer class in AoC (immune to anti-CA bonuses). **Elite**: 70 HP, 18 attack (DE 19), 2/2, 70% acc, +2 vs buildings. Upgrade 900F 600G, 60 s. **Missionary** (Monastery, Castle Age, 100G; DE 30F 100G): mounted Monk, 30 HP, conversion range 7, heal range 4, speed 1.1, LOS 9, train 51 s; cannot pick up relics in AoC; Cavalry class (takes anti-cavalry bonus; AoC: not affected by Bloodlines/cavalry armour). |
| Unique tech (AoC, Imp) | **Supremacy** (400F 250G, 60 s): Villagers +6 attack, +2/+2 armour, +40 HP. |
| DE Castle UT | **Inquisition**: Monks convert faster (-1 min/-1 max interval; -6 for buildings). |
| Missing | Archery: Crossbowman/Arbalester (Archer only), Cavalry Archer line (DE: replaced by Mounted Crossbowman), Parthian Tactics. Siege: Heavy Scorpion, Siege Onager, Siege Engineers. University: Heated Shot. Economy: Crop Rotation, Gold Shaft Mining. Blacksmith/Stable/Barracks/Dock/Monastery: complete. |

### 4.19 Summary matrix (AoC, units only)

| Civ | Champion | Halberdier | Arbalest | HCA | Hand Cannon | Hussar | Paladin | Heavy Camel | Siege Onager | Siege Ram | Heavy Scorp | Bombard Cannon | Elite Cannon Galleon | Keep | Bombard Tower |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Britons | Y | Y | Y | - | - | - | - | - | Y | - | Y | - | - | Y | - |
| Byzantines | Y | Y | Y | Y | Y | Y | Y | Y | - | Y | - | Y | Y | Y | Y |
| Celts | Y | Y | - | - | - | Y | Y | - | Y | Y | Y | - | - | Y | - |
| Chinese | Y | Y | Y | - | - | - | - | Y | - | Y | Y | - | - | Y | Y |
| Franks | Y | Y | - | - | Y | - | Y | - | - | - | Y | Y | - | - | - |
| Goths | Y | Y | - | Y | Y | Y | - | Y | Y | - | Y | Y | - | - | - |
| Japanese | Y | Y | Y | Y | Y | - | - | Y | - | - | Y | - | Y | Y | - |
| Mongols | Y | - | Y | Y | - | Y | - | Y | Y | Y | Y | - | - | - | - |
| Persians | Y | Y | - | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | - | - |
| Saracens | Y | - | Y | Y | Y | Y | - | Y | Y | Y | - | Y | Y | Y | - |
| Teutons | Y | Y | - | - | Y | - | Y | - | Y | - | Y | Y | - | Y | Y |
| Turks | Y | - | - | Y | Y | Y | - | Y | - | Y | Y | Y | Y | Y | Y |
| Vikings | Y | - | Y | - | - | - | - | - | Y | Y | Y | - | - | - | - |
| Aztecs | Y | - | Y | - | - | - | - | - | Y | Y | - | - | - | - | - |
| Huns | - | Y | - | Y | - | Y | Y | - | - | Y | - | - | - | - | - |
| Koreans | Y | Y | Y | Y | Y | Y | - | - | Y | - | - | Y | - | Y | Y |
| Mayans | - | Y | Y | - | - | - | - | - | Y | Y | Y | - | - | Y | - |
| Spanish | Y | Y | - | - | Y | Y | Y | Y | - | Y | - | Y | Y | Y | Y |

(Derived from the DE tree with the AoC corrections noted above; cells marked in the per-civ lists as "verify" are
best-effort.)

---

## 5. Units

Column key: Cost | HP | Atk (M = melee, P = pierce) | Armour melee/pierce | Range (min range in brackets) | LOS |
Speed | Reload | Train time | Attack bonuses | Armour classes. Values are AoC unless a "DE" note says otherwise;
where the DE game data [S-TechTree] and the wiki infobox agree the value is unchanged since 2000.

Every unit is 1 population. "Acc" = accuracy percentage for projectile units (chance to hit a stationary target;
misses still land nearby and can hit something else for 50% damage).

### 5.1 Villager

| Unit | Age | Building | Cost | HP | Atk | Armour | Range | LOS | Speed | Reload | Train | Bonuses | Classes |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Villager | D | Town Center | 50F | 25 | 3 M | 0/0 | 0 (3 as hunter) | 4 | 0.8 | 2 | 25 | +3 All Buildings, +6 Stone Defense (AoK +3) | - |

Loom: +15 HP, +1/+2 armour. Carry capacity 10 (hunters 35). Can garrison in TC/towers/Castle/rams (rams since HD).

### 5.2 Barracks: Militia line

| Unit | Age | Cost | HP | Atk | Armour | LOS | Speed | Reload | Train | Bonuses | Upgrade cost / time |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Militia | D | 60F 20G (DE 50F 20G) | 40 | 4 M | 0/1 (AoK 0/0) | 4 | 0.9 | 2 | 21 | - | - |
| Man-at-Arms | F | 60F 20G | 45 | 6 M | 0/1 | 6 | 0.9 (DE 0.96) | 2 | 21 | +2 Eagle (DE +3), +1 Standard Building (DE +2) | 100F 40G / 40 s |
| Long Swordsman | C | 60F 20G | 55 (DE 60) | 9 M | 0/1 (DE 1/1) | 6 | 0.9 (DE 0.96) | 2 | 21 | +4 Eagle (DE +8), +2 Standard Building (DE +3) | 200F 65G / 45 s (DE 150F 65G / 40 s) |
| Two-Handed Swordsman | I | 60F 20G | 60 (DE 65) | 11 M (DE 12) | 0/1 (DE 1/1) | 7 | 0.9 (DE 0.96) | 2 | 21 | +6 Eagle (DE +9), +3 Standard Building (DE +4) | 300F 100G / 75 s (DE 200F 100G / 45 s) |
| Champion | I | 60F 20G | 70 | 13 M (DE 14) | 1/1 | 7 | 0.9 (DE 0.96) | 2 | 21 | +8 Eagle (DE +9), +4 Standard Building | 750F 350G / 100 s (DE 650F 350G / 70 s) |

Classes: Infantry. Benefits from Blacksmith infantry attack/armour, Squires (+10% speed), Tracking (+2 LOS, AoC),
Conscription (+33% train speed), Arson (DE, +2 vs buildings).

### 5.3 Barracks: Spearman line

| Unit | Age | Cost | HP | Atk | Armour | LOS | Speed | Reload | Train | Bonuses (AoC) | Upgrade |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Spearman | F | 35F 25W | 45 | 3 M | 0/0 | 4 | 1.0 | 3 | 22 | +15 Cavalry, +15 War Elephant, +7 Camel (DE +12), +7 Ship (DE +9), +1 Eagle, +1 Standard Building | - |
| Pikeman | C | 35F 25W | 55 | 4 M | 0/0 | 4 | 1.0 | 3 | 22 | +22 Cavalry, +25 War Elephant, +11 Camel (DE +18), +11 Ship (DE +16), +1 Eagle, +1 Standard Building | 215F 90G / 45 s (DE 160F 90G / 35 s) |
| Halberdier | I | 35F 25W | 60 | 6 M | 0/0 | 4 | 1.0 | 3 | 22 | +32 Cavalry, +28 War Elephant, +16 Camel (DE +26), +16 Ship (DE +17), +1 Eagle, +1 Standard Building | 300F 600G / 50 s |

Classes: Infantry, Spearman. (DE also gives +7 vs Mamelukes to Pikeman/Halberdier.) The Spearman class is what
archers and skirmishers get their +2/+3 bonus against.

### 5.4 Barracks: Eagle Warrior line (Aztecs, Mayans only)

In AoC the starting scout is an "Eagle Warrior" (Dark Age stats) and the Eagle Warrior becomes trainable in the Castle
Age; HD/DE renamed the base unit Eagle Scout, made it trainable in Feudal, and inserted a Castle-Age "Eagle Warrior"
upgrade (200F 200G, 50 s).

| Unit | Age | Cost | HP | Atk | Armour | LOS | Speed | Reload | Train | Bonuses (DE values) | Upgrade |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Eagle Scout (DE) / starting Eagle | D/F | 20F 50G | 50 | 4 M | 0/2 | 5 | 1.1 | 2 | 46 (Feudal) / 35 | +8 Monk, +3 Siege | 200F 200G / 50 s to Eagle Warrior (DE) |
| Eagle Warrior | C | 20F 50G | 50 (DE 55) | 7 M | 0/3 | 6 | 1.15 | 2 | 35 | +8 Monk, +3 Siege, +2 Cavalry (DE +3), +1 Camel (DE +2), +1 Ship | - |
| Elite Eagle Warrior | I | 20F 50G | 60 | 9 M | 0/4 | 6 | 1.3 | 2 | 20 | +10 Monk, +5 Siege, +4 Cavalry, +3 Camel, +2 Ship | 800F 500G / 50 s |

Classes: Infantry, Eagle Warrior (shock infantry). Benefits from Blacksmith infantry upgrades and Squires.

### 5.5 Archery Range

| Unit | Age | Cost | HP | Atk | Armour | Range | LOS | Speed | Reload | Acc | Train | Bonuses | Upgrade |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Archer | F | 25W 45G | 30 | 4 P | 0/0 | 4 | 6 | 0.96 | 2 | 80% | 35 (AoK 27) | +3 Spearman (AoK none) | - |
| Crossbowman | C | 25W 45G | 35 | 5 P | 0/0 | 5 | 7 | 0.96 | 2 | 85% | 27 | +3 Spearman | 125F 75G / 35 s (DE 175F 100G) |
| Arbalester | I | 25W 45G | 40 | 6 P | 0/0 | 5 | 7 | 0.96 | 2 | 90% | 27 | +3 Spearman | 350F 300G / 50 s (DE 450F 350G) |
| Skirmisher | F | 25F 35W | 30 | 2 P | 0/3 | 4 (min 1) | 6 | 0.96 | 3 | 90% | 22 (DE 26) | +3 Archer, +3 Spearman (AoK none) | - |
| Elite Skirmisher | C | 25F 35W | 35 | 3 P | 0/4 | 5 (min 1) | 7 | 0.96 | 3 | 90% | 22 | +4 Archer, +3 Spearman (DE +4), +2 Cavalry Archer | 250W 160G / 50 s (DE 230W 130G) |
| Cavalry Archer | C | 40W 70G (DE 60G) | 50 | 6 P | 0/0 | 4 (3 before 1.0b) | 5 | 1.43 (DE 1.4) | 2 | 50% | 34 (DE 37) | +2 Spearman | - |
| Heavy Cavalry Archer | I | 40W 70G (DE 60G) | 60 | 7 P | 1/0 | 4 | 6 | 1.43 (DE 1.4) | 2 | 50% (DE 80%) | 34 (DE 30) | +2 Spearman (DE +4) | 900F 500G / 50 s |
| Hand Cannoneer | I (Chemistry) | 45F 50G | 35 (DE 40) | 17 P | 1/0 | 7 | 9 | 0.96 | 3.45 | 65% (AoK 55%, DE 75%) | 34 | +10 Infantry (AoK +8), +2 Ram, +1 Spearman (DE) | - |

Classes: Archer (all); Cavalry Archer + Cavalry (CA line); Gunpowder (Hand Cannoneer); Skirmisher (DE class).
Archer-line and Skirmishers benefit from Fletching/Bodkin/Bracer (+1 attack +1 range each), archer armour, Thumb
Ring (100% accuracy, +18% fire rate for archers only), Ballistics. Hand Cannoneers do not get range from the Fletching line; Chemistry (their prerequisite) gives +1 attack to all
missile units including them.
Cavalry Archers additionally get Bloodlines (+20 HP), Husbandry (+10% speed), Parthian Tactics (+1/+2 armour, +4 vs
spearmen; Heavy CA +6 in AoC).

### 5.6 Stable

| Unit | Age | Cost | HP | Atk | Armour | LOS | Speed | Reload | Train | Bonuses | Upgrade |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Scout Cavalry | F (also the starting unit) | 80F | 45 | 3 M (5 from Feudal) | 0/2 | 4 (+2/age -> 10) | 1.2 (1.55 from Feudal) | 2 | 30 | +6 Monk (TC) | - |
| Light Cavalry | C | 80F | 60 | 7 M | 0/2 | 8 | 1.5 | 2 | 30 | +10 Monk | 150F 50G / 45 s |
| Hussar | I | 80F | 75 | 7 M | 0/2 | 10 | 1.5 | 1.9 | 30 | +12 Monk | 500F 600G / 50 s |
| Knight | C | 60F 75G | 100 | 10 M | 2/2 | 4 | 1.35 | 1.8 | 30 | - | - |
| Cavalier | I | 60F 75G | 120 | 12 M | 2/2 | 4 | 1.35 | 1.8 | 30 | - | 300F 300G / 100 s (DE 80 s) |
| Paladin | I | 60F 75G | 160 | 14 M | 2/3 | 5 | 1.35 | 1.9 | 30 | - | 1300F 750G / 170 s |
| Camel | C | 55F 60G | 100 | 5 M (DE 6) | 0/0 | 4 (DE 5) | 1.45 (AoK 1.4) | 2 | 22 (AoK 29) | +10 Cavalry (DE +9), +5 Camel, +5 Ship | - |
| Heavy Camel | I | 55F 60G | 120 | 7 M | 0/0 | 5 | 1.45 | 2 | 22 | +18 Cavalry, +9 Camel, +9 Ship, +7 Mameluke | 325F 360G / 125 s (DE 105 s) |

Classes: Cavalry (scouts, knights); Camel (camels - **not** Cavalry, so they take the smaller anti-camel bonus).
Scout line has innate conversion resistance (section 8.6). All benefit from Bloodlines (+20 HP), Husbandry (+10%
speed), Blacksmith cavalry attack/armour.

### 5.7 Monastery

| Unit | Age | Cost | HP | Armour | Conversion range | Heal range | LOS | Speed | Train | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| Monk | C | 100G | 30 | 0/0 | 9 (0 for buildings/rams/trebuchets: must touch) | 4 | 11 (9 carrying a relic) | 0.7 | 51 | Rests 62 s after a successful conversion ("faith" bar recharges). Class: Monk. |
| Missionary (Spanish) | C | 100G (DE 30F 100G) | 30 | 0/0 | 7 | 4 | 9 | 1.1 | 51 | Classes: Monk, Cavalry. Cannot carry relics (AoC). |

Conversion and healing mechanics: section 8.6 and 8.7. Monk techs: section 7.5. Monks cannot attack and never
auto-convert; attack-move with Monks only heals.

### 5.8 Market / Dock civilians

| Unit | Age | Building | Cost | HP | Armour | LOS | Speed | Train | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Trade Cart | F | Market | 100W 50G | 70 | 0/0 | 7 | 1.0 (DE 1.25) | 51 | Gold formula in 2.6. Caravan +50% speed (AoC). |
| Trade Cog | F | Dock | 100W 50G | 80 | 0/6 | 6 | 1.32 (DE 1.65) | 36 | Class Ship. |
| Fishing Ship | D | Dock | 75W | 60 (DE 50) | 0/4 (DE 1/1) | 5 | 1.26 | 40 | Carry 15 (Gillnets etc. +5); gathers 0.28/s shore, 0.49/s deep (DE 0.24/0.42), 0.35/s traps. Builds Fish Traps. |
| Transport Ship | F in AoC (D in DE) | Dock | 125W | 100 (DE 70) | 4/8 (DE 0/2) | 5 | 1.45 | 46 | Carries 5 units (10 with Careening, 15 with Dry Dock in AoC; DE 20 base). Saracens 2x HP, +5. |

### 5.9 Siege Workshop and Castle siege

| Unit | Age | Building | Cost | HP | Atk | Armour | Range | LOS | Speed | Reload | Train | Bonuses | Upgrade |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Battering Ram | C | Siege Workshop | 160W 75G | 175 | 2 M | -3/180 | 0 | 3 (DE 5) | 0.5 (DE 0.6) | 5 | 36 | +125 All Buildings (DE +150), +40 Siege | - |
| Capped Ram | I | Siege Workshop | 160W 75G | 200 | 3 M | -2/190 | 0 | 3 | 0.5 | 5 | 36 | +150 All Buildings (DE +160), +50 Siege | 300F / 50 s |
| Siege Ram | I | Siege Workshop | 160W 75G | 270 | 4 M | -1/195 | 0 | 3 | 0.5 | 5 | 36 | +200 All Buildings, +65 Siege; small blast | 1000F / 75 s |
| Mangonel | C | Siege Workshop | 160W 135G | 50 | 40 M | 0/6 | 7 (min 3) | 9 | 0.5 (DE 0.6) | 6 | 46 | +35 All Buildings, +12 Siege (TC); area 1 tile (tapering) | - |
| Onager | I | Siege Workshop | 160W 135G | 60 | 50 M (DE 55) | 0/7 (DE 8) | 8 (min 3) | 10 | 0.5 | 6 | 46 | +45 All Buildings, +12 Siege; area 1.25; can cut trees | 800F 500G / 75 s |
| Siege Onager | I | Siege Workshop | 160W 135G | 70 | 75 M | 0/8 | 8 (min 3) | 10 | 0.5 | 6 | 46 | +60 All Buildings, +12 Siege; area 1.5 | 1450F 1000G / 150 s |
| Scorpion | C | Siege Workshop | 75W 75G | 40 | 12 P (DE 11) + 0 M | 0/6 (DE 7) | 7 (min 2; AoK 5/1) | 9 | 0.65 | 3.6 | 30 | +6 War Elephant (DE +7), +2 All Buildings (DE +3), +1 Ram; bolt passes through (50% to others) | - |
| Heavy Scorpion | I | Siege Workshop | 75W 75G | 50 (DE 60) | 16 P (DE 14) | 0/7 (DE 1/8) | 7 (min 2) | 9 | 0.65 | 3.6 | 30 | +10 War Elephant, +6 All Buildings, +2 Ram, +2 Infantry (DE) | 1000F 1100W / 50 s (DE 800F 750W) |
| Bombard Cannon | I (Chemistry) | Siege Workshop | 225W 225G | 80 | 40 M | 2/5 | 12 (min 5) | 14 | 0.7 | 6.5 | 56 | +200 All Buildings, +40 Ship, +20 Siege (DE also +40 Stone Defense); 92% acc; small blast | - |
| Trebuchet | I | Castle | 200W 200G | 150 | 200 M | 2/8 unpacked, 1/150 packed | 16 (min 4) | 19 | 0.8 (packed) | 10 | 50 | +250 All Buildings; ~15% acc vs units; must pack (5 s) to move | - |
| Petard | C | Castle | 80F 20G (DE 65F) | 50 | 25 M | 0/2 | 0 | 4 | 0.8 | - | 25 | +500 All Buildings, +900 Wall/Gate, +100 Castle, +60 Siege; suicide, blast 0.5 | - |

Classes: Siege Weapon (all); Ram (rams, packed Trebuchet - huge pierce armour so arrows do 1 damage); Gunpowder
(Bombard Cannon). Mangonel line and Bombard Cannon deal **melee** damage (good vs other siege). Rams can garrison
infantry (AoC: 4, DE 6) which adds speed and attack; Villagers since HD. Siege is repaired by Villagers, not healed by
Monks; immune to conversion without Redemption. Siege Engineers: +1 range (+20% vs buildings).

### 5.10 Dock: warships

| Unit | Age | Cost | HP | Atk | Armour | Range | LOS | Speed | Reload | Train | Bonuses | Upgrade |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Galley | F | 90W 30G | 120 (DE 110) | 6 P | 0/6 (DE 0/3) | 5 | 7 | 1.43 (DE 1.36) | 3 | 60 (AoK 36; DE 36) | +8 All Buildings (DE +6), +3 Ram | - |
| War Galley | C | 90W 30G | 135 (DE 125) | 7 P | 0/6 (DE 0/4) | 6 | 8 | 1.43 | 3 | 36 (DE 27) | +7 All Buildings, +4 Ram | 230W 100G (DE 150W 100G) / 50 s |
| Galleon | I | 90W 30G | 165 (DE 155) | 8 P (DE 9) | 0/8 (DE 0/6) | 7 | 9 | 1.43 | 3 | 36 (DE 27) | +8 All Buildings, +4 Ram | 400W 315G / 65 s |
| Fire Ship | C (DE: Fire Galley in Feudal) | 75W 45G | 100 (DE 120) | 2 P + 1 M (DE 3 M) | 0/6 (DE 2/4) | 2.49 | 5 | 1.43 (DE 1.47) | 0.25 | 36 (DE 27) | +2 All Buildings; continuous flame | - |
| Fast Fire Ship | I | 75W 45G | 120 (DE 140) | 3 P + 1 M (DE 4 M) | 0/8 (DE 3/7) | 2.49 | 6 | 1.43 (DE 1.56) | 0.25 | 36 | +3 All Buildings | 280W 250G / 50 s (verify) |
| Demolition Ship | C (DE: Raft in Feudal) | 70W 50G (DE 45W 80G) | 60 (AoK 50; DE 50) | 110 M (DE 95) | 0/3 (DE 1/0) | 0 | 6 | 1.6 (DE 1.52) | - | 31 | +220 All Buildings; suicide blast ~2.5 | - |
| Heavy Demolition Ship | I | 70W 50G | 60 (DE 70) | 140 M (DE 120) | 0/5 (DE 2/0) | 0 | 6 | 1.6 | - | 31 | +280 All Buildings | 200W 300G / 50 s |
| Cannon Galleon | I (Chemistry) | 200W 150G | 120 | 35 M (DE 50) | 0/6 (DE 0/5) | 13 (min 3) | 15 | 1.1 (DE 1.05) | 10 | 46 | +200 All Buildings, +40 Siege (DE +25); 50% acc (100% DE) | - |
| Elite Cannon Galleon | I | 200W 150G | 150 | 45 M (DE 60) | 0/8 (DE 0/6) | 15 (min 3) | 17 | 1.1 | 10 | 46 | +275 All Buildings, +40 Siege (DE +25) | 525W 500G / 30 s |
| Longboat (Vikings) | C | 100W 50G | 130 (DE 125) | 7 P x (1 + 2 extra arrows) (DE 5 P) | 0/6 (DE 0/3) | 6 | 8 | 1.54 (DE 1.46) | 3 | 25 (AoK 41) | +9 Ship? (DE: +2 Ram, +1 warship), +4 Ram | - |
| Elite Longboat | I | 100W 50G | 160 (DE 130) | 8 P x (1 + 3 extra) (DE 7 P) | 0/8 (DE 1/6) | 7 | 9 | 1.54 | 3 | 25 | as above | 750F 475G / 60 s |
| Turtle Ship (Koreans) | C | 200W 200G (DE 190W 180G) | 200 (DE 260) | 50 M (DE 20 + rockets) | 6/5 (DE 2/7) | 6 | 8 | 0.9 (DE 1.0) | 6 | 50 | +7 All Buildings, +7 Castle, +6 Wall/Gate (DE list) | - |
| Elite Turtle Ship | I | 200W 200G | 300 (DE 350) | 50 M (DE 25) | 8/6 (DE 3/12) | 6 (DE 7) | 9 | 0.9 | 6 | 50 | as above | 1000F 800G / 65 s (DE 650F 500G) |

Classes: Ship (all); Gunpowder (cannon galleons, turtle ships); DE adds Fire Ship, Long-range Warship and Capital
Ship classes. Ships are repaired by Villagers. Dock techs: Careening (+1 pierce armour, +5 transport capacity),
Dry Dock (+15% speed, +10 transport capacity; DE changed to +1 pierce), Shipwright (-20% wood cost, faster build).
Galley-line arrows benefit from Fletching/Bodkin/Bracer and Ballistics, and from Chemistry (+1).

### 5.11 Unique units (Castle unless noted)

Stats in "base / Elite" pairs; civ bonuses not applied. AoC values with DE notes; see the civ tables for narrative.

| Unit (civ) | Cost | HP | Atk | Armour | Range | LOS | Speed | Reload | Train | Bonuses | Classes | Elite upgrade |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Longbowman (Britons) | 35W 40G | 35 / 40 | 6 / 7 P | 0/0 / 0/1 | 5 / 6 (+civ bonus) | 7 / 8 | 0.96 | 2 | 19 (DE 18) | +2 Spearman | Archer, UU | 850F 850G / 60 s |
| Cataphract (Byzantines) | 70F 75G | 110 / 150 | 9 / 12 M | 2/1 | 0 | 4 / 5 | 1.35 | 1.8 / 1.7 (AoK 2) | 20 (AoK 23) | +9 / +12 Infantry (DE +13/+18); Cavalry-class armour +12 / +16 | Cavalry, UU | 1600F 800G / 50 s (DE 1200F 800G) |
| Woad Raider (Celts) | 65F 25G (DE 70F) | 65 / 80 (DE 70/85) | 8 / 13 M (DE 11/15) | 0/1 | 0 | 3 / 5 (+2) | 1.03 base (1.2 with bonus; DE 1.17) | 2 | 10 (AoK 16) | +2 / +3 Eagle, +2 / +3 Standard Building | Infantry, UU | 1000F 800G / 45 s |
| Chu Ko Nu (Chinese) | 40W 35G | 45 / 50 | 8 / 8 P (DE Elite 10) + 2 / 4 extra arrows of 3 | 0/0 | 4 | 6 | 0.96 | 3 (3.44 / 3.89 in practice) | 19 / 13 (DE 16/13) | +2 Spearman | Archer, UU | 1300F 1300G / 50 s (AoK 950/950) |
| Throwing Axeman (Franks) | 55F 25G | 50 / 60 (DE 60/70) | 7 / 8 M (ranged) | 0/0 / 1/0 | 3 / 4 (DE 5/6) | 5 / 6 (+2) | 0.9 (DE 1.0) | 2 | 17 (DE 13) | +1 / +2 Eagle, +1 / +2 Standard Building | Infantry, UU | 1000F 850G / 45 s (DE 850F 550G) |
| Huskarl (Goths) | 80F 40G (DE 75F 35G) | 60 / 70 | 10 / 12 M | 0/6 / 0/8 (AoK 0/4 / 0/6) | 0 | 3 / 5 (+2) | 1.05 (AoK 0.9) | 2 | 16 (DE 13; AoK 26) | +6 / +10 Archer (AoK +3/+4), +2 / +3 Eagle, +2 / +3 Standard Building (+civ) | Infantry, UU | 1200F 550G / 40 s |
| Samurai (Japanese) | 60F 30G (DE 45F 30G) | 60 / 80 (DE 70/80) | 8 / 12 M (DE 10/12) | 1/1 (AoK 1/0) | 0 | 4 / 5 (+2) | 1.0 (AoK 0.9) | 1.9 (AoK 2) | 9 (AoK 16) | +10 / +12 Unique Unit (AoK +5/+6), +2 / +3 Eagle, +2 / +3 Standard Building | Infantry, UU | 950F 875G / 60 s (DE 750F 650G) |
| Mangudai (Mongols) | 55W 65G | 60 / 60 | 6 / 8 P | 0/0 / 1/0 | 4 | 6 | 1.45 (DE 1.4) | 2.1 (AoK 2) | 26 (AoK 21) | +3 / +5 Siege, +1 Spearman (TC); 95% acc | Archer, Cavalry Archer, Cavalry, UU | 1100F 675G / 50 s |
| War Elephant (Persians) | 200F 75G (DE 170F 85G) | 450 / 600 | 15 / 20 M | 1/2 / 1/3 | 0 | 4 / 5 (DE 7/8) | 0.6 (DE 0.8) | 2 | 31 (DE 25) | +7 / +10 All Buildings and Stone Defense (DE +30); Elite tramples (0.5 tile, 50%) | Cavalry, War Elephant, UU | 1600F 1200G / 75 s (DE 1350F 800G) |
| Mameluke (Saracens) | 55F 85G | 65 / 80 (DE 80/90 base) | 7 / 10 M (DE 8/10), ranged 3 | 0/0 / 1/0 | 3 | 5 | 1.4 | 2 | 23 | +9 / +12 Cavalry; AoC also +11 anti-cavalry armour | Camel, Mameluke, UU (AoC also Archer class) | 600F 500G / 50 s |
| Teutonic Knight (Teutons) | 85F 40G (DE 30G) | 70 / 100 (DE 90/110) | 12 / 17 M (DE 14/17) | 5/2 / 10/2 (DE 7/2 / 8/2 base) | 0 | 3 / 5 (+2) | 0.7 (DE 0.8) | 2 | 12 (AoK 19) | +4 Eagle, +4 Standard Building | Infantry, UU | 1200F 600G / 50 s (DE 950F 500G) |
| Janissary (Turks) | 60F 55G | 35 / 40 (x1.25 civ) | 17 / 22 P (AoK 15/18) | 1/0 / 2/0 | 8 / 8 (DE 7/8) | 10 | 0.96 | 3.45 | 21 | +2 / +3 Ram; 50% / 65% acc; no min range | Archer, Gunpowder, UU | 850F 750G / 55 s |
| Berserk (Vikings) | 65F 25G (DE 20G) | 48 / 60 base (DE 54/62) | 9 / 14 M (DE 12/14) | 0/1 / 2/1 (AoK 0/0) | 0 | 3 / 5 (+2) | 1.05 (AoK 0.9) | 2 | 16 / 16 (DE 14/12) | +2 / +3 Eagle, +2 / +3 Standard Building; regen 20 HP/min (DE 40) | Infantry, UU | 1300F 550G / 45 s (DE 1075F 475G) |
| Jaguar Warrior (Aztecs) | 60F 30G | 50 / 75 (DE 65/75) | 10 / 12 M (DE 15/19) | 1/0 / 2/0 (DE 1/2 / 2/2) | 0 | 3 / 5 (+2) | 1.0 | 2 | 20 (DE 12) | +10 / +11 Infantry (DE +5/+6), +2 Eagle, +2 Standard Building | Infantry, UU | 1000F 500G / 45 s |
| Tarkan (Huns) | 60F 60G | 90 / 150 (DE 100/150) | 7 / 11 M (DE 8/11) | 1/2 / 1/3 (DE 1/3 / 1/4) | 0 | 5 / 7 | 1.35 (DE 1.4) | 2.1 (DE 1.9) | 14 | +8 / +10 All Buildings; DE also +12 Stone Defense, +10 Castle, +8/+10 Wall/Gate | Cavalry, UU | 1000F 500G / 45 s |
| War Wagon (Koreans) | 120W 60G (DE 200W -50%) | 150 / 200 | 9 / 9 P | 0/3 / 0/4 (DE 0/2 / 0/4) | 4 / 5 (5/6 before 1.0b) | 8 / 9 (DE 7/8) | 1.2 | 2.5 | 25 (DE 21) | +5 Standard Building (DE +2); 100% acc | Archer, Cavalry Archer, Cavalry, UU | 1000W 800G / 75 s |
| Plumed Archer (Mayans) | 46W 46G (DE 55W 55G) base, minus civ discount | 50 / 65 | 5 / 5 P | 0/1 / 0/2 | 4 / 5 | 6 / 7 | 1.2 | 1.9 | 16 | +2 Spearman, +1 / +2 Infantry; 80% / 90% acc | Archer, UU | 500F 1000W / 45 s (DE 700F 1000W) |
| Conquistador (Spanish) | 60F 70G (DE 75F 60G) | 55 / 70 | 16 / 18 P (DE 19) | 2/2 (DE 2/1 / 2/2) | 6 | 8 / 9 | 1.3 | 2.9 | 24 (DE 26) / 24 | +4 / +6 Ram (DE), Elite +2 All Buildings; 65% / 70% acc; no min range | Archer, Cavalry, Gunpowder, UU (DE adds Cavalry Archer) | 900F 600G / 60 s |
| Turtle Ship, Longboat, Missionary | see 5.10 and 5.7 | | | | | | | | | | | |

"+2 LOS" on infantry UUs is the free Feudal LOS bonus (AoC: Tracking).

---

## 6. Buildings

Key: Cost | HP | Armour melee/pierce (base, Dark Age; most buildings gain +1/+1 per age reached, see 1.3) | Size in
tiles | Garrison | Build time with one Villager | Age | Prerequisite building. "Prod. garrison" = units can only be
placed inside by setting the rally point on the building; they heal but add no arrows.

HP notes: AoC gives every economy building its full HP from the start; HD "The Forgotten" staggered Dark/Feudal HP
(DE values in brackets). Masonry +10% HP +1/+1 (+3 building-class armour), Architecture the same again.

### 6.1 Economic and town buildings

| Building | Cost | HP (AoC; DE) | Armour | Size | Garrison | Build | Age | Prereq | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Town Center | 275W 100S (AoK 275W) | 2400 | 3/5 (foundation 0/7) | 4x4 | 15 (TC also heals) | 150 (AoK 100) | D | - (limit 1 until Castle Age) | +5 pop. Fires 1 arrow (5 pierce, range 6, reload 2) per garrisoned Villager/archer, up to 10 arrows (DE); +5 vs Ship, +5 vs All Buildings. LOS 8. Needs flat 4x4 terrain. Cannot be converted. Trains Villagers, researches ages/Loom/Wheelbarrow/Hand Cart/Town Watch/Town Patrol. Repair costs wood only. |
| House | 30W (HD/DE 25W) | 900 (DE 550/750/900/900) | 0/7 | 2x2 | 0 | 25 | D | - | +5 pop. LOS 2. Huns cannot build. |
| Mill | 100W | 1000 (DE 600/800/1000) | 0/7 | 2x2 | 0 | 35 | D | - | Food drop-off; farm techs; farm queue. Prereq for Farm and Market. |
| Lumber Camp | 100W | 1000 (DE 600/800/1000) | 0/7 | 2x2 | 0 | 35 | D | - | Wood drop-off; wood techs. |
| Mining Camp | 100W | 1000 (DE 600/800/1000) | 0/7 | 2x2 | 0 | 35 | D | - | Gold/stone drop-off; mining techs. |
| Farm | 60W | 480 | 0/0 | 3x3 | 0 | 15 | D | Mill | 175 food (section 2.4). Walkable. Not auto-targeted by enemies. |
| Dock | 150W | 1800 | 0/7 | 3x3 (on shoreline) | prod. 10 (DE: also 5 Fishing Ships) | 35 | D | - | Trains ships, Dock techs, fishing drop-off, trade cog home. LOS 6. |
| Fish Trap | 100W | 50 (DE 250) | 0/0 | 1x1 (water) | 0 | 53 (AoK 15; DE 40) | F | Dock | 715 food. |
| Market | 175W | 2100 (DE 1800 in Feudal) | 1/8 | 4x4 | prod. 10 | 60 | F | Mill | Trade, tribute, Trade Carts, Market techs. LOS 6. |
| Blacksmith | 150W | 2100 (DE 1800 in Feudal) | 1/8 | 3x3 | 0 | 40 | F | - | All attack/armour techs. Prereq for Siege Workshop. |
| Monastery | 175W | 2100 | 2/9 | 3x3 | 10 (relics + prod. monks) | 40 | C | - | Monks, relics, Monastery techs. |
| University | 200W | 2100 | 2/9 | 4x4 | 0 | 60 | C | - | Building/missile techs. |
| Wonder | 1000W 1000S 1000G | 4800 | 3/10 | 5x5 | 0 | 3500 (~58 min solo; ~6 min with 25 Villagers) | I | - | Victory countdown (6.9). Cannot be converted. Reveals its location to everyone when started. |

### 6.2 Military production buildings

| Building | Cost | HP by age (F/C/I) | Armour | Size | Garrison | Build | Age | Prereq | Trains |
|---|---|---|---|---|---|---|---|---|---|
| Barracks | 175W | 1200 (D) / 1500 / 1800 / 2100 | 0/7 | 3x3 | prod. 10 | 50 | D | - | Militia line, Spearman line, Eagles (Meso); Tracking, Squires, (Arson, Gambesons DE). Prereq for Archery Range and Stable. |
| Archery Range | 175W | 1500 / 1800 / 2100 | 1/8 | 3x3 | prod. 10 | 50 | F | Barracks | Archer line, Skirmisher line, Cavalry Archer line, Hand Cannoneer; Thumb Ring, Parthian Tactics. |
| Stable | 175W | 1500 / 1800 / 2100 | 1/8 | 3x3 | prod. 10 | 50 | F | Barracks | Scout line, Knight line, Camel line; Bloodlines, Husbandry. |
| Siege Workshop | 200W | 1800 / 2100 (DE 1500 F) | 2/9 | 4x4 | prod. 10 | 40 | C | Blacksmith | Rams, Mangonel line, Scorpion line, Bombard Cannon. |
| Castle | 650S | 4800 | 8/11 | 4x4 | 20 (foot, mounted, siege by production) | 200 (AoK 150) | C | - | +20 pop. Unique unit, Petard, Trebuchet; unique tech, Conscription, Sappers, Hoardings, Spies/Treason. 5 base arrows of 11 pierce (range 8, min 1, reload 2) + up to 20 extra from garrison; +2 vs Spearman, +11 vs Ship, +1 vs Camel. Class Castle (Petard +100, Tarkan +10). Heals garrison at 0.2 HP/s (towers/TC 0.1). LOS 11. Cannot be converted. Prereq (alone) for the Imperial Age. Hoardings +21% HP (~1000). |

### 6.3 Defensive structures

| Building | Cost | HP | Armour | Size | Garrison | Build | Age | Prereq / tech | Attack | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| Outpost | 25W 25S (TC 25W 10S; DE 5S) | 500 | 0/0 | 1x1 | 0 | 15 | D | - | none | LOS 6 (AoK 4), +2 per age up to 12. |
| Palisade Wall | 2W (DE 3W) | 250 (DE 150 Dark / 250 Feudal) | 2/2 (DE 2/5) | 1x1 (drag to lay lines) | 0 | 5 (DE 7) | D | - | none | Wall & Gate class. Foundation 0 armour. |
| Palisade Gate (HD/DE only) | 20W (DE 30W) | 400 (DE 240/400) | 2/2 (DE 2/4) | 4x1 | 0 | 30 | D | - | none | Not in AoC. |
| Stone Wall | 5S | 1800 (DE 1080 Feudal / 1800 Castle+) | 8/10 | 1x1 | 0 | 8 (DE 10) | F | - | none | +16 vs building-class bonus damage. Stone Defense class. |
| Fortified Wall | 5S (upgrade of existing walls) | 3000 | 12/12 | 1x1 | 0 | 8 | C | University: Fortified Wall tech (200F 100W, 50 s) | none | Upgrades every Stone Wall and Gate in place. +24 building-class armour. |
| Gate | 30S | 2750 (DE 1650 Feudal / 2750 Castle+); Fortified Gate ~4000 (verify) | 6/6 | 4x1 (or 1x4; diagonal variants) | 0 | 70 | F | - | none | Opens for own/allied units automatically; can be locked open/closed. Cannot be converted. Must sit on a straight wall line (or free). |
| Watch Tower | 25W 125S (HD 50W; DE 35W) | 1020 (DE 850 Feudal / 1020 Castle) | 1/7 | 1x1 | 5 (Teutons 10) | 80 | F | - | 5 P, range 8, min range 1 (removed by Murder Holes), reload 2, 100% acc; +1 Camel, +7 Ship | +1 arrow per garrisoned Villager/archer (max +4). LOS 10 (8 in AoK). Stone Defense class. |
| Guard Tower | same as Watch Tower (upgrade in place) | 1500 | 2/8 | 1x1 | 5 | 80 | C | University: Guard Tower (100F 250W, 30 s) | 6 P (DE 7), range 8 | +9 vs Ship (DE +7). |
| Keep | same | 2250 | 3/9 | 1x1 | 5 | 80 | I | University: Keep (500F 350W, 75 s) | 7 P (DE 8), range 8 | +10 vs Ship (DE +8). |
| Bombard Tower | 125S 100G | 2220 | 3/9 | 1x1 | 5 | 80 | I | University: Bombard Tower (800F 400W, 60 s) + Chemistry | 120 M (AoK melee, pierce since 1.0b), range 8, min 1, reload 6, +40 Ship | Gunpowder class. Fires cannonballs with small blast. Turks Artillery +2 range. |

Towers/Castle/TC: garrisoned Villagers add 1 arrow each (2.5 "firepower" vs a 2.5 building), archers add by their DPS
(section 8.5). Murder Holes removes minimum range. Heated Shot +125% attack vs ships in AoC (DE: +25% towers, +50% Castle). Arrowslits (DE) +1 tower attack.

### 6.4 Build order dependencies (tech-tree graph)

```
Town Center ─┬─ House, Mill ─ Farm, Market(F) ─ (trade, Coinage, Caravan, Banking, Guilds)
             ├─ Lumber Camp, Mining Camp, Outpost, Palisade Wall
             ├─ Dock ─ Fish Trap(F)
             └─ Barracks ─┬─ Archery Range(F)
                          └─ Stable(F)
Feudal Age ──── Blacksmith ─ Siege Workshop(C)
                Watch Tower, Stone Wall, Gate
Castle Age ──── Monastery, University, Castle, extra Town Centers
Imperial Age ── Wonder; Keep & Bombard Tower via University techs
```

### 6.5 Garrison capacities summary

| Container | Capacity | Who | Effect |
|---|---|---|---|
| Town Center | 15 (Teutons DE 25) | Villagers, foot soldiers, archers, Monks, King | +1 arrow each for Villagers/archers (max 10 extra), heal 0.1 HP/s |
| Towers | 5 (Teutons 10) | same | +1 arrow each (max 4), heal 0.1 HP/s |
| Castle | 20 | all land units incl. cavalry (siege only by production) | archers/Villagers add arrows (max 20 extra), heal 0.2 HP/s |
| Production buildings | 10 | newly trained units only | heal only |
| Monastery | 10 | relics (+ new Monks) | relic gold |
| Transport Ship | 5 (+5 Careening, +5 Dry Dock; Saracens +5; DE 20) | any land unit | carry |
| Battering Ram line | 4 (DE 6; AoK 0) | infantry (Villagers since HD) | ram +speed, +attack |

Herbal Medicine: garrisoned units heal 4x faster (DE 6x). Garrisoned units are untargetable; the building ejects
them when it reaches 20% HP or is destroyed (no damage to the units). Units inside do not count as "idle" and still
occupy population.

### 6.6 Building placement rules

- Buildings snap to the tile grid; a foundation can be placed on any unobstructed, buildable terrain. Buildings of
  3x3 and larger need (mostly) flat terrain; the TC and Wonder need fully flat ground. 1x1 and 2x2 buildings and
  farms can be placed on slopes.
- Foundations have 1 HP per build-second until finished; an unfinished foundation blocks movement like a finished
  building but has 0 armour. Foundations cannot be converted or garrisoned.
- Walls can be dragged in straight or diagonal lines; each segment is a separate 1x1 building. Gates need a
  4-tile-long straight wall stretch (or empty line).
- Docks must touch water on their waterline side; Fish Traps must be in water.
- A player can only have 1 Town Center until the Castle Age (unless they own none).
- Enemy units standing on a tile block foundation placement; your own Villagers are pushed out.

### 6.7 Building HP by age (summary of the "free" per-age upgrades)

| Building | Dark | Feudal | Castle | Imperial |
|---|---|---|---|---|
| Barracks | 1200 | 1500 | 1800 | 2100 |
| Archery Range / Stable | - | 1500 | 1800 | 2100 |
| Siege Workshop | - | - | 1800 | 2100 |
| House (DE) | 550 | 750 | 900 | 900 |
| Mill / camps (DE) | 600 | 800 | 1000 | 1000 |
| Market / Blacksmith (DE) | - | 1800 | 2100 | 2100 |
| Armour (all except walls/farms/outposts) | 0/7 | 1/8 | 2/9 | 3/10 |
| Town Center armour | 3/5 | 4/6 | 5/7 | 6/8 |

### 6.8 Conversion immunity (buildings)

Never convertible (even with Redemption): Town Center, Castle, Monastery, Wonder, walls, gates, farms, fish traps,
foundations. Everything else needs Redemption and a Monk standing next to it (range 0).

### 6.9 Wonder / relic victory countdown [S-Wonder]

When the countdown reaches zero the owner (or their team with Allied Victory) wins. Each Wonder has its own timer;
destroying it cancels that timer. The relic countdown works the same once a team holds every relic. In AoC the
timer is 200 "years"; 1 year = 5 s of game time, so 1000 s (16:40) at 1.0x, i.e. ~11 real minutes at 1.5x speed.
DE scales with map size:

| Map size | Tiny-Medium | Normal | Large | Huge/Giant | larger DE sizes |
|---|---|---|---|---|---|
| Years | 200 | 300 | 350 | 400 | 450-500 |
| Game seconds (1 yr = 5 s) | 1000 | 1500 | 1750 | 2000 | 2250-2500 |

Huns' Atheism adds 100 years (500 s) to everyone else's countdown.

---

## 7. Technologies

Cost / research time are AoC values (verified against the DE game data; DE deltas noted). "Effect" is what the engine
applies. Nearly every tech has a line prerequisite (Bodkin needs Fletching, etc.).

### 7.1 Town Center

| Tech | Age | Cost | Time | Effect |
|---|---|---|---|---|
| Loom | D | 50G | 25 | Villagers +15 HP, +1/+2 armour. |
| Feudal Age | D | 500F | 130 | See section 1. |
| Castle Age | F | 800F 200G | 160 | |
| Imperial Age | C | 1000F 800G | 190 | |
| Town Watch | F | 75F | 25 | Buildings +4 LOS. |
| Wheelbarrow | F | 175F 50W | 75 | Villagers +10% speed, +25% carry (rounded up: 10 -> 13; hunters 35 -> 44). |
| Town Patrol | C | 300F 100G | 40 | Buildings +4 LOS (stacks). |
| Hand Cart | C | 300F 200W | 55 | Villagers +10% speed, +50% carry (10 -> 15; hunters 35 -> 53). Requires Wheelbarrow. |

### 7.2 Mill, Lumber Camp, Mining Camp

| Tech | Building | Age | Cost | Time | Effect |
|---|---|---|---|---|---|
| Horse Collar | Mill | F | 75F 75W | 20 | Farms +75 food (250). |
| Heavy Plow | Mill | C | 125F 125W | 40 | Farms +125 food (375); farmers +1 carry. |
| Crop Rotation | Mill | I | 250F 250W | 70 | Farms +175 food (550). |
| Double-Bit Axe | Lumber Camp | F | 100F 50W | 25 | Lumberjacks +20% gather speed. |
| Bow Saw | Lumber Camp | C | 150F 100W | 50 | +20%. |
| Two-Man Saw | Lumber Camp | I | 300F 200W | 100 | +10%. |
| Gold Mining | Mining Camp | F | 100F 75W | 30 | Gold miners +15%. |
| Gold Shaft Mining | Mining Camp | C | 175F 75W | 75 | +15%. |
| Stone Mining | Mining Camp | F | 100F 75W | 30 | Stone miners +15%. |
| Stone Shaft Mining | Mining Camp | C | 175F 75W | 75 | +15%. |

Percentages are additive on the base work rate (0.39 wood -> 0.468 -> 0.546 -> 0.585 ... the engine actually
multiplies: 0.39 x 1.2 x 1.2 x 1.1 = 0.618).

### 7.3 Blacksmith

| Tech | Age | Cost | Time | Effect |
|---|---|---|---|---|
| Forging | F | 150F | 50 | Infantry and cavalry +1 attack. |
| Iron Casting | C | 220F 120G | 75 | +1 attack. |
| Blast Furnace | I | 275F 225G | 100 | +2 attack. |
| Scale Mail Armor | F | 100F | 40 | Infantry +1/+1 armour. |
| Chain Mail Armor | C | 200F 100G | 55 | Infantry +1/+1. |
| Plate Mail Armor | I | 300F 150G | 70 | Infantry +1/+2. |
| Scale Barding Armor | F | 150F | 45 | Cavalry +1/+1. |
| Chain Barding Armor | C | 250F 150G | 60 | Cavalry +1/+1. |
| Plate Barding Armor | I | 350F 200G | 75 | Cavalry +1/+2. |
| Fletching | F | 100F 50G | 30 | Archers, towers, Castles, TCs, galleys: +1 attack, +1 range (+1 LOS). |
| Bodkin Arrow | C | 200F 100G | 35 | +1 attack, +1 range. |
| Bracer | I | 300F 200G | 40 | +1 attack, +1 range. |
| Padded Archer Armor | F | 100F | 40 | Archers (incl. CA, skirms, hand cannons) +1/+1. |
| Leather Archer Armor | C | 150F 150G | 55 | +1/+1. |
| Ring Archer Armor | I | 250F 250G | 70 | +1/+2. |

### 7.4 University

| Tech | Age | Cost | Time | Effect |
|---|---|---|---|---|
| Masonry | C | 150F 175W | 50 | Buildings +10% HP, +1/+1 armour, +3 building-class armour. |
| Architecture | I | 300F 200W | 70 | Buildings +10% HP, +1/+1, +3 building-class armour (requires Masonry). |
| Treadmill Crane | C | 300F 200W | 40 | Villagers build 20% faster. |
| Ballistics | C | 300W 175G | 60 | Archers, towers, Castles, TCs, galleys, Scorpions (DE) lead moving targets (section 8.4). |
| Murder Holes | C | 200F 100S | 35 | Towers and Castles lose their minimum range. |
| Heated Shot | C | 350F 100G | 30 | Towers and Castles +125% damage vs ships (DE: +25%/+50% towers/castle-type bonus). |
| Guard Tower | C | 100F 250W | 30 | Upgrades all Watch Towers. |
| Keep | I | 500F 350W | 75 | Upgrades all Guard Towers. |
| Bombard Tower | I | 800F 400W | 60 | Enables Bombard Towers (needs Chemistry). |
| Fortified Wall | C | 200F 100W | 50 | Upgrades all Stone Walls and Gates. |
| Chemistry | I | 300F 200G | 100 | Missile units (archers, skirms, CA, towers, Castles, ships) +1 attack; unlocks Hand Cannoneer, Bombard Cannon, Cannon Galleon, Bombard Tower. |
| Siege Engineers | I | 500F 600W | 45 | Siege units +1 range (Mangonel, Scorpion, Trebuchet, Bombard Cannon lines) and +20% attack vs buildings. |
| Arrowslits (HD/DE) | I | 250F 250W | 25 | Watch Tower line +1 attack (+2 Guard, +3 Keep in DE). |

### 7.5 Monastery

| Tech | Age | Cost | Time | Effect |
|---|---|---|---|---|
| Redemption | C | 475G | 50 | Monks can convert siege weapons and most buildings (range 0 for buildings). |
| Fervor | C | 140G | 50 | Monks +15% speed. |
| Sanctity | C | 175G | 60 | Monks +15 HP (45). |
| Atonement | C | 325G | 40 | Monks can convert enemy Monks. |
| Heresy (TC) | C | 1000G | 60 | Your units die instead of changing sides when converted. |
| Herbal Medicine (TC) | C | 200G | 35 | Garrisoned units heal 4x faster (DE 6x). |
| Illumination | I | 120G | 65 | Monks regain faith ~50% faster after a conversion (rest 62 s -> ~33 s). |
| Faith | I | 750G (DE 550F 750G) | 60 | All your units and buildings are harder to convert (+4 min / +4 max intervals in DE; AoC: resistance level +3, +2 min, +4 max). |
| Block Printing | I | 200G | 55 | Monks +3 conversion range (9 -> 12). |
| Theocracy (TC) | I | 200G | 75 | When several Monks convert together only one loses his faith. |

### 7.6 Market

| Tech | Age | Cost | Time | Effect |
|---|---|---|---|---|
| Cartography | F | 100F 100G | 30 | See allies' LOS (free and hidden since HD 5.7). |
| Coinage | F (HD 5.8+: C) | 200F 100G | 70 | Tribute tax 30% -> 20%. |
| Caravan (TC) | C | 200F 200G | 40 | Trade Carts/Cogs +50% speed (DE +20%). |
| Banking | C (HD 5.8+: I) | 300F 200G | 70 | Tribute tax 0%. |
| Guilds | I | 300F 200G | 50 | Commodity fee 30% -> 15%. |

### 7.7 Barracks, Archery Range, Stable, Siege Workshop, Dock, Castle

| Tech | Building | Age | Cost | Time | Effect |
|---|---|---|---|---|---|
| Tracking (AoC only) | Barracks | F | 50F | 35 | Infantry +2 LOS (DE removed; given free). |
| Squires | Barracks | C | 100F | 40 | Infantry +10% speed. |
| Arson (HD/DE) | Barracks | C (DE F) | 75F 25G | 25 | Infantry +2 vs standard buildings. |
| Thumb Ring (TC) | Archery Range | C | 300F 250W | 45 | Archers 100% accuracy (vs stationary), foot archers +18% fire rate. |
| Parthian Tactics (TC) | Archery Range | I | 200F 250G | 65 | Cavalry archers +1/+2 armour, +2 vs spearmen (DE: +4; Heavy CA +6 in AoC? - verify). |
| Bloodlines (TC) | Stable | F | 150F 100G | 50 | All mounted units +20 HP. |
| Husbandry | Stable | C | 150F (verify AoC; often listed 250F) | 40 | Mounted units +10% speed. |
| Siege unit upgrades | Siege Workshop | C/I | see 5.9 | | Capped Ram 300F, Siege Ram 1000F, Onager 800F 500G, Siege Onager 1450F 1000G, Heavy Scorpion 1000F 1100W. |
| Careening | Dock | C | 250F 150G (DE 100F 200G) | 50 | Ships +1 pierce armour, transports +5 capacity. |
| Dry Dock | Dock | I | 600F 400G (DE 200F 400G) | 60 | Ships +15% speed, transports +10 capacity (DE: +1 pierce armour only). |
| Shipwright | Dock | I | 1000F 300G | 60 | Ships cost -20% wood, build +35% faster (DE +54%). |
| Gillnets (TC) | Dock | C | 150F 100W (DE 100F 125W) | 45 (DE 30) | Fishing Ships work +25% faster (DE: +carry). |
| War Galley / Galleon / Fast Fire / Heavy Demo / Elite Cannon Galleon | Dock | C/I | see 5.10 | | DE merged these into "Medium/Heavy Warships". |
| Conscription | Castle | I | 150F 150G | 60 | Barracks, Archery Range, Stable, Castle produce 33% faster. |
| Sappers | Castle | I | 400F 200W | 10 | Villagers +15 vs buildings and +15 vs stone defence. |
| Hoardings | Castle | I | 400F 400W | 75 | Castles +21% HP (+1000, 4800 -> 5800). |
| Spies | Castle | I | 200G per enemy Villager (max 30000, i.e. 150 Villagers) | 1 | See everything enemies see for the rest of the game. Not in Regicide/Deathmatch. |
| Treason (Regicide) | Castle | I | 400G | 1 | Reveals all enemy Kings for a few seconds; repeatable. |
| Unique tech | Castle | I (DE: C + I) | per civ (section 4) | | |

### 7.8 Unit upgrade costs (for the data file)

| Upgrade | Age | Cost | Time |
|---|---|---|---|
| Man-at-Arms | F | 100F 40G | 40 |
| Long Swordsman | C | 200F 65G (DE 150F 65G) | 45 (DE 40) |
| Two-Handed Swordsman | I | 300F 100G (DE 200F 100G) | 75 (DE 45) |
| Champion | I | 750F 350G (DE 650F 350G) | 100 (DE 70) |
| Pikeman | C | 215F 90G (DE 160F 90G) | 45 (DE 35) |
| Halberdier | I | 300F 600G | 50 |
| Eagle Warrior (HD/DE) | C | 200F 200G | 50 |
| Elite Eagle Warrior | I | 800F 500G | 50 |
| Crossbowman | C | 125F 75G (DE 175F 100G) | 35 |
| Arbalester | I | 350F 300G (DE 450F 350G) | 50 |
| Elite Skirmisher | C | 250W 160G (DE 230W 130G) | 50 |
| Heavy Cavalry Archer | I | 900F 500G | 50 |
| Light Cavalry | C | 150F 50G | 45 |
| Hussar | I | 500F 600G | 50 |
| Cavalier | I | 300F 300G | 100 (DE 80) |
| Paladin | I | 1300F 750G | 170 |
| Heavy Camel | I | 325F 360G | 125 (DE 105) |
| Capped Ram | I | 300F | 50 |
| Siege Ram | I | 1000F | 75 |
| Onager | I | 800F 500G | 75 |
| Siege Onager | I | 1450F 1000G | 150 |
| Heavy Scorpion | I | 1000F 1100W (DE 800F 750W) | 50 |
| War Galley | C | 230W 100G (DE 150W 100G) | 50 |
| Galleon | I | 400W 315G | 65 |
| Fast Fire Ship | I | 280W 250G (verify) | 50 |
| Heavy Demolition Ship | I | 200W 300G | 50 |
| Elite Cannon Galleon | I | 525W 500G | 30 |
| Guard Tower | C | 100F 250W | 30 |
| Keep | I | 500F 350W | 75 |
| Fortified Wall | C | 200F 100W | 50 |
| Elite unique units | I | see 5.11 | |

---

## 8. Combat formulas

### 8.1 Armour classes [S-ArmorClass]

Every unit has a list of (class, armour value) pairs; every attack has a list of (class, attack value) pairs. Class 4
(melee) and class 3 (pierce) are the visible "attack" and "armour" numbers; the others are hidden bonuses. If a target
lacks a class entirely, its armour for that class is treated as infinite (base armour 1000 in AoC, 10000 in DE), so
the bonus simply does not apply.

| ID | Class | Who has it (AoC) |
|---|---|---|
| 0 | Wonder (DE) | Wonders |
| 1 | Infantry | Militia line, Spearman line, Eagles, infantry UUs (not foot archers) |
| 2 | Turtle Ship (capital ship) | Turtle Ships |
| 3 | Base pierce | everything (pierce armour / pierce attack) |
| 4 | Base melee | everything |
| 5 | War Elephant | War Elephants |
| 8 | Cavalry | Scouts, Knights, Cavalry Archers, Mangudai, Cataphracts, Tarkans, War Wagons, Missionaries, War Elephants, Conquistadors - but **not** camels |
| 11 | All Buildings | every building |
| 13 | Stone Defense | towers, Castle (secondary), stone walls, gates |
| 14 | Predator animal | wolves etc. |
| 15 | Archer | all foot and mounted archers, skirmishers, hand cannoneers, janissaries |
| 16 | Ship (and camel in AoC: "Ship & Camel") | all ships; in AoC camels and Mamelukes shared this class, which is why anti-ship bonuses hit camels |
| 17 | Ram | Battering/Capped/Siege Ram, packed Trebuchet |
| 18 | Tree | trees (for Onager tree-cutting) |
| 19 | Unique Unit | all castle unique units (Samurai bonus) |
| 20 | Siege Weapon | rams, mangonels, scorpions, bombard cannons, trebuchets, petards |
| 21 | Standard Building | all buildings except Wonder (always armour 0; Goth/infantry anti-building bonus) |
| 22 | Wall & Gate | palisade and stone walls, gates |
| 23 | Gunpowder Unit | Hand Cannoneer, Janissary, Conquistador, Bombard Cannon, Cannon Galleon, Bombard Tower, Turtle Ship |
| 24 | Boar | Wild Boar (Goth villager bonus) |
| 25 | Monk | Monks, Missionaries |
| 26 | Castle | Castles (Petard +100, Tarkan +10) |
| 27 | Spearman | Spearman line |
| 28 | Cavalry Archer | CA line, Mangudai, War Wagon (Conquistador only since DE) |
| 29 | Eagle Warrior (shock infantry) | Eagle line |
| 30 | Camel (DE) | camels, Mamelukes (split out of class 16 in HD) |
| 35 | Mameluke (DE) | Mamelukes |
| 36 | Hero / King | Kings, campaign heroes |

Positive armour in a bonus class resists that bonus (Cataphract +12 cavalry armour); negative armour amplifies it
(Elephant Archer -4 CA armour in DE).

### 8.2 Damage formula [S-ArmorClass][S-Damage]

For one hit of attacker A on target T:

```
damage_i = max(0, attack_i(A) - armour_i(T))        for each armour class i the attacker has an attack value for
raw      = sum_i damage_i
total    = max(1, raw * k_elevation)                 # k = 1.25 attacker higher, 0.75 attacker lower, 1 otherwise
```

- Classes with no attack value contribute nothing; classes the target lacks contribute nothing (infinite armour).
- Minimum damage is 1 per hit (so rams still scratch anything, arrows still tick walls).
- Damage can be fractional (hill multiplier, blast tapering); HP is stored as a float and a unit dies when HP < 1.
- Blacksmith "+1 attack" adds to the base melee (4) or pierce (3) class only; bonuses are flat and are not improved by
  attack upgrades (exception: percentage multipliers such as Siege Engineers' +20% vs buildings, Bohemian bonus,
  which multiply the stored bonus value and round to an integer before the subtraction).
- Example: Elite Skirmisher (3 pierce, +4 archer, +3 spearman) vs Spearman (0/0): 3 + 3 = 6. Vs Crossbowman
  (0/0 + archer armour 0): 3 + 4 = 7. Vs Knight (2/2): max(0, 3-2) = 1.
- Example: Halberdier (6 melee, +32 cavalry) vs Paladin (2/3): (6-2) + 32 = 36 per hit, so 5 hits kill a 160-HP
  Paladin.
- Example: Pikeman (4 melee, +22 cavalry) vs Cataphract (2/1, cavalry armour +12): (4-2) + (22-12) = 12.
- Example: Mangonel (40 melee, +35 building) vs House (0/7): 40 + 35 = 75; vs Knight: 40 - 2 = 38 (main target),
  splash targets take tapered damage.
- Archer (4 pierce) vs Stone Wall (8/10): max(1, 4-10) = 1 per arrow.

Engine order (DE documentation; AoC is the same minus the extra multipliers): 1) apply percentage attack multipliers
to stored attack values (integers); 2) per class `max(min(0, atk), atk - armour)`; 3) bonus-damage resistance
multipliers (DE only); 4) sum; 5) elevation / misfire multipliers; 6) `max(1, ...)`.

### 8.3 Elevation and cliffs [S-Elevation]

- Any attacker standing on higher ground than its target (or above a cliff) deals **+25%** (x1.25) damage; standing
  lower deals **-25%** (x0.75). The size of the height difference does not matter; it is a binary check applied after
  armour subtraction. Buildings on hills get the bonus too.
- Both melee and ranged attacks are affected (unlike AoE1 where it was ranged only).
- Cliffs are impassable terrain features that count as a higher elevation level for the unit standing on top.
- Line of sight is not affected by elevation in AoE2 (DE: cliffs do not block LOS either); the only elevation effect
  on vision is cosmetic.
- Buildings 3x3+ cannot be placed on uneven terrain (see 6.6).

### 8.4 Projectiles, accuracy and Ballistics [S-Ballistics]

- Ranged attacks spawn a projectile with a flight speed (archer arrows 7 tiles/s, Scorpion bolts 6, mangonel stones
  3.5, cannonballs 1.95-5.5). Damage is applied on impact, not on launch.
- Accuracy (e.g. Archer 80%, CA 50%, Hand Cannoneer 65%): the roll decides whether the projectile is aimed exactly
  at the target's position or scattered randomly within a radius proportional to range. A scattered projectile that
  happens to hit any unit deals 50% damage to it (DE rule; AoC: full damage to whatever it hits).
- Without **Ballistics** the projectile is aimed at the target's position at the moment of firing, so a target that
  keeps moving sideways is missed. With Ballistics the aim point is extrapolated from the target's velocity (lead
  shooting), so units moving in a straight line are hit; changing direction still dodges. Mangonel, Trebuchet,
  Bombard Cannon and Cannon Galleon shots never lead the target (Spanish Cannon Galleons excepted). Scorpions use
  Ballistics only in DE.
- Thumb Ring sets archer accuracy to 100% (and +18% rate of fire for foot archers). Chemistry +1 attack to all
  projectile units.
- Minimum range: units closer than the minimum (Skirmisher 1, Scorpion 2, Mangonel 3, Trebuchet 4, Bombard Cannon
  5, towers/Castle 1 until Murder Holes) cannot be targeted.
- Projectile units have an attack delay (wind-up before the projectile launches; Archer 0.35 s, CA 1.0 s in AoC)
  which matters for hit-and-run.
- Area damage: Mangonel line (radius 1 / 1.25 / 1.5 tiles), Petard 0.5, Demolition Ships ~2.5, Bombard Cannon/Tower
  small, Siege Ram small, Elite War Elephant trample 0.5 at 50%. Friendly fire applies to Mangonel/Onager stones
  and demolition blasts (not to Scorpion bolts). The main target always takes 100%; others taper with distance
  (DE; AoC: inner 50% radius full damage, outer half damage).
- Attack ground: Mangonel line (and DE cannon galleons) can target a tile.

### 8.5 Garrison arrows [S-Garrison]

- TCs, towers and Castles fire their own arrow(s) only if the building has an innate attack: Watch Tower line and
  Castle fire 1 (Castle 5) arrows always; the TC fires **only** when at least one Villager/archer is inside (AoC).
- Each garrisoned Villager adds one arrow to a 2.5-DPS tower/TC (Villager "firepower" 2.5 = 5 attack / 2 s). Archers
  add arrows according to their DPS relative to the building's: `extra = floor(sum(unit DPS) / building DPS)`,
  capped at +4 for towers (Teutons +8), +10 for TCs, +16 for Castles (20 total). Infantry, cavalry, Monks add nothing
  (Teutons with Crenellations: infantry add arrows).
- Garrison arrows use the building's attack value and bonuses, range and Ballistics, and benefit from
  Fletching/Bodkin/Bracer/Chemistry.

### 8.6 Conversion [S-Conversion]

- A Monk in range (9; 0 for buildings/rams/trebuchets) attempts conversion once per **conversion interval** (CI, 1 s
  of game time in AoC; ~1.25 s in DE). Each attempt after the minimum has a fixed success chance; after the maximum
  the conversion is guaranteed.
- AoC: 25% chance per interval (28% in some references); minimum 4 CI, maximum 10 CI for units. Units with a
  **conversion resistance level** n divide the chance by n: buildings, ships and the Scout line had resistance 3 in
  AoK; TC raised Scouts/Eagles to resistance 8. Faith: resistance +3, min +2, max +4 (6-14). Teutons team bonus:
  resistance +2, min +1, max +2 (5-12).
- DE (update 99311): 38% per CI, units 5-9 CI; scouts 8-10; ships 7-11; Devotion +1/+1; Faith +4/+4; buildings
  15-25 CI at 13%; Teutons +3/+1.
- Several Monks on one target roll independently; success by any converts the unit and all of them lose their faith
  (unless Theocracy). Faith recharges in 62 s (Illumination ~33 s); a Monk cannot start converting below 100% faith.
- Switching targets keeps the interval count (except building -> unit in DE); manual move resets it.
- Immune: Kings, heroes, Monks (unless Atonement), siege and buildings (unless Redemption), TC/Castle/Monastery/
  Wonder/walls/gates/farms always, units inside buildings/transports. Heresy makes your converted units die instead.
- Converted units keep their original civilisation's stats/bonuses and their current upgrades; they do not receive
  the new owner's future upgrades. Converting ignores the population cap.
- Scouts/Light Cav/Hussar/Eagles also have +6/+10/+12 (+8/+10) attack vs Monks, making them the anti-Monk units.

### 8.7 Healing and regeneration [S-Healing][S-Monk]

- Monk heal rate: `75 x (n + 1)` HP/min for n Monks on one target, i.e. the first Monk heals 150 HP/min (2.5 HP/s),
  every additional one adds 75 HP/min. Heal range 4 (Teutons 8). Byzantine allies: +200% in AoK, +50% in AoC, +100%
  in DE. Monks cannot heal siege, ships, buildings, or themselves, and cannot heal while moving.
- Garrison healing: towers/TC 0.1 HP/s, Castle 0.2 HP/s; Herbal Medicine x4 (DE x6). Cavalry can only garrison in
  Castles and transports; units in transports/rams do not heal.
- Berserks regenerate 20 HP/min (Berserkergang: faster). Nothing else regenerates in AoC.
- Repair (buildings, siege, ships): Villagers only, see 2.8.

### 8.8 Speed, LOS and other combat constants

- Base movement speeds: Villager 0.8, infantry 0.9-1.05, archers 0.96, Monk 0.7, siege 0.5-0.8, Knight 1.35, Scout
  1.2/1.55, Light Cav/Hussar 1.5, Camel 1.45, CA 1.43, ships 1.1-1.6. Squires +10% infantry, Husbandry +10% cavalry.
- Units have a collision size (Villager/infantry 0.2-0.25 tile radius, Knights 0.25, rams ~0.5) that defines pathing
  and how many units can surround a target; roughly 8 melee units fit around a 1-tile target.
- LOS is a circle in tiles; building LOS is from the edge. Fletching line adds +1 LOS to archers as well as range.
  Town Watch/Patrol +4/+8 building LOS. Fog of war: explored terrain stays visible as last seen; units/buildings are
  only visible inside current LOS.
- Rate of fire: `reload` is the time between the *starts* of two attacks; the attack animation (frame delay) decides
  when in that window the hit lands. Melee hits are instant at the animation keyframe.
- Attack bonus vs buildings: Militia line +1/+2/+3/+4 from Man-at-Arms to Champion (see 5.2) and all Villagers
  +3; everything else does max(1, attack - building armour) which is why archers barely scratch buildings
  and melee cavalry (14 - 3 = 11 vs a 3/10 building) does fine.

### 8.9 Resource and unit decay

- Killed huntables (deer, boar) lose food over time once dead (decay ~0.33 food/s after a grace period) - gather
  them promptly. Sheep do not decay until killed; a killed sheep decays like a deer.
- Corpses are cosmetic; no corpse looting.

---

## 9. Unit abilities and controls

| Command | Available on | Behaviour (AoC unless noted) |
|---|---|---|
| Move (right-click ground) | all | Path to the point ignoring enemies; a moving unit does not retaliate until it arrives. |
| Attack (right-click enemy) | military, Villagers | Chase and attack that target; ranged units stop when in range. |
| Attack move (DE/HD only; `Patrol` was the AoC substitute) | military | Move toward a point, engaging any enemy met on the way; resumes moving after killing it. Monks only heal on attack-move. |
| Patrol | military, Villagers | Walk back and forth between the current position and the target (several waypoints with Shift), attacking anything seen (Aggressive). The classic way to get "attack move" behaviour in AoC. Units leave the patrol line to chase. |
| Guard | military | Follow the guarded unit/building and attack anything that attacks it or comes within range. |
| Follow | all | Follow a unit without attacking. |
| Stop | all | Halt and clear orders (unit keeps its stance behaviour). |
| Delete | all | Kill own unit/building instantly (refund nothing). |
| Garrison (right-click own building / Alt+right-click, or garrison button) | eligible units | Enter the building (section 6.5). Ungarrison button ejects all; ejected units go to the rally point. |
| Ungarrison / Ring Town Bell | TC | Town Bell: all Villagers within 23 tiles seek the nearest TC/tower/Castle with room; ring again to send them back to their previous tasks. |
| Set gather point (rally) | production buildings | Right-click a spot, resource, unit or building while the building is selected: new units walk there (Villagers start gathering a resource, units garrison into a building, ships to water). AoC: single point; DE: Shift for waypoints. |
| Stances | military (not siege in AoC; Monks none) | **Aggressive** (default): engage anything in LOS and pursue. **Defensive**: engage within range, pursue a short distance then return. **Stand ground**: never move, attack what is in range. **No attack**: never attack unless ordered. |
| Formations | groups of 2+ military | **Line** (melee front, ranged back; default), **Box** (fragile units inside), **Staggered** (spread against splash), **Flank** (two groups). Units in formation move at the speed of the slowest member; a long march (>10 tiles, DE 30) turns the line into a column. Unit order front-to-back: cavalry, infantry, archers, gunpowder/scorpions, monks/siege/trade. Villagers and fishing ships never form up. |
| Control groups | any selection | Ctrl+0-9 assigns, 0-9 selects, double-tap centres the camera. Max selection size 40 units in AoC (60 in DE). |
| Select all of type | | Double-click a unit selects all visible units of that type on screen; Ctrl+click adds; Shift+click toggles. |
| Idle villager | HUD button / `.` key | Cycles through Villagers with no task. DE also has idle-military. |
| Shift-queue | Villagers, military, buildings | Shift+right-click chains orders (e.g. build several houses, gather then build, move via waypoints); Shift+train button queues 5 units; Shift+farm places a ring of farms (DE). |
| Build | Villager | Place a foundation; several Villagers speed it up (2.9). Villagers automatically help build when ordered to the foundation. Buildings can be queued (up to 15 units/techs per building queue; 5 of a kind per Shift-click). |
| Repair | Villager | Section 2.8. |
| Convert / Heal | Monk | Right-click enemy unit converts; right-click friendly heals; Monks auto-heal nearby injured friendlies when idle. |
| Attack ground | Mangonel line, DE cannon ships | Fire at a tile instead of a unit. |
| Pack / Unpack | Trebuchet | ~5 s each; packed trebs move at 0.8 and have ram armour. |
| Drop off / Seek shelter (DE) | Villager | Return carried resources now; or garrison in the nearest building. |
| Trade | Trade Cart/Cog | Right-click a foreign Market/Dock: loops automatically. |
| Tribute | player | Diplomacy panel; needs a Market; amounts in 100s (and 500/1000). |
| Diplomacy | player | Ally / Neutral / Enemy per player, Allied Victory checkbox, Cartography shares LOS with allies. |
| Flare | player | Mark a spot on allies' minimaps. |
| Chat / taunts | player | 42 numbered taunts in AoC (DE 105). |

Automatic behaviours worth copying:

- Idle Villagers near an attacked Villager do nothing (no auto-flee); Villagers attacked by melee units stop working
  but do not fight back unless ordered. Herdables auto-follow nothing; they must be moved.
- Villagers automatically move to the next tree/bush/mine of the same type when theirs is exhausted (search radius
  of a few tiles). A farmer reseeds automatically only if the Mill queue holds a farm; otherwise he goes idle.
- Military in Aggressive stance auto-acquire targets in LOS, prioritising units over buildings (TC over other
  buildings; walls/farms/herdables never auto-targeted). Rams and Trebuchets only auto-target buildings.
- Towers/Castles/TCs auto-target the nearest enemy in range (units before buildings).
- Units retaliate against an attacker even in Defensive stance; Stand ground retaliates without moving.
- Game speed: slow 1.0x, normal 1.5x, fast 2.0x (AoC multiplayer default "normal"). Pause allowed in single player
  and in multiplayer with a limited number of pauses.

---

## 10. Simplifications Wars will make

What can be cut or flattened for a first version without changing the feel of an age-advancing RTS. Each line notes
what to keep instead.

1. **Civilisations**: ship with 3-4 civs (e.g. an archer civ, a cavalry civ, an infantry civ, a Meso civ with no
   Stable) and only the AoC single Imperial-Age unique tech. Implement bonuses as data (percent modifiers on cost,
   HP, speed, work rate, train time) rather than bespoke code; drop bonuses that need new mechanics (Huns' no-house
   start can be a modifier on population cap; Spanish gold-per-tech later).
2. **Unique units**: one per civ, no Elite tier at first (or Elite = +HP/+attack delta only). Skip Missionary,
   Longboat, Turtle Ship as second UUs.
3. **Armour classes**: keep the formula but trim the class list to ~10: Infantry, Cavalry, Camel, Archer, Siege,
   Ram, Building, Stone Defense, Ship, Monk, Spearman, Eagle. Drop War Elephant, Unique Unit, Gunpowder, Castle,
   Wall, Boar, Mameluke, Hero.
4. **Navy**: ship without water first (no Dock, Fishing Ship, Fish Trap, warships, trade cogs, transports). Add the
   Galley/Fire/Demo triangle later; Cannon Galleons never.
5. **Gunpowder**: drop Hand Cannoneer, Bombard Cannon, Bombard Tower, Cannon Galleon, Chemistry. Trebuchet remains
   the Imperial siege answer to Castles.
6. **Siege**: Ram line, Mangonel line, Trebuchet. Drop Scorpion line and Petard initially (pass-through and suicide
   mechanics).
7. **Monks**: keep conversion (it is the identity of the Monastery) but simplify to: fixed 25%/s after 4 s, guaranteed
   at 10 s, resistance as +min/+max seconds, faith recharge 60 s. Keep relics (0.5 gold/s). Drop Theocracy,
   Atonement, Heresy initially; keep Redemption (siege/buildings) because it affects balance, or defer it.
8. **Economy**: integer-friendly fixed-point rates (the sim has no floats): express gather rates as units per 100
   ticks, carry 10. Keep Villager gather rates, drop-off buildings, farms as finite nodes with reseed cost, Wheelbarrow
   / Hand Cart, the three wood and four mining techs (or collapse to one per age). Drop herdables' ownership-by-
   proximity and boar luring (make boars neutral static food or omit); keep berries, sheep (owned on sight), deer.
9. **Market**: implement the 100-lot buy/sell with a global price that moves by 2 per lot and the 30% fee; implement
   tribute with the 30% tax; **drop** Coinage/Banking/Guilds initially. Trade carts can wait until team games exist.
10. **Walls and gates**: palisade + stone wall + gate; drop Fortified Wall, Palisade Gate, diagonal gates. Towers:
    Watch Tower -> Guard Tower -> Keep as a single upgrade chain; drop Murder Holes/Heated Shot/Arrowslits (or make
    "no minimum range" the default and never have a minimum range).
11. **Garrison**: TC, towers, Castle only; arrows = 1 per Villager/archer up to the cap; healing 0.1/0.2 HP/s. Skip
    production-garrison, rams carrying infantry, transports.
12. **Projectiles**: resolve ranged attacks at launch + flight time with the hit decided at launch (accuracy roll)
    instead of simulating collisions; Ballistics becomes "ranged units no longer miss moving targets" (a flat accuracy
    modifier). Blast damage: full damage in radius, no tapering.
13. **Elevation**: either omit terrain height entirely (flat maps) or keep the single binary +25%/-25% rule. No
    cliffs in v1.
14. **Formations and stances**: Line formation only (or none: units path individually), stances Aggressive /
    Stand ground only. Keep patrol-as-attack-move, control groups, shift-queue, rally points, idle-villager cycling,
    town bell.
15. **Victory**: conquest only. AoC's rule is "all units and all buildings except walls, gates, farms, fish traps and
    outposts"; simplify to "all Town Centers, Castles and military units". Wonder/relic countdown and Regicide later.
16. **Ages**: keep all four with the same costs/times; building prerequisites can be the AoC ones (they are cheap to
    implement as "count of completed buildings of type in set").
17. **Population**: house +5, TC +5, Castle +20, cap 200 (configurable); every unit 1 pop.
18. **Techs**: keep the Blacksmith (9 techs + 3 archer), eco techs, Loom, Town Watch (LOS), Ballistics, Masonry,
    Squires/Husbandry/Bloodlines, Conscription. Drop University extras, Sappers, Hoardings, Spies, Treason,
    Cartography (always share LOS with allies), Parthian Tactics, Thumb Ring (fold into accuracy 100% by default).
19. **Civ tech-tree gaps**: express as a per-civ denylist of unit/tech ids; start with 3-5 obvious holes per civ
    (Britons no Paladin/CA, Mayans/Aztecs no Stable, Teutons no Light Cav, Turks no Pikeman, Celts no Arbalest...).
20. **DE-only content**: ignore entirely (Eagle Scout tier, Gambesons, Arson, Supplies, Devotion, regional units,
    Fire Galley/Demo Raft). The AoC roster above is the target.
21. **Numbers to keep exactly** (they define the feel): ages, Villager 50F/25 s, gather rates, farm 60W/175 food,
    house 25-30W/+5, TC 275W 100S/+5, Castle 650S, Knight 60F 75G 100 HP 10 atk 2/2, Archer 25W 45G 4 atk range 4,
    Militia 60F 20G, Spearman 35F 25W +15 vs cav, Pikeman +22, Halberdier +32, Skirmisher +3 vs archers, Mangonel
    40 attack blast, ram +125/+150/+200 vs buildings, Trebuchet 200 attack range 16, towers 5/6/7 attack range 8,
    relic 0.5 gold/s, market 100/30%/2, tribute 30%, conversion 4-10 s.

---

## 11. Sources

All pages fetched October 2026 via the MediaWiki API (wikitext) and the aoe2techtree data export; numbers were
re-tabulated, not copied.

- [S-TechTree] aoe2techtree.net game data (DE, current patch): https://aoe2techtree.net/ and
  https://aoe2techtree.net/data/data.json (unit/building/tech stats, per-civ availability)
- [S-Villager] https://ageofempires.fandom.com/wiki/Villager_(Age_of_Empires_II)
- [S-Market] https://ageofempires.fandom.com/wiki/Market_(Age_of_Empires_II)
- [S-TradeCart] https://ageofempires.fandom.com/wiki/Trade_Cart_(Age_of_Empires_II)
- [S-TradeCog] https://ageofempires.fandom.com/wiki/Trade_Cog
- [S-Trade] https://ageofempires.fandom.com/wiki/Trade
- [S-Tribute] https://ageofempires.fandom.com/wiki/Tribute
- [S-Relic] https://ageofempires.fandom.com/wiki/Relic_(Age_of_Empires_II)
- [S-Farm] https://ageofempires.fandom.com/wiki/Farm_(Age_of_Empires_II)
- [S-FishTrap] https://ageofempires.fandom.com/wiki/Fish_Trap_(Age_of_Empires_II)
- [S-FishingShip] https://ageofempires.fandom.com/wiki/Fishing_Ship_(Age_of_Empires_II)
- [S-Fish] https://ageofempires.fandom.com/wiki/Fish
- [S-DarkAge] https://ageofempires.fandom.com/wiki/Dark_Age_(Age_of_Empires_II)
- [S-FeudalAge] https://ageofempires.fandom.com/wiki/Feudal_Age_(Age_of_Empires_II)
- [S-CastleAge] https://ageofempires.fandom.com/wiki/Castle_Age_(Age_of_Empires_II)
- [S-ImperialAge] https://ageofempires.fandom.com/wiki/Imperial_Age_(Age_of_Empires_II)
- [S-Deathmatch] https://ageofempires.fandom.com/wiki/Deathmatch
- [S-Regicide] https://ageofempires.fandom.com/wiki/Regicide_(game_mode)
- [S-Population] https://ageofempires.fandom.com/wiki/Population
- [S-Wonder] https://ageofempires.fandom.com/wiki/Wonder_(Age_of_Empires_II)
- [S-Garrison] https://ageofempires.fandom.com/wiki/Garrison
- [S-Healing] https://ageofempires.fandom.com/wiki/Healing
- [S-Monk] https://ageofempires.fandom.com/wiki/Monk_(Age_of_Empires_II)
- [S-Conversion] https://ageofempires.fandom.com/wiki/Conversion
- [S-ArmorClass] https://ageofempires.fandom.com/wiki/Armor_class_(Age_of_Empires_II)
- [S-Damage] https://ageofempires.fandom.com/wiki/Attack
- [S-Elevation] https://ageofempires.fandom.com/wiki/Elevation
- [S-Ballistics] https://ageofempires.fandom.com/wiki/Ballistics_(Age_of_Empires_II)
- [S-Formation] https://ageofempires.fandom.com/wiki/Unit_formation
- [S-Stance] https://ageofempires.fandom.com/wiki/Unit_stance
- [S-RallyPoint] https://ageofempires.fandom.com/wiki/Rally_Point
- [S-Hotkey] https://ageofempires.fandom.com/wiki/Hotkey
- [S-UI] https://ageofempires.fandom.com/wiki/User_interface
- Unit pages: https://ageofempires.fandom.com/wiki/Militia_(Age_of_Empires_II), .../Man-at-Arms_(Age_of_Empires_II),
  .../Long_Swordsman_(Age_of_Empires_II), .../Two-Handed_Swordsman, .../Spearman_(Age_of_Empires_II),
  .../Pikeman_(Age_of_Empires_II), .../Halberdier_(Age_of_Empires_II), .../Eagle_Scout_(Age_of_Empires_II),
  .../Eagle_Warrior_(Age_of_Empires_II), .../Elite_Eagle_Warrior, .../Archer_(Age_of_Empires_II),
  .../Crossbowman_(Age_of_Empires_II), .../Arbalester, .../Skirmisher_(Age_of_Empires_II),
  .../Elite_Skirmisher_(Age_of_Empires_II), .../Cavalry_Archer_(Age_of_Empires_II),
  .../Heavy_Cavalry_Archer_(Age_of_Empires_II), .../Hand_Cannoneer, .../Scout_Cavalry_(Age_of_Empires_II),
  .../Light_Cavalry_(Age_of_Empires_II), .../Hussar_(Age_of_Empires_II), .../Knight_(Age_of_Empires_II),
  .../Cavalier_(Age_of_Empires_II), .../Camel_Rider_(Age_of_Empires_II), .../Heavy_Camel_Rider,
  .../Battering_Ram_(Age_of_Empires_II), .../Mangonel_(Age_of_Empires_II), .../Onager_(Age_of_Empires_II),
  .../Scorpion_(Age_of_Empires_II), .../Petard_(Age_of_Empires_II), .../Transport_Ship_(Age_of_Empires_II),
  .../Galley_(Age_of_Empires_II), .../War_Galley_(Age_of_Empires_II), .../Galleon_(Age_of_Empires_II),
  .../Fire_Galley_(Age_of_Empires_II), .../Fire_Ship_(Age_of_Empires_II), .../Fast_Fire_Ship,
  .../Demolition_Raft, .../Demolition_Ship, .../Heavy_Demolition_Ship, .../Cannon_Galleon,
  .../Elite_Cannon_Galleon, .../Longboat_(Age_of_Empires_II), .../Turtle_Ship, .../King_(Age_of_Empires_II)
- Unique unit pages: .../Longbowman_(Age_of_Empires_II), .../Cataphract_(Age_of_Empires_II), .../Woad_Raider,
  .../Chu_Ko_Nu_(Age_of_Empires_II), .../Throwing_Axeman_(Age_of_Empires_II), .../Huskarl_(Age_of_Empires_II),
  .../Samurai_(Age_of_Empires_II), .../Mangudai_(Age_of_Empires_II), .../War_Elephant_(Age_of_Empires_II),
  .../Mameluke_(Age_of_Empires_II), .../Teutonic_Knight_(Age_of_Empires_II), .../Janissary_(Age_of_Empires_II),
  .../Berserk_(Age_of_Empires_II), .../Jaguar_Warrior, .../Tarkan, .../War_Wagon_(Age_of_Empires_II),
  .../Plumed_Archer, .../Conquistador_(Age_of_Empires_II), .../Missionary_(Age_of_Empires_II)
- Building pages: .../House_(Age_of_Empires_II), .../Mill_(Age_of_Empires_II), .../Lumber_Camp_(Age_of_Empires_II),
  .../Mining_Camp_(Age_of_Empires_II), .../Dock_(Age_of_Empires_II), .../Barracks_(Age_of_Empires_II),
  .../Archery_Range_(Age_of_Empires_II), .../Stable_(Age_of_Empires_II), .../Siege_Workshop_(Age_of_Empires_II),
  .../Blacksmith_(Age_of_Empires_II), .../Monastery_(Age_of_Empires_II), .../University_(Age_of_Empires_II),
  .../Town_Center_(Age_of_Empires_II), .../Castle_(Age_of_Empires_II), .../Outpost_(Age_of_Empires_II),
  .../Watch_Tower_(Age_of_Empires_II), .../Guard_Tower_(Age_of_Empires_II), .../Keep_(Age_of_Empires_II),
  .../Bombard_Tower, .../Palisade_Wall_(Age_of_Empires_II), .../Palisade_Gate_(Age_of_Empires_II),
  .../Stone_Wall_(Age_of_Empires_II), .../Fortified_Wall_(Age_of_Empires_II), .../Gate_(Age_of_Empires_II)
- Civilisation pages: https://ageofempires.fandom.com/wiki/Britons, .../Byzantines_(Age_of_Empires_II), .../Celts,
  .../Chinese_(Age_of_Empires_II), .../Franks, .../Goths, .../Japanese_(Age_of_Empires_II),
  .../Mongols_(Age_of_Empires_II), .../Persians_(Age_of_Empires_II), .../Saracens, .../Teutons, .../Turks,
  .../Vikings_(Age_of_Empires_II), .../Aztecs_(Age_of_Empires_II), .../Huns, .../Koreans, .../Mayans,
  .../Spanish_(Age_of_Empires_II)
- General background: https://en.wikipedia.org/wiki/Age_of_Empires_II and
  https://en.wikipedia.org/wiki/Age_of_Empires_II:_The_Conquerors ; https://www.ageofempires.com/games/aoeiide/
