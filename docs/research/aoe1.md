# Age of Empires (1997) — research notes for Wars

> Scope: *Age of Empires* (Ensemble Studios / Microsoft, 1997), its expansion *The Rise of Rome* (1998),
> the 2018 *Definitive Edition* where it matters, and a few notes on *Return of Rome* (2023, the AoE1
> mode inside AoE2:DE) where it clarifies how a rule evolved.
>
> Conventions used below:
> - **Orig** = the 1997 game + *Rise of Rome* (patch 1.0c / 1.0a), i.e. the thing people are nostalgic about.
> - **DE** = *Definitive Edition* (2018) after its balance updates.
> - Costs are written as `F` food, `W` wood, `G` gold, `S` stone. Times are seconds of game time at 1.0× speed.
> - Where Orig and DE values differ, the Orig value is given first and the DE value in parentheses, unless stated otherwise.
> - Numbers were cross-checked between the Age of Empires fandom wiki, Wikipedia, and the old fan
>   reference "Telcontar's Empire of the Ages" (artho.com). Any value I could not confirm is marked *(unverified)*.
>
> Nothing in this file is copied verbatim from any source; see **Sources** at the end.

---

## Table of contents

1. [History, studio, release and reception](#1-history-studio-release-and-reception)
2. [The four ages](#2-the-four-ages)
3. [Resources, villagers, economy, population](#3-resources-villagers-economy-population)
4. [Civilizations (12 + 4)](#4-civilizations)
5. [Unit roster and building roster](#5-unit-roster-and-building-roster)
6. [Technologies by building](#6-technologies-by-building)
7. [Combat model](#7-combat-model)
8. [Random maps, scenario editor, multiplayer](#8-random-maps-scenario-editor-multiplayer)
9. [What the Definitive Edition changed](#9-what-the-definitive-edition-changed)
10. [What Wars should borrow from AoE1](#10-what-wars-should-borrow-from-aoe1)
11. [Sources](#11-sources)

---

## 1. History, studio, release and reception

### 1.1 The studio and the people

- **Ensemble Studios** (Dallas, Texas) was founded by Tony Goodman, Rick Goodman and John Boog-Scott in 1995; *Age of Empires* was the studio's very first game.
- **Bruce Shelley**, co-designer of Sid Meier's *Civilization* and *Railroad Tycoon*, joined as lead designer. The "advance through ages / tech tree" DNA comes straight from *Civilization*; the real-time control comes from *Warcraft II* and *Command & Conquer*, which were the RTS benchmarks of 1995–96.
- Other key names: Rick Goodman and Brian Sullivan (design), Tony Goodman (art direction), Angelo Laudon (lead programmer), Matt Pritchard (graphics engine and optimisation), Dave Pottinger (AI), Mark Terrano and Paul Bettner (networking — authors of the famous GDC paper "1500 Archers on a 28.8"), Sandy Petersen (scenario design; later lead designer of *The Rise of Rome*), Stephen and David Rippy (music).
- Team size was roughly 25 people over about two and a half years. The working title was **"Dawn of Man"**; it was first shown at E3 in June 1996.
- The engine is the 2D sprite-based **Genie Engine**, later reused for *Age of Empires II*. Most of the game was C++, but the sprite blitting core was around 13,000 lines of hand-written 32-bit x86 assembly, which Ensemble credited for the game's smooth drawing of hundreds of units on a Pentium 90.
- The three East Asian civilizations (Shang, Choson, Yamato) were added halfway through development after Microsoft decided the game would be marketed in Asia.

### 1.2 Releases

| Product | Date | Notes |
|---|---|---|
| *Age of Empires* | 13–15 October 1997 (North America; sources disagree by two days), 23 October 1997 (UK) | Windows 95/NT, later Mac OS. Published by Microsoft. |
| Patches 1.0a, 1.0b, 1.0c | 1997–1999 | 1.0c (17 Dec 1999) shipped together with RoR 1.0a: idle-villager key (`.`), Shang villagers 35 → 40 food, Phoenician woodcutting 30% → 15%, anti-trainer sync checks, cheat removal in multiplayer. |
| *The Rise of Rome* (expansion) | 22 October 1998 (NA), 6 November 1998 (Europe) | Lead designer Sandy Petersen. Built partly to buy time for the delayed *Age of Kings*. |
| *Gold Edition* | 24 August 1999 | Base game + RoR. |
| *Collector's Edition* | November 2000 / 2001 | AoE + RoR + AoK + Conquerors. |
| *Pocket PC Edition* | 2003 | Floodgate Entertainment, single-player only, 320×240. |
| *Definitive Edition* | 20 February 2018 (Microsoft Store), 19 August 2019 (Steam, cross-play) | Forgotten Empires + Tantalus Media, published by Microsoft Studios. Announced 12 June 2017 at E3 for a 19 October 2017 launch that slipped. |
| *Return of Rome* | 16 May 2023 | AoE1 rebuilt as a game mode inside *AoE II: Definitive Edition*; adds the Lac Viet civilization. |

### 1.3 Sales and reception

- Debuted at #7 in the US PC charts for October 1997; more than 178,000 US copies and over US$8 M by the end of 1997, the best-selling US RTS of that period.
- 650,000 shipped worldwide by 12 December 1997; 850,000 sell-through by February 1998; about **3 million copies and roughly US$120 M** by 2000. Platinum (200,000) in Germany by November 1998; ~150,000 in South Korea by 2000.
- *The Rise of Rome*: an estimated 0.8–1.2 million copies by the early 2000s, Computer Games Strategy Plus "Add-On of the Year 1998".
- Critical scores (original): Metacritic 83/100, GameRankings 87 %, Computer Gaming World 4.5/5, PC Zone 9.4/10, CVG 9/10, Next Generation 4/5; the outliers were GameSpot (6.8) and IGN (7.0), both complaining about micromanagement (no build queues, dumb unit AI) and the 50-unit population cap. Elliott Chin (CGW) called the pop cap the game's "most serious grievance".
- Awards: Gamecenter Game of the Year 1997; tied with *StarCraft* for PC Strategy Game of the Year at the first Interactive Achievement Awards (1998); runner-up for CGW's Strategy Game of the Year.
- RoR: GameSpot 7.4, CGW 4/5, PC PowerPlay 90 %.
- DE: Metacritic 69; PC Gamer 60/100, GameSpot 6/10 — "a faithful remaster of a game whose problems are still there".

### 1.4 Why it is iconic — the nostalgia inventory

**Sound and voice**
- Units speak a **made-up language** (Ensemble had no budget for real dead languages). Decipherable words include *"Rogan?"*, *"Aikhe"*, *"Erectus"*, *"Homus"*, *"Zokanta"*, *"Yair winnie"*; before *Return of Rome* the lines were random and had no relation to the action ordered.
- **"Wololo"** — the Priest's conversion chant, sampled from the Spectrasonics "Heart of Africa vol. 1" sample library. It became an internet meme and is retroactively attached to AoE2's Monks, who never actually say it.
- Some ambient sound effects were lifted from the *Microsoft Plus! for Kids* "Bugs", "Horses" and "Underwater" desktop themes.
- The soundtrack by Stephen and David Rippy mixes real period instruments with samples; the main menu theme and the Rise of Rome menu theme are instantly recognisable.
- The **villager "whoop"** on selection, the elephant trumpet, the catapult "thunk", the Priest's chant and the "unit lost under attack" horn are the sounds people remember.

**Taunts** (multiplayer chat: type the number and press Enter — 25 of them)

| # | Line | # | Line |
|---|---|---|---|
| 1 | Yes. | 14 | Start the game already! |
| 2 | No. | 15 | Who's the man? |
| 3 | I need food. | 16 | Attack them now! |
| 4 | Somebody pass the wood. | 17 | *(low laugh)* |
| 5 | Gold please. | 18 | I am weak, please don't kill me! |
| 6 | Gimme some stone. | 19 | *(high-pitched laugh)* |
| 7 | *(whimper)* | 20 | I just got some… satisfaction! |
| 8 | Your attempts are futile. | 21 | Hey, nice town! |
| 9 | *(group cheer)* | 22 | We will NOT tolerate this behavior. |
| 10 | Hey, I'm in your town. | 23 | Get out! |
| 11 | *(group groan)* | 24 | Dad gum! |
| 12 | Join me! | 25 | Aw, yeah! |
| 13 | I don't think so. | | |

Taunt 14 ("Start the game already!") and 10 ("Hey, I'm in your town") are the ones everybody quotes. AoE2 inherited the design and some of the lines ("Long time, no siege").

**Cheat codes** (type in chat, not case-sensitive; cheat units spawn at the newest Town Center and cannot spawn while the TC is researching an age)

| Code | Effect |
|---|---|
| `PEPPERONI PIZZA` | +1,000 food |
| `WOODSTOCK` | +1,000 wood |
| `COINAGE` | +1,000 gold |
| `QUARRY` | +1,000 stone |
| `STEROIDS` | Instant build / research / gather / train for **all** players (toggle) |
| `REVEAL MAP` / `NO FOG` | Map reveal / disable fog of war |
| `BIGDADDY` | "Winsett's Z": a white sports car with a rocket launcher (600 damage every 2 s) |
| `PHOTON MAN` | Laser Trooper (futuristic robot with rapid-fire laser) |
| `E=MC2 TROOPER` | Nuke Trooper, 88-tile nuke launcher |
| `BIG BERTHA` | Stone Throwers / Catapults / Heavy Catapults become Big Berthas (more HP, range, blast) |
| `ICBM` | Ballistae and Helepoleis get 100 range |
| `HOYOHOYO` | Priests get huge speed and 600 HP |
| `DARK RAIN` | Foot archers become Stealth Archers (turn into trees when idle, walk on water) |
| `BLACK RIDER` | Horse Archers become Black Riders; on death they become a catapult |
| `MEDUSA` | Villagers become Medusas; on death they respawn as Black Riders |
| `FLYING DUTCHMAN` | Catapult Triremes / Juggernauts can sail on land |
| `GAIA` | Control all animals (and lose your civ) |
| `JACK BE NIMBLE` | Catapults throw cows and caped villagers |
| `DIEDIEDIE` | Kill all enemies (single-player) |
| `HARI KARI` / `RESIGN` / `HOME RUN` / `KILLn` | Suicide / resign / instant win / kill player *n* |
| RoR additions: `BIG MOMMA` (faster-firing car), `CONVERT THIS!` (Saint Francis, lightning every 3 s), `STORMBILLY` (Zug 209 mech), `POW` (baby on a tricycle with a shotgun), `GRANTLINKSPENCE` (animals become "King" animals — the names of Sandy Petersen's three sons), `UPSIDFLINTMOBILE` (Chariot Archers very fast, fire 3× faster), `KING ARTHUR` (birds become dragons) |

Patch 1.0c removed `GAIA`, `REVEAL MAP`, `NO FOG` and `STEROIDS` from multiplayer, and DE does **not** disable achievements when cheats are used.

**Campaigns**

| Game | Campaign | Civ | Scenarios | What it covers |
|---|---|---|---|---|
| AoE | Ascent of Egypt (learning campaign) | Egyptians | 12 | Hunting → farming → Tool/Bronze → wonders (the first scenario, "Hunting", starts with 140F/80W and a herd of gazelles) |
| AoE | Glory of Greece | Greeks | 8 | Mycenae, Troy, Marathon, Alexander |
| AoE | Voices of Babylon | Babylonians | 8 | Hammurabi to Nebuchadnezzar |
| AoE | Yamato, Empire of the Rising Sun | Yamato | 8 | Early Japan, Kyushu, Korea |
| AoE demo | Reign of the Hittites | Hittites | 3 | Demo-only; included again in DE |
| RoR | Rise of Rome | Romans | 6 | Early Rome, Pyrrhus |
| RoR | Ave Caesar | Romans | 4 | Caesar's campaigns |
| RoR | Pax Romana (DE: "Imperium Romanum") | Romans / Palmyrans | 5 | Actium, Zenobia |
| RoR | Enemies of Rome | Carthage, Macedon, Palmyra, Rome | 4 | Hannibal, Spartacus etc. |
| RoR demo | The First Punic War | Carthaginians | 3 | Demo-only; included in DE |

Every scenario opens with a history text, hints and the objectives; most campaign scenarios were built in the shipping Scenario Builder with hand-placed units because the editor had no triggers (see §8).

**Other things people remember**
- Walls are tiny 1×1 stone segments that cost 5 stone each, and there are **no gates** — you leave a gap, plug it with a House, or delete a wall segment when you need to get out.
- Farms are 3×3 buildings that run out and have to be rebuilt by hand; in Orig units cannot walk over them.
- Villagers idle constantly and the only idle-villager tool is the `.` key added in 1.0c.
- The Town Center has no attack. A single Clubman in your town in the Stone Age is a crisis.
- The pop cap of **50** in the original; `Rise of Rome` allowed up to 200 in multiplayer.
- Random map "Death Match" (huge starting stockpiles) and the "Highland" / "Small Islands" map types.
- The Wonder: a 5×5 monument costing 1,000 of wood, gold and stone, winning the game if it survives 2,000 game years (≈10 real minutes at normal speed).
- The score screen with its timeline chart, and the achievements screen "Military / Economy / Religion / Technology / Survival / Wonder".

---

## 2. The four ages

Ages are researched at the Town Center. While the TC is researching you cannot train villagers from it (other TCs, if any, still can). Every age advance needs **two buildings of the previous age** (any two different ones from the list), plus food (and gold for Iron).

| Age | Cost | Research time | Buildings required (any two of) | Historical frame (manual) |
|---|---|---|---|---|
| **Stone Age** | — (start) | — | — | Palaeolithic hunter-gatherers |
| **Tool Age** | 500 F | 120 s | Barracks, Dock, Granary, Storage Pit | Neolithic: farming, stable food, first militias |
| **Bronze Age** | 800 F | 140 s | Archery Range, Market, Stable | Metals, trade, religion, first empires |
| **Iron Age** | 1,000 F + 800 G | 160 s | Academy, Government Center, Siege Workshop, Temple | Iron weapons, siege engines, wonders of the ancient world |

Civ modifier: Greeks (DE) advance 10 % faster from the Tool Age on (their TC works 10 % faster).

### 2.1 What each age unlocks

**Stone Age (start)**
- Buildings: Town Center (one only; can be rebuilt if lost), House, Barracks, Granary, Storage Pit, Dock.
- Units: Villager (TC), Clubman (Barracks), Fishing Boat and Trade Boat (Dock).
- No technologies at all. Strategy is purely "find food, make villagers, build two buildings, click Tool Age".

**Tool Age**
- Buildings: Archery Range, Stable, Market, Farm, Small Wall, Watch Tower (walls and towers need a Granary tech first).
- Units: Axeman (after Battle Axe), Slinger (RoR), Bowman, Scout, Light Transport, Scout Ship.
- Techs: Granary → Small Wall, Watch Tower. Storage Pit → Toolworking, Leather Armor ×3. Market → Woodworking, Stone Mining, Gold Mining, Domestication.
- First real raids: the "Tool rush" with Axemen/Slingers/Bowmen/Scouts is a staple of competitive AoE1.

**Bronze Age**
- Buildings: Academy, Siege Workshop, Temple, Government Center, additional Town Centers (after a Government Center), Medium Wall, Sentry Tower.
- Units: Short/Broad Swordsman, Improved/Composite Bowman, Chariot Archer (needs Wheel), Chariot (Wheel), Cavalry, Camel Rider (RoR), Hoplite, Stone Thrower, Priest, Fishing Ship, Merchant Ship, War Galley.
- Techs: Storage Pit → Metalworking, Scale Armor ×3, Bronze Shield. Market → Artisanship, Plow, Wheel. Government Center → Nobility, Writing, Architecture, Logistics (RoR). Temple → Astrology, Mysticism, Polytheism. Granary → Medium Wall, Sentry Tower.
- This is the age where most games are decided: Hoplites, Stone Throwers, Cavalry, Chariot Archers and Priests all arrive here.

**Iron Age**
- Buildings: Fortified Wall, Guard Tower, Ballista Tower (needs Ballistics), Wonder.
- Units: Long Swordsman, Legion, Horse Archer, Heavy Horse Archer, Elephant Archer, Heavy Cavalry, Cataphract, Scythe Chariot (RoR), War Elephant, Armored Elephant (RoR), Catapult, Heavy Catapult, Ballista, Helepolis, Phalanx, Centurion, Heavy Transport, Trireme, Catapult Trireme, Juggernaut, Fire Galley (RoR).
- Techs: Storage Pit → Metallurgy, Chain Mail ×3, Iron Shield, Tower Shield (RoR). Market → Craftsmanship, Siegecraft, Coinage, Irrigation. Government Center → Aristocracy, Ballistics, Alchemy, Engineering. Temple → Afterlife, Monotheism, Fanaticism, Jihad/Zealotry, Medicine (RoR), Martyrdom (RoR). Granary → Fortified Wall, Guard Tower, Ballista Tower.
- Several top-tier unit upgrades (Legion, Cataphract, Heavy Horse Archer, Scythe Chariot, Armored Elephant, Heavy Catapult, Helepolis, Juggernaut, Ballista Tower) cost 1,000–2,600 resources each, nearly as much as a Wonder, which forces players to specialise.

### 2.2 Starting-age options
Random map games can start in any age ("Starting Age": Stone, Tool, Bronze, Iron, plus "Post-Iron" in DE). Starting in Bronze automatically unlocks Market/Farm/Government Center/Temple/Watch Tower/Small Wall and all military buildings; starting in Iron also unlocks extra Town Centers and upgraded walls/towers.

---

## 3. Resources, villagers, economy, population

### 3.1 The four resources and where they come from

| Resource | Sources | Amount per node (Orig → DE) | Drop-off buildings | Used for |
|---|---|---|---|---|
| **Food** | Berry bush (forage), hunting, shore fishing, Fishing Boats, Farms | Berry bush 150; Gazelle ~150; Elephant ~300; Lion/Alligator 100 (decays 1/s); Shore fish 250; Salmon/Tuna 250; Whale 250 (300 DE); Farm 250 base | Forage/farm food → Granary or TC; hunt/fish food → Storage Pit or TC; boats → Dock | Villagers, most infantry/cavalry, age advances, almost every tech |
| **Wood** | Trees (straggler 75, forest tree 40; 25 HP each) | finite — trees never regrow | Storage Pit, TC | All buildings except walls/towers/Wonder (which also need wood), ships, siege, archers, chariots, market techs |
| **Gold** | Gold mines (400 → 450 → 500 per mine tile), sea trade | finite | Storage Pit, TC | Everything from the Bronze Age on, Priests (gold only), Iron Age (800), Wonder (1,000) |
| **Stone** | Stone mines (250 → 300 → 350 per tile) | finite | Storage Pit, TC | Walls (5), towers (150), Slingers (10), Wonder (1,000), wall/tower techs |

Nothing regrows. Trees do not reseed, mines do not refill, fish do not respawn. The only renewable sources are Farms (which cost wood) and sea trade (which costs other resources in Orig). This is the biggest economic difference from AoE2, and it makes long games into a scramble for the last gold and stone piles.

Animal carcasses **decay**: gazelle −1 food every 4 s, predators −1 food per second, elephants slowest, so you send several villagers to a kill at once. Elephants fight back (a lone villager will lose), and the classic trick is to shoot an elephant, run, and lure it next to your Town Center.

### 3.2 Default starting conditions

- 1 Town Center + 3 Villagers (campaigns vary; "Hunting" starts with 1 villager and 140F/80W).
- Standard starting stockpile: **200 food, 200 wood, 100 gold, 150 stone** *(unverified; from memory of the original settings screen — the DE options are Default / Low / Medium / High)*.
- Death Match (RoR): 20,000 F, 20,000 W, 10,000 G, 5,000 S *(the AoE2 numbers; AoE1's Death Match used the same order of magnitude — unverified)*.
- Shang start with −40 food (patch 1.0c), Palmyrans with +75 food (DE), Carthaginians with +50 of everything (DE).

### 3.3 Villagers

| Stat | Value |
|---|---|
| Cost / time | 50 F, 20 s (Shang 35 F orig → 40 F in 1.0c; Palmyrans 75 F) |
| HP | 25 (Sumerians 40; Jihad +40) |
| Attack | 3 melee; hunters throw spears for 4 at range 4 |
| Armor | 0/0 (Palmyran 1/1) |
| Speed | 1.1 tiles/s (Orig; 0.92 in RoR) — Wheel +0.55 (≈ +50 %), Jihad +0.3, Tool Age +10 % (DE) |
| LOS | 4 |
| Carry capacity | 10 of any resource; +3 per mining tech, +2 per woodworking tech; Jihad −8 (−6 for food) |

Villagers are renamed by task: Builder, Farmer, Fisherman, Forager, Woodcutter, Miner, Hunter, Repairman. Switching to a different resource type throws away whatever they are carrying (Orig: instantly on the order; DE: only food→food switches keep the load). Only villagers can build and repair (buildings and ships).

**Gather rates** (resources per game second, DE values; Orig differed slightly)

| Task | Base rate | Notes |
|---|---|---|
| Forage (berries) | 0.45 | Palmyran 0.5625 |
| Hunt | 0.45 (0.4725 DE) | Persian +30 % (orig stated +30 %, actual +67 % before DE) and +3 carry |
| Shore fishing (villager) | 0.60 | fastest food in the game; shore fish are 1×1 so few villagers fit |
| Fishing Boat | 0.40 | slowest, but unlimited parking space on 2×2 fish |
| Farm | 0.45 | Minoan +10 % (RoR) |
| Wood | 0.55 → 0.95 fully upgraded (Orig: → 1.15) | Phoenician +30 %/+15 % (1.0c), Palmyran +25 % |
| Stone | 0.45 → 1.05 (Orig) / 0.5175 → 0.87 (DE) | Babylonian +30 % (orig) / +20 % (DE) |
| Gold | 0.45 → 0.94 (Orig) / 0.5175 → 0.84 (DE) | Egyptian +20 % (orig) and +2 carry |

The effective rate is dominated by walking distance to the drop-off, which is why you build Storage Pits at the wood line and the gold pile and why Wheel (villager speed) is the most important economic tech in the game.

### 3.4 Drop-off buildings

| Building | Cost | HP | Build | Accepts | Also does |
|---|---|---|---|---|---|
| Town Center | 200 W | 600 | 60 s | everything | trains villagers, ages up, +4 pop |
| Granary | 120 W | 350 | 30 s | berries, farm food | researches walls and towers; prerequisite for the Market |
| Storage Pit | 120 W | 350 | 30 s | wood, stone, gold, hunted/fished food | researches attack and armor techs |
| Dock | 100 W | 350 | 50 s | Fishing Boat food (villagers cannot use it in Orig) | builds ships, trade target |

The Granary/Storage Pit split (berries and farms vs hunting and fishing) is a well-known AoE1 quirk; AoE2 merged them into the Mill and Lumber/Mining Camps.

### 3.5 Farms

- Tool Age, needs a Market. 75 W, 50 HP, 30 s build (24 s DE), 3×3 tiles, **250 food** base.
- Domestication +75 (325), Plow +75 orig / +100 DE (425), Irrigation +75 orig / +125 DE (550). Sumerian farms +250 (DE +125), Minoan +25 % (orig), Persian −30 % (orig).
- Only one villager gathers at a time in practice (a second one waits until the first leaves to drop off).
- Orig: farms block movement (they have little huts), cannot be auto-reseeded, and must be rebuilt manually when exhausted. DE made them walkable, let you right-click an exhausted farm to rebuild it, and later added an auto-reseed toggle at the Granary.

### 3.6 Fishing, hunting, trade

- Fishing Boat 50 W, 45 HP, speed 1.45; Fishing Ship (Bronze upgrade 50F/100W) 75 HP, speed 2.1. Both gather 0.4/s and drop at the Dock.
- Sea trade (Orig/DE): a Trade Boat (100 W, 200 HP, speed 2.1) or Merchant Ship (upgrade 200F/75W; 250 HP, speed 2.5) sails to **another player's Dock** (any diplomatic stance, even a defeated player), spends 20 of a resource you choose (food/wood/stone — it is simply destroyed, not given to the other player), picks up "trade goods" and converts them into gold back at your own Dock. Gold scales with straight-line distance between the two closest Docks. The target Dock restocks 20 trade goods every 20 s (1/s, max 100). Yield is 0.1–0.2 gold/s on short routes; only a map-spanning route matches an unupgraded gold miner (~0.5/s). Palmyran trade yields double. There is no land trade in Orig/DE (Trade Carts arrive with *Return of Rome*).
- There is no resource market/exchange in Orig or DE. The only resource transfer between players is **tribute** at the Market, with a tax that Coinage removes (Palmyrans never pay it).

### 3.7 Population and houses

- Town Center = 4 pop, each House = 4 pop (30 W, 75 HP, 20 s build, 2×2). No limit on number of houses.
- **Hard cap: 50 units** in the original. Patch 1.0a / RoR let the host set 25–200 in multiplayer (single-player stayed at 50 until DE). DE: 25–250 in steps of 25 (campaigns still 50). RoR mode: up to 500.
- Every unit costs 1 pop, including boats. Logistics (RoR, Government Center) halves the pop cost of Barracks units. Hittite towers (+4) and Shang TCs (+4) are team bonuses in RoR only.
- Orig has no build queue, so you cannot "over-queue" past the cap; RoR added queuing and lets queued units wait for housing.
- You can exceed the cap by converting units with Priests or by training from several buildings simultaneously at the last slot.
- Houses are the cheapest building and are routinely used as walls in the Stone Age (they survive 38 s of Clubman hits).

---

## 4. Civilizations

### 4.1 Architecture sets

| Set | Civilizations |
|---|---|
| Greek | Greeks, Minoans, Phoenicians |
| Egyptian | Egyptians, Assyrians, Sumerians |
| Mesopotamian ("Babylonian") | Babylonians, Hittites, Persians |
| East Asian | Shang, Choson, Yamato |
| Roman (RoR) | Romans, Carthaginians, Macedonians, Palmyrans |

Every civ shares the same unit sprites and the same Stone Age buildings; only the Tool→Iron building graphics differ per set. No civ has unique units — the differences are bonuses and tech-tree holes, which is a leaner model than AoE2's unique units.

### 4.2 Bonus table (Orig values, with DE changes in brackets)

| Civ | Focus | Bonuses |
|---|---|---|
| **Assyrians** | archers, Tool rush | Villagers +30 % speed as stated (really ≈18 %; DE 10 %). Foot/mounted archers fire 40 % faster (DE: 11/22/33 % in Tool/Bronze/Iron). |
| **Babylonians** | defence, Priests | Stone mining +30 % and +3 carry (DE +20 %, +2). Walls and towers +100 % HP (DE +60 %). Priests regain faith 30 % faster (DE 30 %). [DE: Market techs −30 %.] |
| **Choson** | infantry, towers | Long Swordsman and Legion +80 HP (DE: whole sword line +5/+15/+20/+60/+80). Towers +2 range. Priests −30 % cost (really −32 %). [DE: Storage Pit techs −40 %.] |
| **Egyptians** | chariots, Priests | Gold mining +20 % (+44 % actual before DE) and +2 carry. Chariot and Chariot Archer +33 % HP. Priests +3 range (DE +2 Bronze / +3 Iron). [DE: Farms −20 %.] |
| **Greeks** | Academy, navy | Hoplite line +30 % speed. War ships +30 % stated (≈17 % actual; DE 20 %). [DE: Academy units −20 % cost, TC 10 % faster from Tool Age.] |
| **Hittites** | siege, archers | Stone Thrower line +100 % HP (DE +50 %). All archers +1 attack. War ships +4 range (DE +1/+2/+3 by age). [RoR: Wheel −50 %.] |
| **Minoans** | archers, navy | Ships −30 % (DE −15/−20/−25/−30 by age). Composite Bowman +2 range (DE: Improved line +1/+2). Farms +25 % food (orig). |
| **Persians** | elephants, navy | Hunting +30 % stated (+67 % actual before DE) and +3 carry. Farms −30 % food (orig). Elephants +50 % speed stated (≈56 % actual; DE 25 %). Trireme line fires 50 % faster (DE 18/25/33 % by age). [RoR: walls −20 %.] |
| **Phoenicians** | elephants, navy | Elephant units −25 % cost. Catapult Trireme / Juggernaut fire 65 % faster (72 % actual; DE 33 %). Woodcutting +30 % (1.0c: 15 %) and +3 carry (DE +2). |
| **Shang** | economy, cavalry | Villagers −30 % (35 F; 1.0c: 40 F, −20 %, and −40 starting food). Walls +100 % HP (DE +60 %). [DE: cavalry attacks 10 % faster.] |
| **Sumerians** | economy, siege | Villagers +15 HP. Stone Thrower line fires 50 % faster (DE 45 %). Farms +250 food (DE +125). [DE: camels +1 pierce armor.] |
| **Yamato** | cavalry, navy | Mounted units −30 % (RoR −25 %, DE −15 %). Villagers +30 % speed (orig). Ships +30 % HP (DE +10/15/20/25 by age). [RoR: Fishing Boats 20 % faster, Stable/Range upgrades −30 %.] |
| **Romans** (RoR) | infantry, buildings | Buildings −15 % (DE −10 %; not towers/walls/Wonder). Towers −50 % (DE −40 %). Swordsmen attack 33 %/50 % faster. |
| **Carthaginians** (RoR) | elephants, Academy, navy | Academy and elephant units +25 % HP. Fire Galleys +25 % attack. Transports +30 % speed (DE 25 %). [DE: +50 of each starting resource; camels +15 % HP.] |
| **Macedonians** (RoR) | Academy, siege, anti-Priest | Non-ranged units +2 LOS. Academy units +2 pierce armor (DE +1 Bronze / +2 Iron). Siege −50 % (DE −25 %). All units and buildings 4× harder to convert. No Temple at all. |
| **Palmyrans** (RoR) | boom, trade, camels | Villagers cost 75 F but have 1/1 armor and work 25 % faster (buggy before DE: +44 % on some tasks, 0 on others). Camel Riders +25 % speed. Tribute is untaxed. Trade ships return double gold. [DE: +75 starting food.] |

(*Return of Rome* later added team bonuses to every civ — e.g. Babylonian builders +10 %, Choson buildings +2 LOS, Hittite towers +4 pop, Shang TCs +4 pop, Sumerian TCs −25 %, Yamato military buildings −33 % wood, Palmyran research +30 % faster, Macedonian houses +50 HP, Roman Priests heal +50 %, Carthaginian Academies 20 % faster. Those did not exist in Orig/DE.)

### 4.3 Tech-tree holes per civilization

Each civ is denied a specific set of units and techs. This is where AoE1's asymmetry really lives: e.g. the Persians have no Academy at all, the Macedonians no Temple, the Hittites no Temple techs beyond Astrology, the Yamato and Minoans no Guard Tower or Fortified Wall.

| Civ | Missing units | Missing technologies |
|---|---|---|
| Assyrians | Slinger, Improved/Composite Bowman, Heavy Horse Archer, Elephant Archer, Scythe Chariot, War/Armored Elephant, Phalanx/Centurion, Catapult Trireme/Juggernaut | Bronze/Iron/Tower Shield, Nobility, Architecture, Aristocracy |
| Babylonians | Heavy Horse Archer, Elephant Archer, Heavy Cavalry/Cataphract, War Elephant, Phalanx/Centurion, Ballista/Helepolis, Trireme, Catapult Trireme | Iron Shield, Tower Shield |
| Choson | Composite Bowman, Heavy Horse Archer, Elephant Archer, Chariot, Chariot Archer, War Elephant, Camel Rider, Phalanx/Centurion, Catapult/Heavy Catapult, Fire Galley, Catapult Trireme (Orig also Heavy Transport) | Chain Mail ×3, Iron/Tower Shield, Aristocracy, Alchemy, Engineering (Orig also Nobility) |
| Egyptians | Broad/Long Swordsman/Legion, Horse Archer, Cavalry line, Armored Elephant, Catapult/Heavy Catapult, Ballista, Phalanx/Centurion, Fire Galley | Bronze/Iron/Tower Shield, Siegecraft (Orig also Coinage) |
| Greeks | Broad Swordsman line, Improved/Composite Bowman, Chariot Archer, Horse Archer, Elephant Archer, Chariot, Cataphract, War Elephant, Camel Rider | Metallurgy, Monotheism, Zealotry |
| Hittites | Slinger, Long Swordsman/Legion, Improved/Composite Bowman, Heavy Cavalry/Cataphract, Ballista/Helepolis, Centurion, Fishing Ship, Trireme, Catapult Trireme | Irrigation, Architecture, Mysticism, Polytheism, Afterlife, Monotheism, Fanaticism, Zealotry, Medicine |
| Minoans | Legion, Chariot Archer, Horse Archer, Elephant Archer, Chariot, Heavy Cavalry/Cataphract, War Elephant, Fire Galley | Fortified Wall, Guard/Ballista Tower, Astrology, Mysticism, Afterlife, Monotheism, Fanaticism, Zealotry |
| Persians | **Academy (no Hoplite line)**, Chariot Archer, Chariot, Heavy Catapult, Ballista/Helepolis, Fire Galley | Ballista Tower, Craftsmanship, Siegecraft, Irrigation, Aristocracy |
| Phoenicians | Horse Archer line, Heavy Cavalry/Cataphract, Catapult/Heavy Catapult, Ballista/Helepolis, Fire Galley | Metallurgy, Chain Mail ×3, Siegecraft, Architecture |
| Shang | Long Swordsman/Legion, Elephant Archer, War Elephant, Heavy Catapult, Phalanx/Centurion, Trireme, Catapult Trireme (Orig also Heavy Transport) | Ballista Tower, Siegecraft, Aristocracy, Alchemy, Engineering (Orig also Coinage; Ballistics until a DE update) |
| Sumerians | Legion, Improved/Composite Bowman, Elephant Archer, Cavalry line, Armored Elephant, Ballista/Helepolis, Catapult Trireme | Metallurgy, Iron/Tower Shield, Craftsmanship, Astrology, Afterlife, Monotheism, Fanaticism, Zealotry |
| Yamato | Broad Swordsman line, Chariot Archer, Elephant Archer, Chariot, Camel Rider, War Elephant, Catapult/Heavy Catapult, Ballista/Helepolis, Fire Galley | Fortified Wall, Guard/Ballista Tower, Tower Shield, Astrology, Mysticism, Monotheism, Fanaticism, Zealotry, Medicine |
| Romans | Composite Bowman, Chariot Archer, Horse Archer line, Elephant Archer, Heavy Cavalry/Cataphract, War Elephant, Camel Rider, Fire Galley | Guard/Ballista Tower, Irrigation, Alchemy, Astrology, Afterlife |
| Carthaginians | Legion, Composite Bowman, Chariot Archer, Heavy Horse Archer, Chariot, Cataphract, Catapult/Heavy Catapult, Catapult Trireme | Metallurgy, Chain Mail ×3, Fortified Wall, Siegecraft, Astrology, Monotheism, Fanaticism |
| Macedonians | Long Swordsman/Legion, Chariot Archer, Elephant Archer, Chariot, Camel Rider, Heavy Catapult, Helepolis, Juggernaut, Fire Galley, **Priest** (Orig also Catapult) | **Temple** and all its techs, Fortified Wall, Craftsmanship, Siegecraft, Nobility, Engineering (Orig also Wheel) |
| Palmyrans | Long Swordsman/Legion, Elephant Archer, Cataphract, Helepolis, Centurion, Catapult Trireme (Orig also Heavy Transport) | Metallurgy, Tower Shield, Craftsmanship, Irrigation, Logistics, Aristocracy, Engineering, Mysticism, Polytheism, Monotheism, Medicine (Orig also Coinage, Plow) |

Full tech tree ("Full Tech Tree" game option) removes all of these holes and all civ bonuses.

---

## 5. Unit roster and building roster

Stat columns: cost · HP · attack (M = melee, P = pierce) · armor melee/pierce · range · speed (Orig tiles/s) · LOS · train time. Rate of fire is 1.5 s for most melee and 1.4 s for foot archers unless noted. Orig values; DE changes noted.

### 5.1 Barracks (125 W, 350 HP, 30 s; Stone Age)

| Unit | Age | Cost | HP | Attack | Armor | Range | Speed | LOS | Train | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| Clubman | Stone | 50 F | 40 | 3 M | 0/0 | melee | 1.2 | 4 | 26 s | The only Stone Age soldier. |
| Axeman | Tool | 50 F | 50 | 5 M | 0/0 (DE 0/1) | melee | 1.2 | 4 | 26 s | Upgrade from Clubman via Battle Axe. |
| Slinger (RoR) | Tool | 40 F + 10 S | 25 | 2 P (+bonus vs archers, +7 vs walls/towers orig) | 0/2 | 4 | 1.2 | 5 | 24 s (DE 35) | Only stone-costing unit in the series; counter to Bowmen. |
| Short Swordsman | Bronze | 35 F + 15 G | 60 | 7 M | 1/0 | melee | 1.2 | 4 | 26 s | Needs Short Sword tech in Orig. |
| Broad Swordsman | Bronze | 35 F + 15 G | 70 (DE 80) | 9 M | 1/0 | melee | 1.2 | 4 | 26 s | |
| Long Swordsman | Iron | 35 F + 15 G | 80 (DE 100) | 11 M | 2/0 | melee | 1.2 | 4 | 26 s | |
| Legion | Iron | 35 F + 15 G | 160 (DE 140) | 13 M | 2/0 | melee | 1.2 | 4 | 26 s | Needs Fanaticism in Orig/DE. Cheap pop-efficient meat shield. |

### 5.2 Archery Range (150 W, 350 HP, 40 s; Tool Age, needs Barracks)

| Unit | Age | Cost | HP | Attack | Armor | Range | Speed | LOS | Train | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| Bowman | Tool | 40 F + 20 W | 35 | 3 P | 0/0 | 5 | 1.2 | 7 | 30 s | ROF 1.4 |
| Improved Bowman | Bronze | 40 F + 20 G | 40 | 4 P | 0/0 | 6 | 1.2 | 8 | 30 s | Needs Improved Bow tech |
| Composite Bowman | Bronze | 40 F + 20 G | 45 | 5 P | 0/0 | 7 | 1.2 | 9 | 30 s | The Bronze Age gold-archer; Minoan +2 range |
| Chariot Archer | Bronze | 40 F + 70 W | 70 | 4 P (+3× vs Priests orig) | 0/0 | 7 | 2.0 | 9 | 40 s | Needs Wheel; no gold; conversion-resistant |
| Horse Archer | Iron | 50 F + 70 G | 60 | 7 P | 0/2 (DE 0/1) | 7 | 2.2 | 9 | 40 s | |
| Heavy Horse Archer | Iron | 50 F + 70 G | 90 | 8 P | 0/2 | 7 | 2.5 | 9 | 40 s | Needs Chain Mail Archers; fastest unit in the game |
| Elephant Archer | Iron | 180 F + 60 G | 600 | 5 P (DE 6) | 0/0 | 7 | 0.9 | 9 | 50 s | Walking tower; Priest bait |

### 5.3 Stable (150 W, 350 HP, 40 s; Tool Age, needs Barracks)

| Unit | Age | Cost | HP | Attack | Armor | Range | Speed | LOS | Train | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| Scout | Tool | 100 F (DE 90) | 60 | 3 M | 0/0 | melee | 2.0 | 8 (DE 7) | 30 s | Explorer, artifact grabber |
| Cavalry | Bronze | 70 F + 80 G | 150 | 8 M (+5 vs infantry) | 0/0 (DE 0/1) | melee | 2.0 | 4 | 40 s | ROF 1.3 orig |
| Heavy Cavalry | Iron | 70 F + 80 G | 150 | 10 M (+5 vs inf) | 1/1 (DE 2/2) | melee | 2.0 | 4 | 40 s | |
| Cataphract | Iron | 70 F + 80 G | 180 (DE 240) | 12 M (+5 vs inf; DE +7) | 3/1 (DE 5/3) | melee | 2.0 | 4 | 40 s | Needs Metallurgy |
| Chariot | Bronze | 40 F + 60 W | 100 | 7 M (+2× vs Priests orig) | 0/0 | melee | 2.0 | 4 | 40 s | Needs Wheel; no gold; conversion-resistant; Egyptian +33 % HP |
| Scythe Chariot (RoR) | Iron | 40 F + 60 W | 120 | 9 M, trample | 2/0 (DE 1/0) | melee | 2.0 | 4 | 40 s | Needs Nobility; area damage to adjacent units |
| Camel Rider (RoR) | Bronze | 70 F + 60 G | 125 | 6 M (+8 vs cavalry) | 0/0 | melee | 2.0 | 4 | 30 s | Anti-cavalry |
| War Elephant | Iron | 170 F + 40 G | 600 | 15 M, trample (+25 vs walls/buildings in DE) | 0/0 | melee | 0.9 | 5 | 50 s | Persian +50 % speed; Phoenician −25 % |
| Armored Elephant (RoR) | Iron | 170 F + 40 G | 600 | 18 M, trample (+35 vs buildings) | 2/1 | melee | 0.9 | 5 | 50 s | Needs Iron Shield |

### 5.4 Academy (200 W → DE 150 W, 350 HP, 60 s; Bronze Age, needs Stable)

| Unit | Age | Cost | HP | Attack | Armor | Speed | LOS | Train | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Hoplite | Bronze | 60 F + 40 G | 120 | 17 M | 5/0 | 0.9 | 4 | 36 s | The Bronze Age wall; Greek +30 % speed |
| Phalanx | Iron | 60 F + 40 G | 120 | 20 M | 7/0 | 0.9 | 4 | 36 s | |
| Centurion | Iron | 60 F + 40 G | 160 | 30 M | 8/0 | 0.9 | 4 | 36 s | Needs Aristocracy. Highest melee attack in the game; Macedonian +2 pierce armor |

ROF for Academy units was 1.5 s in Orig (2.0 s in early DE, back to 1.5 later).

### 5.5 Siege Workshop (200 W, 350 HP, 60 s; Bronze Age, needs Archery Range)

| Unit | Age | Cost | HP | Attack | Armor | Range (min) | Speed | LOS | Train | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| Stone Thrower | Bronze | 180 W + 80 G | 75 | 50 M, blast 0.5 (+140 vs buildings, +50 vs towers) | 0/0 | 10 (2) | 0.8 | 13 | 60 s | ROF 5 s; Sumerian +50 % ROF; Hittite ×2 HP |
| Catapult | Iron | 180 W + 80 G | 75 | 50 (DE 60) M, blast 1.5 | 0/0 | 12 (2) | 0.8 | 15 | 60 s | |
| Heavy Catapult | Iron | 180 W + 80 G | 150 | 60 M, blast 2.0 | 0/0 | 13 (2) | 0.8 | 16 | 60 s | Needs Siegecraft; can attack-ground to clear trees |
| Ballista | Iron | 100 W + 80 G | 55 | 40 P (+5 vs Fire Galley) | 0/0 | 9 (3) | 0.8 | 11 | 50 s | ROF 3 s; precise single-target |
| Helepolis | Iron | 100 W + 80 G | 55 | 40 P | 0/0 | 10 (3) | 0.8 | 12 | 50 s | Needs Craftsmanship; ROF 1.5 s (DE 1.85) — the "machine gun" |

Catapult projectiles hit everything in the blast, including your own units; good players dodge boulders because projectiles are aimed at the target's position when fired (Ballistics makes towers/archers/siege lead moving targets).

### 5.6 Temple (200 W, 350 HP, 60 s; Bronze Age, needs Market)

| Unit | Cost | HP | Range | Speed | LOS | Train | Notes |
|---|---|---|---|---|---|---|---|
| Priest | 125 G | 25 | 10 | 0.8 | 12 | 50 s | Heals allies, converts enemies. Gold only. Egyptian +3 range, Choson −30 % cost, Babylonian rejuvenation +30 %. Sees further than any land unit. |

### 5.7 Dock (100 W, 350 HP, 50 s; Stone Age)

Docks must sit on a shoreline (not deep water). Dock work rate is 150 % in Stone/Tool and 200 % in Bronze/Iron (ships train faster as you age). Trade is described in §3.6.

| Unit | Age | Cost | HP | Attack | Range | Speed | LOS | Train | Notes |
|---|---|---|---|---|---|---|---|---|---|
| Fishing Boat | Stone | 50 W | 45 | — | — | 1.45 | 6 | 20 s | 0.4 food/s |
| Fishing Ship | Bronze | 50 W | 75 | — | — | 2.1 | 6 | 20 s | Upgrade 50 F + 100 W |
| Trade Boat | Stone | 100 W | 200 | — | — | 2.1 | 4 | 25 s | |
| Merchant Ship | Bronze | 100 W | 250 | — | — | 2.5 | 4 | 25 s | Upgrade 200 F + 75 W |
| Light Transport | Tool | 150 W | 150 | — | — | 1.45 | 4 | 38 s | Carries 5 |
| Heavy Transport | Iron | 150 W | 200 | — | — | 1.8 | 5 | 38 s | Carries 10; upgrade 300 F |
| Scout Ship | Tool | 135 W | 120 | 5 P | 5 | 1.8 | 7 | 30 s | ROF 1.4–1.5 |
| War Galley | Bronze | 135 W | 160 | 8 P | 6 | 1.8 | 9 | 30 s | Upgrade 150 F + 75 W |
| Trireme | Iron | 135 W | 200 | 12 P | 7 | 1.8 | 10 | 30 s | Upgrade 250 F + 100 W; Persian +50 % ROF |
| Catapult Trireme | Iron | 135 W + 75 G (DE 50) | 120 (DE 135) | 35 "melee" blast, ignores armor (+140 vs buildings) | 9 | 1.4 | 12 | 45 s | Phoenician fires 65 % faster |
| Juggernaut | Iron | 135 W + 75 G | 200 | 35 blast | 10 | 1.4 | 13 | 45 s | Needs Engineering; upgrade 1,300 F + 700 W |
| Fire Galley (RoR) | Iron | 115 W + 40 G | 200 | 24 M (+130 vs buildings) | 1 | 2.0 | 8 | — | Only melee ship; Carthaginian +25 % attack |

All ships have 0/0 armor; the Hittite range bonus and Yamato HP bonus are the main naval civ perks. War ships are deadly against shoreline melee units, which cannot fight back.

### 5.8 Heroes and special units (campaigns/editor only)

Heroes are named reskins with better stats: e.g. Hero Alexander, Hero Xerxes, Hero Perseus, Hero Jason, Hero Tiberius, Hero Hannibal, Hero Caesar, Hero Mark Antony, Hero Scipio, Hero Vercingetorix, Hero Archimedes, Cleopatra's Barge, plus Corlis and Hersifon (villager-looking heroes). Editor-only objects include Artifacts (movable, captured by proximity), Ruins (static, captured by line of sight, indestructible), Discoveries, Rafts, Tame Lions, Blind Lame Priests, and in DE the Chieftain/General/Pharaoh/High Priest.

### 5.9 Building roster

| Building | Age | Cost | HP | Build time | Size | Armor (M/P) | Notes |
|---|---|---|---|---|---|---|---|
| Town Center | Stone | 200 W | 600 | 60 s | 3×3 | 0/0 | +4 pop; extra TCs need Government Center; no attack, no garrison in Orig/DE |
| House | Stone | 30 W | 75 | 20 s | 2×2 | 0/0 | +4 pop |
| Barracks | Stone | 125 W | 350 | 30 s | 3×3 | 0/0 | unlocks Range and Stable |
| Granary | Stone | 120 W | 350 | 30 s | 3×3 | 0/0 | unlocks Market; wall/tower techs |
| Storage Pit | Stone | 120 W | 350 | 30 s | 3×3 | 0/0 | attack/armor techs |
| Dock | Stone | 100 W | 350 | 50 s | 3×3 | 0/0 | on shoreline |
| Archery Range | Tool | 150 W | 350 | 40 s | 3×3 | 0/0 | unlocks Siege Workshop |
| Stable | Tool | 150 W | 350 | 40 s | 3×3 | 0/0 | unlocks Academy |
| Market | Tool | 150 W | 350 | 40 s | 3×3 | 0/0 | unlocks Farm, Temple, Government Center; tribute |
| Farm | Tool | 75 W | 50 | 30 s (DE 24) | 3×3 | 0/0 | 250 food; needs Market |
| Small Wall | Tool | 5 S | 200 | 7 s | 1×1 | 0/3 | needs Granary tech (50 F) |
| Watch Tower | Tool | 150 S | 100 (DE 125) | 80 s (DE 65) | 2×2 | 0/3 | 3 P attack (DE 5), range 5 (DE 6), LOS 8; needs Granary tech |
| Academy | Bronze | 200 W (DE 150) | 350 | 60 s | 3×3 | 0/0 | needs Stable |
| Siege Workshop | Bronze | 200 W | 350 | 60 s | 3×3 | 0/0 | needs Archery Range |
| Temple | Bronze | 200 W | 350 | 60 s | 3×3 | 0/0 | needs Market |
| Government Center | Bronze | 175 W | 350 | 60 s | 3×3 | 0/0 | needs Market; unlocks extra TCs |
| Medium Wall | Bronze | 5 S | 300 | 7 s | 1×1 | 0/4 | upgrade 180 F + 100 S |
| Sentry Tower | Bronze | 150 S | 150 (DE 185) | 80 s | 2×2 | 0/4 | 4 P, range 6, LOS 9; upgrade 120 F + 50 S |
| Fortified Wall | Iron | 5 S | 400 | 7 s | 1×1 | 0/4 (DE 5) | upgrade 300 F + 175 S |
| Guard Tower | Iron | 150 S | 200 (DE 240) | 80 s | 2×2 | 0/4 (DE 5) | 6 P, range 7, LOS 10; upgrade 300 F + 100 S |
| Ballista Tower | Iron | 150 S | 200 (DE 240) | 80 s | 2×2 | 0/4 | 20 P bolt (DE 18), ROF 3 s, range 7; needs Ballistics; upgrade 1,800 F + 750 S |
| Wonder | Iron | 1,000 W + 1,000 G + 1,000 S | 500 | 8,000 work-seconds (many villagers needed) | 5×5 | 0/0 | Standard-victory countdown of 2,000 years once complete |

Gates do not exist in Orig or DE (they arrived with *Return of Rome*: 30 S, 300/450/600 HP). Buildings have no melee/pierce armor but take only 20 % of ordinary damage (see §7), which is why "+140 vs buildings" bonuses exist.

Architecture (Government Center) gives all buildings +20 % HP and −33 % build time. Choson towers +2 range, Babylonian walls/towers +100 % HP (DE +60 %), Shang walls +100 % HP, Roman buildings −15 % and towers −50 %.

---

## 6. Technologies by building

All costs are Orig/RoR unless noted. "Time" is research time in seconds.

### 6.1 Town Center

| Tech | Age | Cost | Time | Effect |
|---|---|---|---|---|
| Tool Age | Stone | 500 F | 120 | Advance; needs 2 Stone Age buildings |
| Bronze Age | Tool | 800 F | 140 | Advance; needs 2 Tool Age buildings |
| Iron Age | Bronze | 1,000 F + 800 G | 160 | Advance; needs 2 Bronze Age buildings |
| City Watch (RoR only) | Tool | 150 F + 175 W | 50 | Buildings +3 LOS |

### 6.2 Granary (defensive structures)

| Tech | Age | Cost | Time | Effect |
|---|---|---|---|---|
| Small Wall | Tool | 50 F | ~30 | Enables Small Wall (5 S / 200 HP) |
| Watch Tower | Tool | 50 F *(unverified)* | ~30 | Enables Watch Tower |
| Sentry Tower | Bronze | 120 F + 50 S | 30 | Towers +60 HP, +1 attack, +1 pierce armor, +1 range/LOS |
| Medium Wall | Bronze | 180 F + 100 S | 60 | Walls +100 HP, +1 pierce armor |
| Guard Tower | Iron | 300 F + 100 S | 75 | Towers +55 HP, +1 attack, +1 pierce armor, +1 range/LOS |
| Fortified Wall | Iron | 300 F + 175 S | 75 | Walls +100 HP, +1 pierce armor |
| Ballista Tower | Iron | 1,800 F + 750 S | 150 | Towers fire ballista bolts (20 attack, ROF 3 s); needs Ballistics |

### 6.3 Storage Pit (weapons and armor)

| Tech | Age | Cost | Time | Effect |
|---|---|---|---|---|
| Toolworking | Tool | 100 F | 40 | Melee units +2 attack (infantry, cavalry, Academy; not elephants) |
| Metalworking | Bronze | 200 F + 120 G | 75 | Melee +2 attack |
| Metallurgy | Iron | 300 F + 180 G | 100 | Melee +3 attack; unlocks Cataphract |
| Leather Armor Infantry | Tool | 75 F | 30 | Infantry +2 melee armor |
| Scale Armor Infantry | Bronze | 100 F + 50 G | 60 | +2 |
| Chain Mail Infantry | Iron | 125 F + 100 G | 75 | +2 |
| Leather Armor Archers | Tool | 100 F | 30 | Archers (incl. mounted) +2 melee armor |
| Scale Armor Archers | Bronze | 125 F + 50 G | 60 | +2 |
| Chain Mail Archers | Iron | 150 F + 100 G | 75 | +2; unlocks Heavy Horse Archer |
| Leather Armor Cavalry | Tool | 125 F | 30 | Cavalry +2 melee armor |
| Scale Armor Cavalry | Bronze | 150 F + 50 G | 60 | +2 |
| Chain Mail Cavalry | Iron | 175 F + 100 G | 75 | +2 |
| Bronze Shield | Bronze | 150 F + 180 G | 50 | Barracks + Academy units (and Slingers) +1 pierce armor |
| Iron Shield | Iron | 200 F + 320 G | 75 | +1 pierce armor; unlocks Armored Elephant |
| Tower Shield (RoR) | Iron | 250 F + 400 G | 100 | +1 pierce armor |

There are **no** ranged-attack upgrades at the Storage Pit; archers only get attack from Alchemy (and range from Woodworking line).

### 6.4 Market (economy; also archer range)

| Tech | Age | Cost | Time | Effect |
|---|---|---|---|---|
| Woodworking | Tool | 120 F + 75 W | 60 | Woodcutters +20 % rate (DE; Orig stated +36 %) and +2 carry; **archers, towers and arrow ships +1 range and LOS** |
| Artisanship | Bronze | 170 F + 150 W | 80 | Same again |
| Craftsmanship | Iron | 240 F + 200 W | 100 | Same again; unlocks Helepolis |
| Stone Mining | Tool | 100 F + 50 S | 30 | Stone miners +30 % (Orig +67 %) and +3 carry; Slingers +1 attack/range/LOS |
| Siegecraft | Iron | 190 F + 100 S | 60 | Stone miners +30 % and +3 carry; villagers +10 attack vs walls/towers; unlocks Heavy Catapult |
| Gold Mining | Tool | 120 F + 100 W | 50 | Gold miners +30 % (Orig +67 %) and +3 carry |
| Coinage | Iron | 200 F + 100 G | 60 | Gold miners +25 % (DE) / mines last 25 % longer; tribute becomes free |
| Domestication | Tool | 200 F + 50 W (DE 150 F) | 40 | Farms +75 food |
| Plow | Bronze | 250 F + 75 W | 75 | Farms +75 (DE +100) |
| Irrigation | Iron | 300 F + 100 W | 100 | Farms +75 (DE +125) |
| Wheel | Bronze | 175 F + 75 W | 75 | Villagers +0.55 speed (~+50 %); enables Chariot and Chariot Archer |

### 6.5 Government Center

| Tech | Age | Cost | Time | Effect |
|---|---|---|---|---|
| Nobility | Bronze | 175 F + 120 G | 70 | Mounted units (not elephants) +15 % HP; unlocks Scythe Chariot (RoR) |
| Writing | Bronze | 200 F + 75 G | 30 | Share exploration with allies |
| Architecture | Bronze | 150 F + 175 W | 45 | Buildings +20 % HP, villagers build 50 % faster (−33 % time) |
| Logistics (RoR) | Bronze | 180 F + 100 G | 60 | Barracks units count as ½ pop |
| Aristocracy | Iron | 175 F + 150 G | 60 | Academy units +0.25 speed (≈ +28 %); unlocks Centurion |
| Ballistics | Iron | 200 F + 50 G | 60 | Projectiles lead moving targets; unlocks Ballista Tower |
| Alchemy | Iron | 250 F + 200 G | 100 | All missile units +1 attack (Orig: Ballista/Helepolis +2, Fire Galley +6) |
| Engineering | Iron | 200 F + 100 W | 70 | Stone Thrower, Ballista and Catapult Trireme lines +2 range and LOS; unlocks Juggernaut |
| Urbanization / Conscription | — | — | — | *Return of Rome* only |

### 6.6 Temple (gold only)

| Tech | Age | Cost | Time | Effect |
|---|---|---|---|---|
| Astrology | Bronze | 175 G | 50 | Conversion and healing 30 % more effective |
| Mysticism | Bronze | 120 G | 50 | Priest HP ×2 (DE +20) |
| Polytheism | Bronze | 120 G | 50 | Priests +40 % speed |
| Afterlife | Iron | 275 G | 75 | Priests +3 range and LOS |
| Monotheism | Iron | 350 G | 75 | Priests can convert enemy Priests and buildings (not TCs, Wonders, walls) |
| Fanaticism | Iron | 150 G | 60 | Faith recovers 50 % faster; unlocks Legion |
| Jihad (DE: Zealotry) | Iron | 120 G | 60 | Villagers +40 HP, +7 attack, +0.3 speed, −8 carry capacity |
| Medicine (RoR) | Iron | 150 G | 50 | Healing +200 % (DE +100 %) |
| Martyrdom (RoR; DE "Sacrifice") | Iron | 400 G | 100 | A Priest can sacrifice himself to convert a unit instantly |

### 6.7 Unit-line upgrades (researched at the production building)

| Building | Upgrade | Cost | Time | Prerequisite |
|---|---|---|---|---|
| Barracks | Battle Axe (Clubman → Axeman) | 100 F | 40 | — |
| Barracks | Short Sword (enables Short Swordsman; Orig only) | 120 F + 50 G | — | Battle Axe |
| Barracks | Broad Sword | 140 F + 50 G | 90 | — |
| Barracks | Long Sword | 240 F + 100 G | 90 | — |
| Barracks | Legion | 1,400 F + 600 G | 150 | Fanaticism |
| Archery Range | Improved Bow (enables Improved Bowman) | 140 F + 80 W *(unverified)* | — | — |
| Archery Range | Composite Bow | 180 F + 100 W | 100 | — |
| Archery Range | Heavy Horse Archer | 1,750 F + 800 G | 150 | Chain Mail Archers |
| Stable | Heavy Cavalry | 350 F + 125 G | 90 | — |
| Stable | Cataphract | 1,600 F + 600 G | 150 | Metallurgy |
| Stable | Scythe Chariot | 1,400 W + 1,000 G | 200 (Orig 150) | Nobility |
| Stable | Armored Elephant | 1,000 F + 1,200 G | 150 | Iron Shield |
| Academy | Phalanx | 300 F + 100 G | 90 | — |
| Academy | Centurion | 1,000 F (DE) *(Orig value unverified; believed higher, ~1,800 F + 700 G)* | 75 | Aristocracy |
| Siege Workshop | Catapult | 300 F + 250 W | 100 | — |
| Siege Workshop | Heavy Catapult | 1,800 F + 900 W | 150 | Siegecraft |
| Siege Workshop | Helepolis | 1,200 F + 1,000 W | 150 | Craftsmanship |
| Dock | Fishing Ship | 50 F + 100 W | 25–50 | — |
| Dock | Merchant Ship | 200 F + 75 W | 50–100 | — |
| Dock | Heavy Transport | 300 F | 50 | — |
| Dock | War Galley | 150 F + 75 W | 64–128 | — |
| Dock | Trireme | 250 F + 100 W | 100–200 | — |
| Dock | Juggernaut | 1,300 F + 700 W | 100–200 | Engineering |

Dock research times depend on the age (the Dock work-rate bonus applies to research too).

Note the spending pattern: tier-1/2 upgrades are a few hundred resources; the final upgrade of every line is a 2,000–2,600-resource investment with a tech prerequisite in another building. A full Iron Age "all upgrades" build is not possible; players choose two or three lines.

---

## 7. Combat model

### 7.1 Damage formula

AoE1 (and AoE2) use additive attack/armor classes, not percentage mitigation:

```
damage_to_unit = max(1,
      ( max(0, melee_attack  − melee_armor)
      + max(0, pierce_attack − pierce_armor)
      + Σ max(0, bonus_i − bonus_resist_i) ) × k_elevation )

damage_to_building = max(0.1, (same sum) × k_elevation × 0.2)
```

- Every attack is either **melee** (villagers, Barracks/Academy/Stable units, Stone Thrower line) or **pierce** (archers, Slingers, towers, Ballista line, arrow ships). The Catapult Trireme/Juggernaut and Fire Galley are flagged as melee but their boulder ignores armor entirely.
- A unit always deals at least 1 damage; buildings always take at least 0.1.
- **Buildings resist 80 %** of all damage. That is why siege carries +140 vs buildings (+50 vs towers) — a Stone Thrower's 50 + 140 = 190 × 0.2 = 38 per shot — while a Villager's +10 from Siegecraft is worth only +2.
- **Elevation**: melee attacking uphill deals ×0.67; ranged attacking downhill deals ×1.5. Hills matter.
- **Attack bonuses** are per armor class: Cavalry +5 vs infantry, Camel +8 vs cavalry, Axeman +4 vs mounted (DE), Slinger +bonus vs archers and walls/towers, Chariot/Chariot Archer ×2/×3 vs Priests (Orig), elephants +25/+35 vs buildings (DE), siege +140/+50 vs buildings/towers, everything +10 vs Fire Galley.
- Armor types: *melee armor* comes from Leather/Scale/Chain Mail (three separate lines for infantry, archers, cavalry — mounted archers use the archer line); *pierce armor* comes from the Shield line (Barracks and Academy units only) and from unit base stats. Villagers, Priests, siege, ships and buildings can never gain melee armor. Towers and walls have 3–5 pierce armor, which is why archers cannot hurt them.
- Rate of fire: 1.5 s standard; 1.4 s foot archers; 1.3 s cavalry (Orig); 5 s catapults; 3 s Ballista; 1.5 s Helepolis; 3 s Ballista Tower. Projectiles have a flight time (archers 8 tiles/s, catapults ~2.7 tiles/s), so they can miss moving targets until Ballistics is researched.
- **Area damage**: Stone Thrower 0.5-tile blast, Catapult 1.5, Heavy Catapult 2.0, Catapult Trireme 1.0, Juggernaut 1.5; blast hurts friends too. War Elephants and Scythe Chariots **trample** (damage to adjacent units).
- Siege has a **minimum range** (2 tiles for catapults, 3 for Ballistae) — rushing them with cavalry is the counter.
- Heavy Catapults can attack-ground to destroy trees, the only way to open a wood line besides cutting it.

### 7.2 Range and line of sight

- Range and LOS are in tiles. LOS is what the unit reveals from the fog; range is what it can hit. Typical: melee LOS 4, Bowman 7/5, Composite 9/7, Priest 12/10, Stone Thrower 13/10, Heavy Catapult 16/13, Scout 8, towers 8–10.
- Woodworking/Artisanship/Craftsmanship add +1 range and LOS each to archers, towers and arrow ships (so a fully upgraded Composite Bowman reaches 10). Engineering +2 to siege and siege ships. Afterlife +3 to Priests. Hittite ships +4, Egyptian Priests +3, Choson towers +2, Minoan Composite +2, Macedonian melee +2 LOS.
- Fog of war: explored-but-not-seen terrain stays visible as last seen; enemy buildings remain drawn in the fog. Writing shares exploration with allies.
- Units are hit only if a projectile physically arrives; towers and archers fire at where the target *was*, so micro-dodging catapult boulders and tower arrows is a real skill.

### 7.3 Conversion (Priests)

- A Priest at full **faith** targets an enemy unit within 10 tiles and starts chanting ("wololo"); after a random duration the unit changes side with all its upgrades frozen at that moment. Faith then drops to zero and refills slowly (Fanaticism +50 %, Babylonian +30 %).
- Base: any unit except other Priests. Buildings and Priests need **Monotheism** (never Wonders, Town Centers, walls). Chariots, Chariot Archers and all ships are more resistant; all Macedonian units and buildings are 4× more resistant; slow units (elephants, siege) are easy prey, and a Priest out-ranges a Stone Thrower.
- Astrology +30 % conversion/healing effectiveness; Afterlife +3 range; Mysticism doubles HP; Polytheism +40 % speed; Martyrdom (RoR) trades the Priest for an instant conversion.
- Healing is free and permanent, so Priests behind a Hoplite line make it almost unkillable by melee.
- Counters: Chariots and Chariot Archers (fast, bonus vs Priests, conversion-resistant), Horse Archers, Ballistae, or simply killing the 25-HP Priest with anything that reaches him.

### 7.4 Unit counters in practice

- Bowman/Slinger/Axeman/Scout rock-paper-scissors in Tool: Slingers beat Bowmen, Axemen beat Slingers, Bowmen beat Axemen, Scouts chase Bowmen.
- Bronze: Hoplites beat everything melee but are slow; Chariot Archers and Composite Bowmen kite them; Cavalry beat archers; Stone Throwers beat clumps; Priests beat slow expensive units; Camels beat Cavalry.
- Iron: Centurions and Cataphracts are the elite; Helepoleis and Heavy Catapults are the artillery; Legions are the cheap filler; Elephants are tanks that Priests steal.
- Water: Triremes vs Catapult Triremes vs Fire Galleys form a triangle (Fire Galleys melt ships but die to Triremes' range).

---

## 8. Random maps, scenario editor, multiplayer

### 8.1 Random map types

| Map | Introduced | Character |
|---|---|---|
| Small Islands | AoE | Each player on their own small island; naval map, Docks mandatory |
| Large Islands | AoE | Fewer, bigger islands shared between players |
| Coastal | AoE | One landmass with a long coastline; mixed land/sea |
| Inland | AoE | All land, open, little or no water — the "Arabia" of AoE1 |
| Highland | AoE | Land map with rivers/lakes and elevation; shallows chokepoints |
| Continental | RoR | One big continent surrounded by sea |
| Mediterranean | RoR | Central sea ringed by land |
| Hill Country | RoR | Hilly, no water |
| Narrows | RoR | Islands connected by narrow land bridges |
| Oasis, Rivers | DE | Added in 2018 |

Map sizes: Tiny, Small, Medium, Normal, Large, Huge, plus **Gigantic (250×250 tiles)** from RoR. Terrain types: grass, desert, forest, water, palm desert, jungle, shallows, pine forest, deep water, with 7 elevation levels and cliffs.

Game setup options: map type, map size, starting age, resources (Default/Low/Medium/High), difficulty (Easiest/Easy/Moderate/Hard/Hardest), population limit (RoR, multiplayer), reveal map (normal/explored/all visible), fixed positions, full tech tree, team together, allow cheats, lock teams, and game type: **Random Map, Death Match (huge starting stockpiles), Scenario**.

Victory conditions:
- **Standard**: conquest, *or* build a Wonder and hold it for 2,000 years, *or* hold every Ruin on the map for 2,000 years, *or* hold every Artifact for 2,000 years (2,000 game years = 1,000 s at 1.0× ≈ 10 real minutes at normal speed). The whole map is warned by a countdown message.
- **Conquest**: destroy all enemy units and buildings (walls not required).
- **Score**: first to reach the score target.
- **Time limit**: highest score when the clock expires.
- Artifacts are wagons that change owner when a unit walks up to them; Ruins are stonehenge-like sites that convert on line of sight.

### 8.2 Scenario Builder (editor)

AoE1's editor had **no triggers** at all. Everything a designer could do was data, not logic:

- **Map tab**: blank map (grass/desert/water/deep water) or generate one of the random map types in any size.
- **Terrain tab**: paint terrains, cliffs and elevation 1–7 with brushes of 1×1 to 9×9.
- **Players tab**: number of players, civ, starting age, starting resources (0–99,999), tribe/ruler name, Human/Computer/Either; AI **Strategy** (which unit mix the AI prefers) and **Personality** (Default, Passive, Defensive, Aggressive, Super Aggressive, "Aggressive 3 Attackers", "Passive Aggressive"…) files.
- **Units tab**: place any unit, building, hero, Gaia animal, resource or decoration for any player, including hidden/beta objects via data hacks (the famous spaceship, Winsett's Z, Mirror Tower, Zenobia's Tower, pyramids, colosseum, aqueducts).
- **Diplomacy tab**: initial stances, allied victory.
- **Global Victory**: Standard / Conquest / Score / Time Limit / Custom (checkbox list of the conditions below).
- **Individual Victory**: up to **12 conditions per player**, each one of: Bring Object to Object, Bring Object to Area, Create *N* Objects, Create Objects in Area, Destroy *N* Objects, Destroy Specific Object, Destroy All Objects (of a type/player), Destroy Player, Capture an Object, Food/Wood/Gold/Stone stockpile, Population, Age, Exploration %, Technology researched, and "Other Attributes" (razings, conversions, kill ratio, military population, technologies count, villager population, kills, wonders). Conditions are ANDed; there is no OR, no timers, no spawning, no dialogue mid-game.
- **Options tab**: disable individual units/buildings/techs per player; full tech tree.
- **Messages tab**: scenario instructions, hints, victory/loss text, history text (shown before the game and in the objectives panel).
- **Cinematics tab**: an AVI before and after the scenario.
- Campaign files chain scenarios linearly; heroes and story live in the pre-scenario text.

Consequences: campaign designers scripted "events" by placing Gaia units that convert when discovered, by pre-damaged buildings, by AI personalities, and by the dozen victory conditions. Triggers only arrived with AoE2 (1999) and, for AoE1 content, with *Return of Rome* (2023). DE kept the AoE1 editor largely as-is (new objects/terrains, no triggers).

### 8.3 Multiplayer

- Up to **8 players**, any mix of humans and AI, on IPX LAN, TCP/IP, modem-to-modem, serial cable, or the **MSN Gaming Zone** (support ended 19 June 2006; today GameRanger/Voobly/Garena, and DE's Xbox Live lobbies).
- **Deterministic lockstep** (Terrano & Bettner, "1500 Archers on a 28.8", GDC 2001):
  - Peer-to-peer; every machine runs the full simulation; only player **commands** are sent, never unit state.
  - Time is sliced into **communication turns of ~200 ms**; a command issued in turn *N* executes in turn *N+2* on every machine, giving packets time to arrive and be acknowledged.
  - UDP with their own ordering/ack/resend layer ("when in doubt, assume it dropped").
  - **Speed control**: each client reports its frame rate and worst ping in a 2-byte "turn done" message; the host adjusts turn length — quickly up when latency spikes, slowly down (a few ms per turn). The target was 15 fps on a 16 MB Pentium 90 with a 28.8 k modem. Command latency under 250 ms was unnoticed, 250–500 ms playable, 500+ ms sluggish; steady latency felt better than fluctuating latency.
  - **Out-of-sync detection** by comparing world checksums each turn; a divergence ends the game with the infamous "Out of sync" error. 1.0c added deliberate desync when known trainers/hacks were detected.
  - Observed input rate: a command every 1.5–2 s on average, bursts of 3–4 per second in battles; repeated identical clicks were filtered.
  - Because every peer must deliver its turn before anyone advances, one slow or lagging machine **stalls everyone** — this is the "Waiting for players…" / connection-lost dialog with the list of player names and the yellow/red lag clock next to the laggard, the single most remembered multiplayer screen of the era. Lag was blamed on "the guy with the modem". Game speed could also be changed mid-game by the host (speed changes are just commands).
- Game speeds: Slow / Normal / Fast (roughly 1.0× / 1.5× / 2.0× of simulation time; DE's "normal" runs at about 1.7×). Single-player allowed changing speed any time; multiplayer could lock it.
- Settings in the lobby: map, size, age, resources, population (RoR 25–200), reveal map, teams (via diplomacy, no fixed team slots in the original — "team together"/"lock teams" came later), full tech tree, allow cheats, victory type.
- No ranked ladder, no replays, no spectators in Orig; the Zone kept a rating. DE added Xbox Live lobbies, Elo, ranked modes and (via Steam) cross-play with the Microsoft Store version.
- Unit selection cap of 25 units (AoE2 raised it to 40, DE to larger numbers) *(unverified)*, number keys for control groups, no formations, no attack-move, no rally points in Orig.

### 8.4 AI

Single-player AI used per-personality text rules (the `.ai`/`.per` files) with no script language; it built by fixed lists and attacked in waves. It cheated on resources at higher difficulties, could not use transports well, and always walled nothing. DE replaced it with a new AI that does boom and raid.

---

## 9. What the Definitive Edition changed

Developed by Forgotten Empires (the AoE2 "Forgotten Empires" modders) with Tantalus Media; Microsoft Store 20 Feb 2018, Steam 19 Aug 2019 with cross-play, also on Game Pass for PC.

**Presentation**
- Redrawn 4K sprites for every unit, building and terrain; new zoom levels; a "Classic Mode" graphics toggle that shows the 1997 art.
- Fully remastered soundtrack (Rippy brothers' themes re-recorded) and remastered sound effects; all unit voices re-recorded (the only Definitive Edition to redo voices).
- Revamped UI with a customisable HUD, Alt+Enter full-screen toggle, achievements, collectible backgrounds.

**Quality of life (things the 1997 game lacked)**
- Attack-move (A + right-click).
- Build queues (RoR had single-type queues; DE later added mixed unit and tech queues and a global queue indicator).
- Rally points for military buildings and villager rally points at the TC.
- Idle-villager button and a per-resource villager counter in the HUD.
- Farms are walkable, can be rebuilt by right-click when exhausted, and (post-launch) auto-reseed via a Granary toggle.
- Villagers keep carried food when switching between food sources.
- Enhanced pathfinding (still criticised), scouts auto-retaliate, priests auto-heal, unit stances (post-launch).
- Population limit selectable 25–250 (campaigns fixed at 50); up to 8 players; new lobby options (lock teams, lock speed, shared exploration, full tech tree, allow cheats, team together).
- Automatic transfer of custom scenarios/maps to lobby members; new editor objects and terrains; still **no triggers**.
- Multiplayer over Xbox Live servers (no longer peer-to-peer hosting), Elo ranked lobbies for Random Map and Death Match.
- Blood toggle, "Going Above and Beyond" and "Tech Tech One Two Free" cheats, `noui`.

**Content**
- All 10 campaigns (the four AoE, four RoR, plus the two demo campaigns "Reign of the Hittites" and "The First Punic War"), several scenarios reworked and renamed (Pax Romana → Imperium Romanum).
- No new civilizations, no new units; new random maps Oasis and Rivers.

**Balance (selection)**
- Civ bonuses normalised: Babylonian walls/towers +100 % → +60 %; Shang walls same; Hittite catapult HP +100 % → +50 %; Babylonian/Egyptian/Phoenician gather bonuses from the buggy 36–44 % to 15–20 %; Persian hunting 67 % → 30 %; Persian elephant speed 56 % → 25 %; Assyrian villagers → 10 %, archer ROF staged by age; Phoenician siege-ship ROF 72 % → 33 %; Minoan ship discount staged by age; Yamato mounted discount 25 % → 15 %; Macedonian siege −50 % → −25 %; Roman buildings −15 % → −10 %, towers −50 % → −40 %; Greek Academy units −20 % cost; Egyptian farms −20 %; Carthaginians +50 starting resources; Sumerian farms +250 → +125.
- Unit stats: Legion 160 → 140 HP; Broad/Long Swordsman +10/+20 HP; Cataphract 180 → 240 HP and 3/1 → 5/3 armor; Heavy Cavalry armor 1/1 → 2/2; Scout 100 → 90 food; Slinger reworked (LOS 6, 3 pierce armor, bonuses tuned); Stone Thrower line got pierce armor (3/4/5) and −2 melee armor, catapult ROF 5 → 5.375, min range 2 → 3; Elephant Archer 5 → 6 attack; Catapult Trireme 75 → 50 gold and 120 → 135 HP; Trade Boat 200 → 120 HP; Watch/Sentry/Guard/Ballista towers +25–40 HP; Fortified Wall and Guard/Ballista towers +1 pierce armor.
- Techs: Short Sword tech removed (Short Swordsman free with Bronze Age); Jihad renamed Zealotry and its speed bonus cut from +0.3 to +0.11; Mysticism from ×2 HP to +20; Medicine 200 % → 100 %; Plow/Irrigation increased to +100/+125; Domestication 200 → 150 food; Architecture build-time discount made faster to research; Alchemy now +1 for all ranged incl. towers.
- The 2023 *Return of Rome* port went further (gates, garrison, Trade Carts, Market exchange, team bonuses, Urbanization, Conscription, City Watch, Theocracy, unit speeds rescaled ×0.83 to AoE2 values) but that is a different engine and outside this document's scope.

**Reception**: mixed (Metacritic 69). Praised: visuals, music, campaigns, pop cap. Criticised: pathfinding, no triggers, Microsoft Store exclusivity at launch, few modern RTS conveniences compared with AoE2:HD, and a launch delay from October 2017 to February 2018.

---

## 10. What Wars should borrow from AoE1

Wars is explicitly an AoE2-style game, so most of the core loop will come from AoE2 research. These are the things from the *first* game that are worth keeping, re-creating or deliberately avoiding.

1. **Four ages with a two-building gate.** "Any two buildings of the current age + food" is simpler to explain than AoE2's building-specific requirements and produces natural build-order decisions (Barracks+Dock vs Granary+Storage Pit). Keep the escalating cost curve (500 F → 800 F → 1,000 F + 800 G) and the 120/140/160 s timers as a baseline.
2. **Finite resources everywhere.** AoE1's "trees never regrow, mines run out, fish run out, only farms and trade are renewable" creates a real mid-game land grab. Wars should keep the map finite but give one or two renewable sinks (farms, trade) exactly as AoE1 did, and expose the decay-on-kill rule for hunted animals (it rewards grouping villagers).
3. **Additive damage model with 20 % building mitigation.** `max(1, (melee − melee_armor) + (pierce − pierce_armor) + bonuses) × elevation`, and buildings at ×0.2 with explicit "+140 vs buildings" siege bonuses. This is cheap to simulate deterministically in the `sim` crate, trivially explainable in a tooltip, and makes siege matter without percentage tables.
4. **Elevation modifiers (×1.5 ranged downhill, ×0.67 melee uphill).** Cheap to implement with the height map Wars already needs for isometric rendering, and it makes Highland-style maps strategic.
5. **Projectiles that can miss, plus a Ballistics tech that lets them lead targets.** Flight time + aim-at-last-position + an Iron Age tech that fixes it is a fantastic skill ceiling (dodging catapult boulders) and a satisfying upgrade. Keep friendly-fire on area damage.
6. **Priests with faith and "wololo".** Conversion that freezes upgrades, a visible faith bar, Monotheism to convert buildings, chariots/ships as natural counters, and a hard-counter civ (Macedonian 4× resistance). Wars' monks should keep the AoE1 range (10) rather than AoE2's shorter leash — the long-range priest duel is the memory.
7. **Civ design = bonuses + tech-tree holes, no unique units.** Sixteen civs with one shared roster is far cheaper to produce procedural art for than 13 civs × unique units, and AoE1 proves it still yields distinct playstyles (Persians without an Academy, Macedonians without a Temple). Wars can ship AoE2-style unique units later; start with the AoE1 model.
8. **Architecture sets per cultural group.** Four or five building skins shared by three civs each gives visual identity without an art explosion; map them onto Wars' procedural art generator.
9. **The 1×1 wall segment at 5 stone with pierce armor, plus towers with real attack.** AoE1 walls were the first in an RTS that actually held. Add gates (AoE1's lack of them is the thing nobody misses) but keep the per-tile cost model.
10. **Taunts by number and a made-up unit language.** Numbered chat taunts (1–25, with "Start the game already!" and "Hey, I'm in your town") are pure nostalgia and cost one afternoon of audio. A synthetic language sidesteps localisation and trademark issues exactly as it did for Ensemble.
11. **Cheat codes with silly units.** Resource codes (`pepperoni pizza`), the rocket car (`bigdaddy`), laser troopers, `steroids` as a dev-speed toggle. These double as debug tools for the sim and as a marketing hook; Wars should ship its own set under its own names.
12. **Scenario editor with data-driven victory conditions as the MVP, triggers later.** AoE1's 12-conditions-per-player editor shows that a surprising amount of campaign content can be built with no scripting. Wars' "triggers" milestone can start with AoE1's condition list (bring object to area, hold stockpile, reach age, exploration %, capture object, destroy player) and grow into AoE2-style triggers.
13. **Deterministic lockstep with 200 ms communication turns, commands scheduled two turns ahead, adaptive turn length, per-turn checksums.** Wars already has "one deterministic sim crate"; the "1500 Archers" recipe is still the correct netcode for an RTS with hundreds of units and a Rust server can relay commands rather than simulate. Keep an explicit "waiting for players" overlay that names the laggard — it is honest, and it is the memory.
14. **Standard victory with Wonder / Ruins / Artifacts countdowns.** Three map-objective victories with a 10-minute global timer and a world-wide warning message are a cheap source of drama and of comeback mechanics; Ruins (static, capture by sight) and Artifacts (mobile, capture by touch) are two different objective types for nearly no code.
15. **Things to avoid on purpose**: the 50 pop cap, no build queues, no idle-villager button, no rally points, no attack-move, farms that block movement, Granary/Storage Pit split, no gates, and 25 % tribute tax. Every one of these is something DE or RoR had to fix; Wars should start where DE ended.

---

## 11. Sources

- Wikipedia — Age of Empires (video game): https://en.wikipedia.org/wiki/Age_of_Empires_(video_game)
- Wikipedia — Age of Empires: The Rise of Rome: https://en.wikipedia.org/wiki/Age_of_Empires:_The_Rise_of_Rome
- Wikipedia — Age of Empires: Definitive Edition (section of the above article; also Microsoft Store/Steam release history)
- Age of Empires fandom wiki — main page, ages, civilizations, units, buildings, technologies, cheat codes, scenario editor, random maps, population, conversion, armor, trade, victory, taunts, patch 1.0a/1.0c, Definitive Edition:
  - https://ageofempires.fandom.com/wiki/Age_of_Empires
  - https://ageofempires.fandom.com/wiki/Age_of_Empires:_The_Rise_of_Rome
  - https://ageofempires.fandom.com/wiki/Age_of_Empires:_Definitive_Edition
  - https://ageofempires.fandom.com/wiki/Stone_Age · /Tool_Age · /Bronze_Age · /Iron_Age
  - https://ageofempires.fandom.com/wiki/Civilization_(Age_of_Empires) and the 16 individual civilization pages
  - https://ageofempires.fandom.com/wiki/Technology_(Age_of_Empires) · /Upgrade_(Age_of_Empires)
  - https://ageofempires.fandom.com/wiki/Villager_(Age_of_Empires) · /Farm_(Age_of_Empires) · /House_(Age_of_Empires) · /Granary_(Age_of_Empires) · /Storage_Pit_(Age_of_Empires) · /Town_Center_(Age_of_Empires) · /Market_(Age_of_Empires) · /Dock_(Age_of_Empires) · /Temple_(Age_of_Empires) · /Government_Center · /Wonder_(Age_of_Empires)
  - Individual unit pages (Clubman … Fire Galley), tower and wall pages
  - https://ageofempires.fandom.com/wiki/Cheat_code_(Age_of_Empires)
  - https://ageofempires.fandom.com/wiki/Scenario_Editor_(Age_of_Empires)
  - https://ageofempires.fandom.com/wiki/Random_map · /Population · /Conversion · /Armor · /Trade · /Victory · /Taunt · /Priest_(Age_of_Empires) · /Fishing · /Tree · /Gold_Mine · /Stone_Mine · /Berry_Bush
  - https://ageofempires.fandom.com/wiki/Patch_1.0a_(The_Rise_of_Rome)
- Telcontar's Empire of the Ages (fan reference with original 1997/RoR stat tables): http://artho.com/age/units.html · http://artho.com/age/stats/aoetable.html · http://artho.com/age/alltribes.html
- Paul Bettner & Mark Terrano, "1500 Archers on a 28.8: Network Programming in Age of Empires and Beyond", GDC 2001 / Gamasutra: https://www.gamedeveloper.com/programming/1500-archers-on-a-28-8-network-programming-in-age-of-empires-and-beyond
- Microsoft, "What's new in Age of Empires: Definitive Edition": https://www.ageofempires.com/news/whats-new-age-empires-definitive-edition-2/
- PCWorld review of the Definitive Edition: https://www.pcworld.com/article/3256271/age-of-empires-definitive-edition-review.html
- Game Developer, "Postmortem: Ensemble's Age of Empires" (March 1998): https://www.gamedeveloper.com/game-platforms/the-game-developer-archives-postmortem-ensemble-s-age-of-empires-
