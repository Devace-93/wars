# Age of Empires II — systems research for Wars

Research notes for the **Wars** project (an original, age-advancing RTS written in Rust).
Age of Empires II (AoE2) is the reference point for the *feel* of the genre: this document
collects how its systems work so that Wars can decide, deliberately, what to borrow, what to
simplify and what to leave out. Nothing here is copied verbatim; all numbers are summarised
from the sources listed at the end. Where a figure could not be confirmed from a primary or
secondary source it is marked **[unconfirmed]**.

Conventions used in this document:

- "Tile" = one terrain cell of the map grid. AoE2 distances (range, line of sight, speed) are
  all expressed in tiles.
- "Game second" = one second of simulation time at 1.0× speed. The in-game clock and the
  "years" of the Wonder countdown are derived from it.
- "AoK" = The Age of Kings (1999), "AoC" = The Conquerors (2000), "HD" = HD Edition (2013),
  "DE" = Definitive Edition (2019+), "UP" = UserPatch (community patch for AoC).

---

## Table of contents

1. [History, editions and reception](#1-history-editions-and-reception)
   - 1.1 Timeline of releases and expansions
   - 1.2 Reception
   - 1.3 Nostalgia: taunts
   - 1.4 Nostalgia: "wololo"
   - 1.5 Nostalgia: cheat codes
   - 1.6 Campaigns
   - 1.7 The iconic UI layout
   - 1.8 Default hotkeys
2. [Rendering and world model](#2-rendering-and-world-model)
3. [Simulation](#3-simulation)
4. [Random maps](#4-random-maps)
5. [Scenario editor, triggers, AI scripting, cheats](#5-scenario-editor-triggers-ai-scripting-cheats)
6. [Multiplayer and netcode](#6-multiplayer-and-netcode)
7. [Sound and feedback](#7-sound-and-feedback)
8. [Systems Wars will implement first](#8-systems-wars-will-implement-first)
9. [Sources](#9-sources)

---

## 1. History, editions and reception

### 1.1 Timeline of releases and expansions

AoE2 was developed by Ensemble Studios (Dallas, Texas) on the **Genie engine**, the 2D
sprite engine first built for Age of Empires (1997). Microsoft published it. The game has
been re-released twice (HD Edition in 2013 by Hidden Path Entertainment; Definitive Edition
in 2019 by Forgotten Empires, Tantalus Media and Wicked Witch, under World's Edge / Xbox Game
Studios) and has received more expansions than almost any other RTS.

The four ages (Dark → Feudal → Castle → Imperial), four resources (food, wood, gold, stone),
villagers that gather and drop off at depository buildings, monks that convert and carry
relics, and an eight-player cap have been constant across all editions.

| Year | Release | Developer | Civilisations added | Headline features |
|------|---------|-----------|---------------------|-------------------|
| 1999 (late Sept) | **The Age of Kings** (AoK) | Ensemble Studios | 13: Britons, Byzantines, Celts, Chinese, Franks, Goths, Japanese, Mongols, Persians, Saracens, Teutons, Turks, Vikings | Four ages; garrisoning; town bell; idle-villager button; unit stances and formations; gates; trebuchets; monks + relics; five campaigns; triggers in the scenario editor; recorded games; up to 8 players |
| 2000 (24 Aug) | **The Conquerors** (AoC) | Ensemble Studios | 5: Aztecs, Mayans, Huns, Koreans, Spanish (18 total) | Unique technologies for every civ; auto-reseeding farms; smart villagers (auto-gather after building a drop site); ship formations; ram garrisoning; Last Man Standing; Wonder Race, King of the Hill, Defend the Wonder modes; new maps (Arena, Ghost Lake, Mongolia, Nomad, Oasis, Salt Marsh, Scandinavia, Yucatán); friend-or-foe colours; locked teams; chat commands to allied AI; four campaigns |
| 2013 (9 Apr) | **HD Edition** | Hidden Path Entertainment | — | Widescreen, Steam multiplayer and workshop, improved AI; Metacritic 68 |
| 2013 (7 Nov) | **The Forgotten** | Forgotten Empires + SkyBox Labs | 5: Incas, Indians, Italians, Magyars, Slavs (23) | Grew out of the fan mod "Forgotten Empires"; population cap raised to 500; Ludicrous map size; new units (Condottiero, Genoese Crossbowman, Kamayuk, Slinger, Boyar, Magyar Huszar, Elephant Archer, Imperial Camel); campaigns Alaric, Sforza, Bari, Dracula, El Dorado (patch 3.3), Prithviraj, Battles of the Forgotten |
| 2015 (5 Nov) | **The African Kingdoms** | Forgotten Empires + SkyBox Labs | 4: Berbers, Ethiopians, Malians, Portuguese (27) | African architecture set; new game modes (Sudden Death, Capture the Relic); Feitoria; first HD campaigns with English voice acting (Tariq ibn Ziyad, Sundjata, Francisco de Almeida, Yodit) |
| 2016 (19 Dec) | **Rise of the Rajas** | Forgotten Empires + SkyBox Labs | 4: Burmese, Khmer, Malay, Vietnamese (31) | South-East Asian and Indian architecture sets; Battle Elephant regional unit; Imperial Skirmisher; campaigns Gajah Mada, Suryavarman I, Bayinnaung, Le Loi |
| 2019 (14 Nov) | **Definitive Edition** (DE) incl. *The Last Khans* | Forgotten Empires, Tantalus, Wicked Witch | 4: Bulgarians, Cumans, Lithuanians, Tatars (35) | 4K sprites, remastered soundtrack, zoom, spectator mode, ranked matchmaking, quick play, benchmark test, reworked AI (with the original and HD AIs still selectable), shift-queued villager tasks, auto-reseeding farms by default; campaigns Ivaylo, Kotyan Khan, Tamerlane, Pachacuti (replaces El Dorado), The Art of War tutorials; Metacritic 84 |
| 2021 (26 Jan) | **Lords of the West** | Forgotten Empires | 2: Burgundians, Sicilians (37) | First expansion with fewer than four civs; campaigns Edward Longshanks, The Grand Dukes of the West, The Hautevilles |
| 2021 (10 Aug) | **Dawn of the Dukes** | Forgotten Empires | 2: Bohemians, Poles (39) | Hussite Wagon, Houfnice, Obuch, Folwark; campaigns Algirdas and Kestutis, Jadwiga, Jan Zizka; co-op campaigns introduced mid-2021 |
| 2022 (28 Apr) | **Dynasties of India** | Forgotten Empires | 4: Bengalis, Dravidians, Gurjaras, Hindustanis (the old "Indians" split and renamed) (42) | Ratha, Urumi Swordsman, Thirisadai, Shrivamsha Rider, Chakram Thrower; campaigns Babur, Rajendra, Devapala |
| 2023 (16 May) | **Return of Rome** | Forgotten Empires | Romans (for AoE2); the whole AoE1 roster + Lạc Việt inside the DE client | AoE1 re-implemented as a mode of the AoE2 engine |
| 2023 (31 Oct) | **The Mountain Royals** | Forgotten Empires | 2: Armenians, Georgians (45 incl. Romans) | Persians reworked; Fortified Church, Mule Cart, Warrior Priest; campaigns Tamar, Ismail, Thoros II |
| 2024 (14 Mar) | **Victors and Vanquished** | Forgotten Empires (designed by Filthydelphia) | — | 19 standalone scenarios (14 rebalanced community scenarios + 5 new), no civs |
| 2024 (14 Nov) | **Chronicles: Battle for Greece** | Forgotten Empires | 3 (Chronicles-only): Achaemenids, Athenians, Spartans | Antiquity "Chronicles" mode; one 21-scenario campaign; LotW/DotD/DoI folded into the base game from this date |
| 2025 (6 May) | **The Three Kingdoms** | Forgotten Empires | 5: Shu, Wei, Wu, Jurchens, Khitans (50) | Hero units with abilities; Pastures (Khitans); Mounted Trebuchet; PS5 launch; campaigns Liu Bei, Cao Cao, Sun Jian; elevation raised to 16 levels in the same update cycle |
| 2025 (14 Oct) | **Chronicles: Alexander the Great** | Forgotten Empires | 3 (Chronicles-only): Macedonians, Thracians, Puru | Second Chronicles chapter |
| 2026 (17 Feb) | **The Last Chieftains** | Forgotten Empires | 3: Mapuche, Muisca, Tupi (53) | South American architecture set; Inca rework and reworked Pachacuti campaign (free); naval rework with two new ship lines; campaigns Lautaro, Pacanchique (branching), Arariboia |
| 2026 (22 Sept) | **The Viking Sagas** | Forgotten Empires | 3: Danes, Saxons, Varangians (56) | Jomsviking, Hearth Troop, Jarl; three campaigns around Harald Hardrada and 1066; cyan player colour throughout |

Other ports and side releases: Mac OS "Gold Edition" (2001), PlayStation 2 port by Konami
(2001–2002, no scenario editor), a J2ME mobile version (2005), Xbox One / Series (31 Jan
2023, with cross-play, a "Story mode" difficulty and controller UI), PlayStation 5 (May 2025)
and macOS via Steam (May 2026). The community UserPatch (UP 1.4/1.5) and Voobly kept the
AoC version alive for competitive play between 2010 and 2019, and much of UP's scripting
surface (RMS `effect_amount`, `up-*` AI commands) was later adopted by DE.

### 1.2 Reception

- Original release: Metacritic 92/100 (21 reviews). Reviewers singled out the sheer
  breadth of features, the *town bell*, the *idle villager button* and the civilisation
  voice lines; the main complaint was that units were visually hard to tell apart and that
  it felt like an iteration of the first game.
- Sales: two million copies shipped within three months; charts topped in seven countries;
  roughly 469k / 595k / 479k US copies in 1999 / 2000 / 2001; German double-platinum and
  UK ELSPA platinum awards.
- Awards: GameSpot Strategy Game of the Year 1999, AIAS Computer Game of the Year, and
  others. IGN ranked it 53rd best game of all time (2005); GQ 98th (2023).
- Influence: the engine and design were licensed or imitated in Star Wars: Galactic
  Battlegrounds (same Genie engine), Empire Earth and Cossacks.
- HD Edition: 68/100 (criticised for changing very little). Definitive Edition: 84/100
  (praised art and quality-of-life, criticised pathfinding at launch); DE triggered the
  2019–2020 resurgence that made AoE2 one of the most-watched RTS games on streaming
  platforms again.
- A security advisory in July 2026 patched a DE vulnerability that allowed placing files on
  another player's machine through multiplayer content transfer — relevant to Wars because
  it shows that *map/mod transfer* between peers is an attack surface.

### 1.3 Nostalgia: taunts

Taunts are short voiced messages. A player types the taunt number into the chat box and
every recipient hears the clip (and sees the number). Numbers 31–42 double as **commands to
allied computer players** since The Conquerors; DE added 43–105 so that the new AI can be
steered by taunts (report resources, change army composition, attack a flared position,
delete excess villagers, and so on).

The AoE2 taunt list (1–42, as shipped with AoK/AoC; DE kept the same lines):

| # | Line | # | Line |
|---|------|---|------|
| 1 | Yes. | 22 | Quit touching me! |
| 2 | No. | 23 | Raiding party! |
| 3 | Food please. | 24 | Dadgum. |
| 4 | Wood please. | 25 | Eh, smite me. |
| 5 | Gold please. | 26 | The wonder, the wonder, the... no! |
| 6 | Stone please. | 27 | You played two hours to die like this? |
| 7 | Ahh! | 28 | Yeah, well, you should see the other guy. |
| 8 | All hail, king of the losers! | 29 | Roggan. |
| 9 | Ooh! | 30 | Wololo. |
| 10 | I'll beat you back to Age of Empires. | 31 | Attack an enemy now. |
| 11 | (Herb laugh) | 32 | Cease creating extra villagers. |
| 12 | Ah! being rushed. | 33 | Create extra villagers. |
| 13 | Sure, blame it on your ISP. | 34 | Build a navy. |
| 14 | Start the game already! | 35 | Stop building a navy. |
| 15 | Don't point that thing at me! | 36 | Wait for my signal to attack. |
| 16 | Enemy sighted! | 37 | Build a wonder. |
| 17 | It is good to be the king. | 38 | Give me your extra resources. |
| 18 | Monk! I need a monk! | 39 | (Ally sound) |
| 19 | Long time, no siege. | 40 | (Enemy sound) |
| 20 | My granny could scrap better than that. | 41 | (Neutral sound) |
| 21 | Nice town, I'll take it. | 42 | What age are you in? |

Community usage that Wars should expect players to bring with them: `1`/`2` for yes/no,
`11` as "lol" (often spammed as `11 11 11`), `14` to ask to resume after a pause, `24` as an
exclamation of surprise, `30` (wololo) as pure celebration, and in DE `105` ("You can resign
again", considered bad manners) and `104` ("Don't resign!"). DE taunts 61–68 ("Attack
player 1! (Blue)" … "Attack player 8! (Orange)") fix the canonical player-colour order:
**blue, red, green, yellow, cyan, purple, gray, orange**. DE also added seasonal/ranked
reward taunts (106–122) and a hidden `205` that makes the AI resign earlier.

Design note for Wars: taunts are cheap to implement (number → audio clip + chat echo) and
are a large part of the social identity of the genre. The "commands to allied AI" idea
(31–42) is also a neat low-UI way of steering bots.

### 1.4 Nostalgia: "wololo"

"Wololo" is the chant of the Priest in Age of Empires I (sampled from a sound library);
the AoE2 Monk's chant is different, but the meme stuck to AoE2 anyway because taunt 30
plays the original clip. "Wololo" has become shorthand for *conversion* across the whole
RTS community (Age of Empires Online even named a staff after it). For Wars this suggests:
give the conversion mechanic a distinctive, looping audio cue; players will nickname it
immediately.

### 1.5 Nostalgia: cheat codes

Cheats are typed into the chat box (Enter, type the code, Enter). They are case-insensitive,
must be enabled in the lobby ("Allow cheats"), and spawned cheat units appear at the
player's Town Center (no TC → nothing spawns). Using any cheat disables achievements for
the session (HD/DE) and campaign medals (DE).

Resources:

| Code | Effect |
|------|--------|
| `cheese steak jimmy's` | +10,000 food (named after a Dallas lunch spot the Ensemble team frequented) |
| `lumberjack` | +10,000 wood |
| `robin hood` | +10,000 gold |
| `rock on` | +10,000 stone |
| `ninjaconnor`, `ninjalui`, `rowshep` | +100,000 of every resource (The Forgotten; named after babies born during its development) |

Map and misc:

| Code | Effect |
|------|--------|
| `marco` | reveal map (toggle) |
| `polo` | remove fog of war (toggle) |
| `aegis` | instant build/research/gather/train for all players, including AI (toggle) |
| `i r winner` | instant victory |
| `resign` | instant defeat |
| `black death` | kills all other players (buggy: historically only the first other player) |
| `wimpywimpywimpy` | commit suicide |
| `torpedo<N>` | destroy everything belonging to player N |
| `natural wonders` | take control of Gaia (irreversible) |
| `woof woof` | birds become flying "Stormy Dogs" |
| `!mute` / `!nomute` | silence / unsilence taunts |
| `noui` | hide the HUD (DE) |
| `going above and beyond` | research the same tech up to 256 times, stacking (DE, update 51737) |
| `tech tech one two free` | free technologies (DE, update 56005) |
| `my cpu can handle it` | population cap 1,000 (DE, update 107882) |
| `sharing is caring` | resource income shared with allies (DE, update 107882) |

Cheat units:

| Code | Unit | Notes |
|------|------|-------|
| `how do you turn this on` | Cobra Car | fast, tanky machine-gun car; the most famous cheat |
| `i love the monkey head` | VMDL | useless but very fast civilian |
| `to smithereens` | Saboteur | land demolition unit |
| `furious the monkey boy` | Furious the Monkey Boy | huge attack, tiny HP (AoC) |
| `i don't exist` | War Penguin | armoured infantry (HD) |
| `alpaca simulator` | Alfred the Alpaca | armoured cavalry (HD) |
| `catzor` | Sharkatzor | flying siege shark (Rise of the Rajas) |
| `photon man` | Photonman | improved hand cannoneer (DE) |

DE also has event "transformation" cheats (`yes we khan`, `put on your capes`,
`grab your pitchforks`, `cameleon`, `flemish reformation`) that re-skin existing units.

Design note for Wars: cheats are a cheap, well-loved feature that double as debug tools
(reveal map, instant build, infinite resources). Implement them as ordinary simulation
commands so that they are logged in replays and only work when the lobby allows them.

### 1.6 Campaigns

AoE2 campaigns are chains of hand-built scenarios with narrated slideshow intros, scouting
reports, hints, history pages and, since DE, bronze/silver/gold medals per difficulty.
The list (grouped by release):

- **AoK (1999):** William Wallace (Celts, tutorial), Joan of Arc (Franks), Saladin
  (Saracens), Genghis Khan (Mongols), Barbarossa (Teutons).
- **AoC (2000):** Attila the Hun (Huns), El Cid (Spanish/Saracens), Montezuma (Aztecs),
  Battles of the Conquerors (8 standalone scenarios).
- **The Forgotten (2013):** Alaric (Goths), Sforza (Italians), Bari (Byzantines), Dracula
  (Magyars/Slavs; renamed "Vlad Dracula" in DE), El Dorado (Spanish; removed in DE), Prithviraj
  (Indians/Hindustanis), Battles of the Forgotten.
- **The African Kingdoms (2015):** Tariq ibn Ziyad (Berbers), Sundjata (Malians), Francisco
  de Almeida (Portuguese), Yodit (Ethiopians).
- **Rise of the Rajas (2016):** Gajah Mada (Malay), Suryavarman I (Khmer), Bayinnaung
  (Burmese), Le Loi (Vietnamese).
- **DE / The Last Khans (2019):** Ivaylo (Bulgarians), Kotyan Khan (Cumans), Tamerlane
  (Tatars), Pachacuti (Incas), The Art of War (tutorial challenges), Historical Battles.
- **Lords of the West (2021):** Edward Longshanks (Britons), The Grand Dukes of the West
  (Burgundians), The Hautevilles (Sicilians).
- **Dawn of the Dukes (2021):** Algirdas and Kestutis (Lithuanians), Jadwiga (Poles),
  Jan Zizka (Bohemians).
- **Dynasties of India (2022):** Babur (Hindustanis/Tatars), Rajendra (Dravidians), Devapala
  (Bengalis).
- **The Mountain Royals (2023):** Tamar (Georgians), Ismail (Persians), Thoros II (Armenians).
- **Victors and Vanquished (2024):** 19 standalone scenarios (Ragnar Lothbrok, Otto the
  Great, Oda Nobunaga, …).
- **The Three Kingdoms (2025):** Liu Bei (Shu), Cao Cao (Wei), Sun Jian (Wu).
- **The Last Chieftains (2026):** Lautaro (Mapuche), Pacanchique (Muisca, branching
  narrative), Arariboia (Tupi); reworked Pachacuti.
- **The Viking Sagas (2026):** three campaigns following Harald Hardrada's Mediterranean
  years, Danish coastal raids and the 1066 invasion of England.
- **Chronicles (2024, 2025):** Battle for Greece (21 scenarios) and Alexander the Great,
  in the separate antiquity mode.

DE counts 46 "classic" campaigns, not counting The Art of War, Historical Battles, Victors
and Vanquished and Chronicles. Every AoK campaign includes at least one scenario where the
player cannot build an economy ("fixed force"), which is a good template for tutorial pacing.

### 1.7 The iconic UI layout

The AoE2 screen has a thin top bar and a tall bottom bar; the map fills everything in
between. Players remember the exact placement of each element, so Wars should either
reproduce it or deviate consciously.

Top bar (left → right):

1. **Resource stockpile**: wood, food, gold, stone, then **population** (`current/cap`).
   In DE the **idle villager button** sits right next to the population counter (in
   AoK/AoC it sat beside the minimap); the number on it shows how many villagers are idle.
2. **Current age** with the age-advance progress bar when researching; in DE clicking the
   age icon jumps to a Town Center.
3. `F11` toggles the game clock / speed / difficulty readout (and FPS in DE).
4. Right side: Objectives (campaign goals or lobby settings), Tech tree, Chat, Diplomacy,
   Options, Menu. Wonder/relic/monument countdowns appear under the top-right corner,
   counted in **years**.
5. Chat and event messages scroll in the top-left area of the map.

Bottom bar (left → right):

1. **Command panel** (bottom-left): a 5×3 grid of buttons. Villagers: build economic /
   build military / repair / garrison / stop / delete. Military: stances (aggressive,
   defensive, stand ground, no attack), formations (line, box, staggered, flank), guard,
   follow, patrol, attack ground (siege), garrison, delete, stop. Buildings: train/research
   queue buttons, set gather point, town bell (TC), garrison/ungarrison. Trebuchets: pack /
   unpack. Monks: convert / heal. Transports: unload.
2. **Selected unit panel** (bottom-centre): portrait, name, HP bar, attack/armour/range
   stats, and the production queue; with several units selected it becomes a grid of
   portraits (up to 40, 60 in HD/DE).
3. **Minimap** (bottom-right): square minimap with buttons for flare (signal allies),
   military-only / economy-only / all filters, and `F4` score display above it. Wonders are
   small white squares; player units use the player colour.

### 1.8 Default hotkeys

AoK/AoC/HD used mnemonic letters; DE switched to a **grid ("QWERTY") layout** where the
hotkey is the key at the same physical position as the button (Q W E R T / A S D F G /
Z X C V B). Both are listed because players ask for both.

Classic (AoK–HD) hotkeys:

| Purpose | Key |
|---------|-----|
| Build menu (economic) / build menu (military) | `B` / `V` |
| House / Mill / Farm / Lumber camp / Mining camp | `E` / `I` / `F` / `Z` / `G` |
| Town Center / Dock / Market / Blacksmith / Monastery / University | `N` / `D` / `M` / `S` / `Y` / `U` |
| Barracks / Archery Range / Stable / Siege Workshop / Castle | `B` / `A` / `L` / `K` / `V` |
| Outpost / Palisade wall / Stone wall / Gate / Watch tower / Bombard tower / Wonder | `Q` / `P` / `W` / `/` / `T` / `J` / `O` |
| Go to Town Center | `H` (cycles through TCs) |
| Go to building X | `Ctrl` + build letter (e.g. `Ctrl+B` barracks, `Ctrl+A` archery range, `Ctrl+L` stable, `Ctrl+K` siege workshop, `Ctrl+V` castle, `Ctrl+M` market, `Ctrl+D` dock, `Ctrl+Y` monastery, `Ctrl+S` blacksmith, `Ctrl+U` university) |
| Train villager (TC) | `C` |
| Militia line / Spearman line / Eagle line | `S` / `E` / `G` |
| Archer line / Skirmisher line / Cavalry archer / Hand cannoneer | `A` / `R` / `C` / `E` |
| Scout line / Knight line / Camel line | `T` / `N` / `C` |
| Ram / Mangonel / Scorpion / Bombard cannon | `R` / `A` / `N` / `C` |
| Unique unit / Petard / Trebuchet (Castle) | `T` / `P` / `R` |
| Monk | `T` |
| Next idle villager / previous | `.` / `Shift+.` |
| Next idle military / previous | `,` / `Shift+,` |
| Select all idle villagers / all idle military | `Ctrl+Shift+.` / `Ctrl+Shift+,` |
| Control group assign / select / add / centre on | `Ctrl+1..9` / `1..9` / `Shift+1..9` / `Alt+1..9` |
| Centre on selection | `Space` |
| Return to previous view | `Backspace` (AoC) |
| Attack move / Patrol / Guard / Follow | `A` (DE: `R`) / `Z` / `G` / `F` (varies by version) |
| Stances: aggressive / defensive / stand ground / no attack | `Q` / `W` / `E` / `R` (post-HD) |
| Stop / Delete / Delete all selected | `G` or `S` / `Delete` / `Shift+Delete` |
| Chat / allied chat prefix / enemy-only / everyone | `Enter` / `;` / `!` / `*` |
| Diplomacy / Objectives / Tech tree / Score toggle | `Ctrl+D` or menu / `Ctrl+O` / `F5` / `F4` |
| Pause / Save / Clock readout | `F3` / `F12` / `F11` |
| Shift-queue (units, waypoints, farms) | hold `Shift` |
| Garrison | `Alt`+right-click or `T` then click |
| Town bell | `B` (TC selected) |

DE grid layout (defaults):

| Purpose | Key |
|---------|-----|
| Economic build screen / military build screen (villager selected) | `Q` / `W` |
| House / Mill / Mining camp / Lumber camp / Dock / Farm / Blacksmith / Market / Monastery / University / Town Center / Wonder / Feitoria | `Q` / `W` / `E` / `R` / `T` / `A` / `S` / `D` / `F` / `G` / `Z` / `X` / `C` |
| Barracks / Archery Range / Stable / Siege Workshop / Castle / Outpost / Palisade wall / Palisade gate / Stone wall / Gate / Tower / Bombard tower | `Q` / `W` / `E` / `R` / `C` / `A` / `S` / `X` / `D` / `Z` / `F` / `G` |
| Go to Town Center | `H` |
| Go to building X (cycle) / select all of type | `Ctrl`+letter / `Ctrl+Shift`+letter (A archery range, B barracks, S blacksmith, V castle, D dock, Z lumber camp, M market, I mill, G mining camp, Y monastery, K siege workshop, L stable, U university) |
| Cycle idle villagers / select all idle villagers | `.` / `Shift+.` |
| Cycle idle military / select all idle military (again for ships) / visible idle military | `,` / `Shift+,` / `Alt+,` |
| Select all military buildings | `Ctrl+Shift+Space` |
| Go to last notification | `Home` or mouse button 3 |
| Attack move / Attack ground / Garrison / Ungarrison / Unload | `R` / `T` / `T` / `B` / `Q` |
| Drop off resources / Seek shelter (villagers) | `A` / `F` |
| Line formation / Flank formation | `Z` / `V` |
| Pack / Unpack trebuchet | `W` / `Q` |
| Control groups 1–10 / 11–20 / append / ungroup / select+snap | `Ctrl+N` / `Ctrl+Alt+N` / `Shift+N` / `U` / `Ctrl+Shift+N` |
| Stop / Delete / Delete all | `G` / `Delete` / `Shift+Delete` |
| Tech tree / clock modes / chat history | `F5` / `F11` / `PageUp`,`PageDown` |
| Spectate player N | `Ctrl+Shift+F<N>` (`F9` = Gaia) |
| Zoom in / out | `Ctrl++` / `Ctrl+-` |
| Runaway move (ignore formation) | `Ctrl`+right-click (update 153015) |

Scenario editor hotkeys (not rebindable): `Ctrl+M/T/P/U/D/V/O/W/C/R` jump to the
Map/Terrain/Players/Units/Diplomacy/Victory/Options/Messages/Cinematics/Triggers tabs;
`Ctrl+G` cycles object placement restrictions (grid snap, allow illegal placement);
`Ctrl+A` shows every object's selection box.

Design note for Wars: ship **both** a mnemonic and a grid profile, make the idle villager
key (`.`) and go-to-TC (`H`) sacred, and support control groups 1–9 with `Ctrl`/`Shift`/
`Alt` semantics exactly as above — these are muscle memory for the target audience.

---

## 2. Rendering and world model

### 2.1 Isometric diamond tiles

- The Genie engine draws the world as a **2:1 isometric diamond grid**. Each terrain tile
  sprite is **97 px wide by 49 px tall** (a 49-row diamond whose widest row, the 25th, is 97
  pixels) at the original resolution; DE ships 2× and 4× sprite sets but keeps the same
  logical geometry. openage's terrain notes document the 97/49 diamond and the mirrored
  half-tile layout.
- Terrain SLP files store 100 variant tiles (10×10) per terrain type; the variant used at map
  position (x, y) is `(x mod 10) + (y mod 10) * 10`, which gives cheap non-repeating texture.
  Farm terrains use 36 or 9 tiles.
- Terrain blending between adjacent different terrains is driven by *blendomatic* masks and
  a per-terrain priority (dirt < grass < water < …); a tile with a lower priority is painted
  first and the higher one is masked in over it.
- Sloped tiles are generated at load time by texture-mapping the flat tile onto one of the
  slope shapes and relighting it (brighter on the sunward slope, darker on the opposite one)
  using pre-computed "inverse colour maps" (ICMs) — a palette-era trick that Wars can replace
  with a per-vertex light factor.
- World axes: openage calls them NE and SE (the two diamond diagonals) plus UP. The physics
  coordinate `(0.5, 0.5)` is the centre of tile `(0, 0)`; openage stores positions as
  **fixed-point integers with a 16-bit fraction**. The original game stores unit positions as
  32-bit floats in the command stream (movement commands carry float x/y targets), but
  waypoints are tile-integers only.
- Screen conversion: `screen_x = (x - y) * 48`, `screen_y = (x + y) * 24 - elevation * h`
  for a 97×49 tile (half-width 48, half-height 24, rounded); openage uses
  `UP / sqrt(8)` as the elevation scale for a visually matching slope.
- Draw order: tiles are painted back-to-front by `(x + y)`; sprites are sorted by their
  ground y position so that a unit behind a building is covered by it. Buildings have a
  separate "shadow" and "snow/foundation" layer.

### 2.2 Map sizes

All AoE2 maps are square. Dimensions in tiles:

| Name (DE UI) | Legacy RMS constant | DE RMS constant | Side | Tiles | Recommended players |
|---|---|---|---|---|---|
| Miniature* | — | `MAPSIZE_MINI` | 80 | 6,400 | 1 |
| Tiny | `TINY_MAP` | `MAPSIZE_TINY` | 120 | 14,400 | 2 |
| Small | `SMALL_MAP` | `MAPSIZE_SMALL` | 144 | 20,736 | 3 |
| Medium | `MEDIUM_MAP` | `MAPSIZE_MEDIUM` | 168 | 28,224 | 4 |
| Normal | `LARGE_MAP` | `MAPSIZE_NORMAL` | 200 | 40,000 | 6 |
| Large | `HUGE_MAP` | `MAPSIZE_LARGE` | 220 | 48,400 | 8 |
| Huge (was "Giant" before update 107882) | `GIGANTIC_MAP` | `MAPSIZE_HUGE` | 240 | 57,600 | 8 |
| Giant* | — | `MAPSIZE_GIANT` | 252 | 63,504 | — |
| Massive* | — | `MAPSIZE_MASSIVE` | 276 | 76,176 | — |
| Enormous* | — | `MAPSIZE_ENORMOUS` | 300 | 90,000 | — |
| Colossal* | — | `MAPSIZE_COLOSSAL` | 320 | 102,400 | — |
| Incredible* | — | `MAPSIZE_INCREDIBLE` | 360 | 129,600 | — |
| Monstrous* | — | `MAPSIZE_MONSTROUS` | 400 | 160,000 | — |
| Ludicrous ("LudiKRIS", from The Forgotten) | `LUDIKRIS_MAP` | `MAPSIZE_LUDICROUS` | 480 | 230,400 | — |

`*` = only visible with the `MORE_MAP_SIZES` launch option (added March 2024). Note the
legacy naming mismatch: the lobby's "Normal" is the script's `LARGE_MAP` and the lobby's
"Large" is `HUGE_MAP`. RMS scaling (`set_scale_by_size`, `set_scaling_to_map_size`) is by
**area** relative to a 100×100 reference, not by side length. Scripts may override the
size with `override_map_size N` (36–480).

### 2.3 Elevation and cliffs

- Terrain height is per-tile-corner; the original game had **7 elevation levels** (1–7).
  DE update 141935 (April 2025) extended this to **16 levels**.
- Slopes are gentle: the engine only allows one level of difference between neighbouring
  tiles, so a hill is a staircase of rings. Water and some other terrains cannot sit on a
  slope and are replaced by grass if painted there.
- **Combat modifier:** attacking from higher ground gives **+25 % damage**, attacking uphill
  **−25 %**. The bonus is binary (any height difference, or being above a cliff, triggers it),
  applied after armour subtraction. Civ bonuses modify it (Tatars +50 % instead of +25 %,
  building-side bonuses for some civs, etc.).
- **Building placement:** 1×1, 2×2 buildings, farms, walls and gates can sit on any slope;
  3×3 and larger need flat ground (with a few corner exceptions); Town Centers and Trade
  Workshops must be entirely flat.
- **Cliffs** are objects, not terrain: six visual types in DE (Granite, Desert, Snow, Marble,
  Limestone, Terraced) that cannot connect to each other. They are impassable, block
  building, and count as "higher ground" for the units on top.

### 2.4 Terrain types

AoK shipped with roughly 30 terrains; DE's scenario editor lists about 100. They fall into
three categories (a terrain can belong to more than one):

- **Land**: walkable and buildable — Grass 1/2/3, Dirt 1/2/3/4, Dirt Mud, Dirt Savannah,
  Desert Sand, Desert Cracked, Gravel, Road / Road Broken / Road Fungus / Road Gravel,
  Snow, Snow Light / Strong, Ice (walkable, not buildable), Grass Jungle, Grass Dry,
  Grass Flowers, Pasture, Rice Farm, Farm (+ 0/33/67 % growth stages and "Dead" variants),
  Beach variants (buildable only for walls/docks), Shallows (walkable *and* navigable),
  Swamp Bogland, Quicksand (walkable, unbuildable), Black (the void).
- **Forest**: painting it spawns tree objects automatically on an underbrush terrain —
  Oak, Pine, Palm (desert and grass variants), Jungle, Bamboo, Lush Bamboo, Rainforest,
  Mangrove (on shallows), Birch, Spruce, Autumn, Dead, Baobab, Acacia, Dragon Tree,
  Mediterranean, Reeds, Bush, plus snow variants. The tree objects are what block movement.
- **Water**: navigable only — Water Shallow (docks allowed), Water Medium, Water Deep,
  Water Deep Ocean, Water Azure / Brown / Green / Yellow / Weeds, Water 2D Shoreless,
  Ice Navigable.

Each terrain carries: a texture, an ambient sound ID, movement passability per unit class
("terrain restrictions" — land, water, amphibious, flying), a buildability flag, and a
blending priority. Since DE, **terrain layering** lets a purely visual terrain be painted
over a base terrain; the base supplies the gameplay properties.

### 2.5 Fog of war and line of sight

Each player holds a per-tile visibility map with three states:

1. **Unexplored** — drawn black; nothing is known (the minimap is black too).
2. **Explored** (fog) — the terrain and the *last seen* static objects (buildings, trees,
   mines) are drawn dimmed; units are not shown; buildings destroyed since are still drawn
   until the tile is revisited.
3. **Visible** — within the line of sight (LOS) of at least one own (or shared) object;
   drawn at full brightness and updated every frame.

Rules:

- Every unit and building has a **Line of Sight radius in tiles** (circular, measured from
  the object's centre; the edge is rasterised with the `blkedge.dat` masks so that the fog
  boundary looks rounded).
- Representative LOS values: Villager 4, Militia line 4 (+2 in Feudal), Scout Cavalry 4
  (+2 per age), Archer 6, Knight 4, Monk 11, Mangonel 9, Trebuchet 19, Galley 7, Town Center
  8 (becomes larger with Fletching/Bodkin/Bracer and Town Watch/Patrol), Outpost 12, Towers
  10, Castle 11. Upgrades that add range (+1 range) also add +1 LOS; Town Watch / Town Patrol
  add +4 LOS to buildings.
- LOS is not blocked by obstacles (there is no occlusion); elevation does not change LOS.
- **Being shot reveals the shooter**: an enemy ranged unit that hits you is briefly added to
  your visibility even if it is outside your LOS.
- **Cartography** originally was a Market technology that shared allies' LOS; HD patch 5.7
  made it automatic once a Market stands; DE update 73855 made allied vision shared from the
  start of the game. **Spies** (Imperial, cost scales with enemy villager count) reveals all
  enemy LOS; **Treason** (Regicide) flashes the kings' positions.
- Lobby options: Reveal Map = Normal / Explored (whole map in fog from the start, the
  default on Black Forest) / All Visible; "Shared Exploration" for teams.
- Gaia objects (herdables, relics, huts) placed as Gaia convert automatically to the first
  human player whose LOS touches them (this is how sheep are captured).

### 2.6 Unit facing and animation

- Units are rendered from **8 facings** (S, SW, W, NW, N, NE, E, SE in sprite order). The
  SLP graphics only store **5** of them (south through north); the eastern three are
  horizontal mirrors — which is why a unit's weapon switches hands when it turns. Buildings
  have a single facing (houses and some decorations use their rotation to pick a variant).
- Each unit *action* (idle, walk, attack, die, decay, carry-food, carry-wood, build, …) is a
  separate graphic with its own frame count and frame rate. Typical AoC values: walking 10
  frames per facing at ~0.1 s per frame, attacking 10–15 frames, dying 10–15 frames, idle
  1–10 frames. Projectile-firing units have an "attack delay" (the frame at which the
  projectile is spawned) that defines their reaction time.
- Villagers switch graphic sets when they switch task (lumberjack, hunter, farmer, miner,
  builder, shepherd…) — in the engine this is implemented as *replacing* the unit with a
  task-specific variant that shares the ID line.
- Shadows are baked into the sprite; player colour is a fixed palette range (8 palette
  entries per player, remapped per player at draw time; DE uses a colour mask texture).
- Sprites are sorted by ground position for overlap; a unit's selection ellipse ("outline")
  and HP bar are drawn in HUD space from its physics position.

### 2.7 Team colours

Eight player slots with fixed colour order: **1 blue, 2 red, 3 green, 4 yellow, 5 cyan,
6 purple (magenta/pink), 7 gray, 8 orange**. Gaia (player 0) uses a distinct colour
(teal/white depending on the game). AoC added **friend-or-foe colours** (self blue, allies
yellow, enemies red, neutral gray — judged by *your* stance toward them), and DE lets
players reorder/replace palette colours client-side. The minimap uses the same colours;
team-game minimaps can be reduced to two colours (blue allies, red enemies).

---

## 3. Simulation

### 3.1 Game speeds and time units

- The simulation advances in **game seconds**. Every rate in the data files (gather rate,
  reload time, train time, movement speed in tiles/second, build time) is expressed in game
  seconds, so the game speed setting is a pure multiplier between real time and game time.
- Classic speeds: **Slow 1.0×, Normal 1.5×, Fast 2.0×**. Competitive play has always used
  Fast (sometimes called "1.7" or "2.0" depending on the version's naming). DE's lobby
  offers **Slow 1.0×, Casual 1.5×, Normal 1.7×, Fast 2.0×**; ranked uses Normal (1.7×).
  "Lock speed" prevents players from changing it in-game.
- The clock shown with `F11` is game time; recorded games and statistics use game time.
- The Wonder/relic countdown is shown in **years**: 200 years = 16 min 40 s at 1.0×, i.e.
  **1 year = 5 game seconds**.
- The original engine ran the simulation in a variable-length update loop (the networking
  article notes simulation steps fluctuating by up to 200 ms depending on what the player
  was looking at) and derived distances moved from the elapsed time of each step — this is
  one of the reasons sync bugs were painful. Wars will use fixed ticks instead (see §8).

### 3.2 Unit movement: tiles with sub-tile positions

- The map is a tile grid, but units live at **continuous positions inside tiles**. The
  original engine used floats (move commands transmit float x/y); openage re-implements the
  same semantics with fixed-point values with a 16-bit fraction. Buildings occupy whole
  tiles (1×1 house, 2×2 mill/lumber camp/tower, 3×3 barracks, 4×4 town center/castle,
  5×5 wonder).
- Speed is in tiles per game second, clamped by the engine to 0.5–7.5. Reference speeds:
  infantry 0.9–1.0 (Militia line 0.9, Spearman 1.0, Eagle 1.1–1.3), Villager 0.8, Archer
  0.96, Knight 1.35, Scout line 1.2–1.5, Cavalry Archer 1.4, Monk 0.7, Trade Cart 1.0,
  Mangonel 0.6, Trebuchet 0.8 packed, Battering Ram 0.5, Galley 1.43, Fire Ship 1.35,
  Transport 1.45. Imperial techs (Husbandry +10 % cavalry, Squires +10 % infantry,
  Wheelbarrow/Hand Cart +10 % villagers each, Dry Dock +15 % ships, Fervor +15 % monks)
  are additive percentages on the base.
- A unit turns to face its heading before moving (openage's move system mirrors this:
  "turned first, then moved"); turning is instantaneous for most units, slow for ships
  and some siege (a rotation speed field exists in the data).
- Movement commands carry a target position; the unit asks the pathfinder for a route
  and follows it waypoint by waypoint, re-planning when blocked.
- **Shift-click waypoints**: up to any number for plain moves (flags stop being drawn after
  10), 10 for patrol/attack-move; waypoint coordinates are tile integers, not sub-tile.
- Ships use a separate passability class (water terrains) and a slower turning rate; the
  Transport Ship carries 5–20 units and unloads on shore.

### 3.3 Pathfinding, collision, groups and formations

The reverse-engineering notes collected by openage (from Ensemble's own postmortem and
Dave Pottinger's GDC 2000 talk) describe **three pathfinders and two obstruction
systems**:

1. A **long-range pathfinder** that works on a mip-mapped passability map (the map is
   subdivided by passability; moving objects are ignored, buildings are included and the
   mip-map chains are updated whenever a building is placed or destroyed).
2. A **tile-based short/medium range pathfinder** (A* on tiles) used for most moves.
3. A **polygonal pathfinder** that builds convex hulls around obstacles dynamically, used in
   crowded situations (combat, units crossing paths).

Collision:

- Every unit has a **collision radius** (a small rectangle/circle in the data: ~0.2–0.3 tiles
  for infantry, ~0.4 for cavalry, larger for siege and ships). Units cannot overlap; when two
  units' paths collide, one waits or sidesteps. Buildings and trees are hard obstacles;
  units can walk through allied *moving* units in some versions (DE lets units push through
  stationary friendly units after a short wait).
- The infamous "unit stuck behind a tree" and "villagers oscillating at a gold pile" bugs
  come from this layered design; DE spent years tuning it, and openage replaced it entirely
  with flow fields (see §6.5).

Group moves:

- Selecting several units and ordering a move makes them a **formation group**: the group
  moves at the speed of its slowest member, members are assigned slots in the formation
  shape, and the formation rotates to face the direction of travel.
- Four formations (AoK): **Line** (wide and shallow; default), **Box** (weak units inside),
  **Staggered** (spread out against splash), **Flank** (two subgroups that try to envelop).
  The formation re-flows when units die, which is where most "formation dance" jank comes
  from.
- Slot order (front to back): cavalry → infantry → archers/mounted archers → gunpowder
  and scorpions → monks, siege, trade carts, kings. Villagers and fishing ships never join
  formations. For long moves (> 30 tiles since update 107882; 10 before) the line turns into
  a column.
- When a formation hits an obstacle it splits around it and re-forms on the other side.
- Ship formations exist since AoC (demolition → fire ships → archer ships → cannon galleons
  / transports).
- Ensemble's Matt Pritchard has said that detailed formation controls existed in development
  but were cut as "too complicated for the speed of the game".

Selection and targeting helpers (from the openage notes): single-click selection uses a
tolerance box about one unit wide/high; own units win ties over enemy units; box selection
fills the 40-unit (60 in HD/DE) queue top-to-bottom; double-click selects all visible units
of the same *line* (so Militia and Champions can share a selection); `Ctrl+click` on a
portrait removes a unit, `Shift+click` keeps only that type.

### 3.4 Gather / drop-off loop

Villagers run a simple state machine:

1. **Task**: right-click a resource (tree, bush, mine, animal, farm, fish). The villager
   switches to the matching variant (lumberjack, forager, miner, hunter, farmer) and walks
   to the resource.
2. **Gather**: a per-task gather rate in units per game second (base, before techs):
   berries 0.31, farm 0.32 (fixed per-farm rate regardless of distance), sheep 0.33,
   deer/boar ~0.41 (meat decays: 1 food per 2 s on a dead animal), wood 0.39, gold 0.38,
   stone 0.36, shore fish 0.43 (villager). Carry capacity 10 (Wheelbarrow +25 %, Hand Cart
   +50 %; farmers +1 per Supplies-type techs in some versions).
3. **Drop-off**: when the carried amount reaches capacity, the villager walks to the
   nearest drop site of the right type (Town Center accepts everything; Mill = food;
   Lumber Camp = wood; Mining Camp = gold/stone; Dock = fish; Folwark/Feitoria/etc. are
   civ variants), deposits, and walks back to the **same** resource (or the nearest of the
   same type if it is exhausted).
4. Effective rate = gather rate × (gather time / (gather time + walk time)) — this is why
   drop-site placement is the whole early-game economy.
5. **Smart villagers** (AoC): after finishing a drop-site building they automatically start
   gathering the nearest resource of that type; after a farm is exhausted the farm is
   **reseeded automatically** if the player queued farms at the Mill (DE: auto-reseed is
   the default); Villagers that kill a boar start eating it.
6. **Idle detection**: a villager with no task (resource exhausted and nothing nearby,
   building finished with no follow-up, blocked) raises the **idle villager** flag; the
   idle button / `.` hotkey cycles through them. Since the Villager's `idle` state is
   literally "no current action", units that are walking to a drop-off are *not* idle.
7. Herdables (sheep, goats, turkeys, cows…) are Gaia units that convert to whoever sees
   them first; they can be *stolen* by walking a scout next to them. Huntables (deer) flee;
   boars/elephants fight back and must be lured. Relics generate gold when garrisoned in a
   Monastery: 0.5 gold/s per relic (1 every 2 seconds, so 10 relics = 5 gold/s).
8. Trade: Trade Carts (Market→allied Market) and Trade Cogs (Dock→allied Dock) earn gold
   proportional to the straight-line distance between the two buildings; Caravan makes
   them 50 % faster.
9. Market buy/sell changes prices by a fixed step per transaction (and prices drift back);
   tribute between players costs a 30 % fee (Coinage lowers it, Banking removes it).

### 3.5 Build and repair

- Multiple villagers speed construction with diminishing returns:
  `time = 3 × base_time / (n + 2)` — one villager builds in `base_time`, two in 75 %, three
  in 60 %, five in 43 %. Construction progress is independent of the foundation's HP (HP
  rises with progress, but an attacked foundation does not slow down).
- Building costs are paid when the foundation is placed; cancelling refunds everything.
  A **foundation** at < 100 % is a special state: it cannot be garrisoned, converted or
  (until a few % in) used to block pathing — walls, in contrast, block as soon as they are
  placed, which is what makes "quick-walling" possible.
- **Repair** (villagers only; buildings, siege and ships): costs **half of the original
  cost, charged proportionally while repairing** (repairing 50 % of HP costs 25 % of the
  build cost). Town Center repairs cost no stone but double wood. The first villager repairs
  **750 HP/min**, every additional villager adds **375 HP/min**; siege weapons and ships
  repair at a quarter of that (187.5 / 93.75 HP/min). Villagers can repair from two tiles
  away. Running out of resources stops the repair and the villager goes idle.
- Farms are a special building: 60 wood, finite food (175 base, raised by Horse Collar /
  Heavy Plough / Crop Rotation), one villager per farm (two for Khitan pastures), reseed
  queue at the Mill.
- Walls: 1×1 segments auto-connect; a Gate is a 1×4 object that opens for allies and locks
  via a toggle command; building walls through a drag places a chain of foundations.

### 3.6 Garrison and arrows

- Town Centers hold 15 (Teutons 25), towers 5 (Teutons 10), Castles 20, Donjons 10,
  Kreposts 20, Houses 5 (Khmer only), Docks 5 fishing ships (Gurjaras), rams 4 infantry,
  siege towers 10, transports 5–20. Production buildings can hold 10 units only at the
  moment they are trained (gather point set on the building).
- Garrisoned units are invulnerable and heal slowly (Castle 0.2 HP/s, TC/towers 0.1 HP/s;
  Herbal Medicine ×6). Horses, camels and most siege cannot garrison.
- **Arrows**: a building's projectile count grows with the garrisoned *pierce* damage:
  `extra_arrows = floor(Σ(unit pierce DPS) / building DPS)`, where a villager counts as a
  fixed 2.5 pierce DPS and melee units add nothing (except Teutons with Crenellations).
  Default/maximum arrow counts: Town Center 0/10 (attack 5), Castle 5/20 (attack 11),
  Tower 1/5 (attack 5/7/8), Bombard Tower 1/4 (only gunpowder units add, with a doubled
  threshold for the first extra shot). All projectiles target the same victim; extra arrows
  deal reduced damage.
- Units are **ejected** when the building is destroyed or converted, and cannot garrison
  (or stay) in a building at ≤ 20 % HP; ejected units take no damage. Siege units eject
  their crew too. Rams move faster (+0.05 tiles/s per infantry) and hit harder (+10 vs
  buildings per unit) when garrisoned.
- **Town bell**: range 25 tiles from the Town Center's centre; villagers go to the nearest
  garrisonable building that is within 25 tiles of *both* the bell and the villager and
  has room (closest villagers get the slots). Ringing again sends everyone back to their
  previous task ("All back to work" / "Seek shelter" are DE's per-building and per-villager
  variants).

### 3.7 Conversion (monks)

- Monks (Castle Age, 100 gold) convert enemy **units** at range **9** (12 with Block
  Printing) and, with **Redemption**, siege and most buildings — rams, elephants,
  trebuchets and buildings must be converted at touch range. Not convertible: kings,
  heroes, other religious units (unless **Atonement**), Town Centers, Castles, Kreposts,
  Monasteries, walls, gates, Wonders, farms, fish traps, and *foundations*.
- Conversion is an **interval-based lottery**. Every ~1.25 game seconds (a "conversion
  interval") the monk rolls: the first intervals are a guaranteed-fail warm-up (commonly
  cited as the first 4), then each interval has a fixed success chance (commonly cited as
  ~28–38 % for a standard unit **[exact base chance unconfirmed; sources differ]**), and
  success is forced at a maximum interval (commonly cited as the 10th). Buildings roll from
  the 15th to the 25th interval at 13 %. **Faith** (+4 min, +4 max), the Teuton team bonus
  and the scout line's innate resistance (resistance level 2 halves the chance; buildings
  level 3) stretch the window; **Heresy** makes converted units die instead.
- Several monks on one target roll independently; any success converts, and *all* of them
  spend their faith unless **Theocracy** was researched.
- A monk needs **100 % faith** to start a conversion. Faith drains to zero on success and
  regenerates slowly (Illumination shortens the rest ~47 %). Moving the monk resets the
  interval counter; retargeting from unit to unit does not.
- Converted units are **frozen at the tech level** they had (no future upgrades apply),
  keep their civ-specific stats, and lose train/build abilities the new owner does not have.
  Converted buildings produce normal units for the new owner; garrisoned units pop out
  un-converted.
- Audio: the converting player hears the chant; the victim hears the same chant with a
  low drone and gets the "unit being converted" alarm.
- Scouts, light cavalry and eagles do bonus damage to monks; monks are the canonical
  "glass counter" to expensive units and siege.

### 3.8 Trebuchet pack / unpack

- Built at the Castle (Imperial): 200 wood 200 gold, 150 HP, 200 attack, range 16 (17
  with Siege Engineers, 19 for Tatars with Timurid Siegecraft), minimum range 4, reload
  10 s, projectile flight ~4–5 s, accuracy 15 % against units (100 % against buildings).
- Two forms with separate graphics and stats: **packed** (movable, 0.8 speed, no attack)
  and **unpacked** (immobile, attacks). Switching takes **~11 s** (2.8 s for Japanese
  with Kataparuto) and can be cancelled at any moment. It is the only unit with a
  *two-state* model that the UI exposes directly (Pack/Unpack buttons).
- Trebuchets can target trees (to open paths), use Attack Ground, and are the only unit
  that only ever auto-targets buildings and other trebuchets. Rams likewise only
  auto-target buildings.

### 3.9 Stances, auto-targeting and idle behaviour

Four stances (AoK), set per unit, default aggressive (DE lets the player pick defensive
as the default):

- **Aggressive**: attacks anything in LOS and chases until the target dies or is lost.
- **Defensive**: attacks anything in *range*, chases a few tiles, then returns to its
  spot; retreats a few tiles when shot from outside LOS.
- **Stand ground**: never moves; attacks what is in range.
- **No attack**: does nothing unless ordered (used for monks, scouting, and lures).
- Siege units default to defensive or no-attack (Mangonel line, Bombard Cannon, Petard);
  DE added **Auto Scout** for scout units.

Target priority when auto-acquiring:

1. Trebuchets → buildings and enemy trebuchets only; rams → buildings only.
2. Everyone else → the **closest enemy unit**; if none, an enemy Town Center in LOS; then
   other buildings with lower priority.
3. Farms, walls, gates and herdables are never auto-targeted.
4. Units remember the last attacker: a unit that is shot will counter-target the shooter if
   its stance permits (and the shooter was revealed by the hit).
5. Mangonels and onagers (since AoC) refuse to fire automatically if the shot would hit
   friendly units ("smart siege").

Other behaviours:

- **Attack move** (patrol to a point): move while engaging anything met. **Patrol** cycles
  between two points. **Guard** follows a unit/building and attacks what attacks it.
  **Follow** just follows.
- An **idle** military unit has no current action; the `,` key cycles idle military.
- Fleeing: units auto-flee from mangonel shots in DE and from attacking monsters only as
  part of AI micro; villagers do not auto-flee.
- Units blocked for a few seconds give up the path and go idle (another source of idle
  villagers).

### 3.10 Death, destruction and ungarrison

- Unit death: HP ≤ 0 → death animation → corpse ("decay" graphic) that lingers, then is
  removed; dead hunted animals are food sources that rot at 1 food per 2 s.
- Building destruction: HP ≤ 0 → collapse animation → rubble object (sized to the
  footprint) for a while. All garrisoned units are ejected unharmed at the moment of
  destruction (also at conversion, and when HP drops to ≤ 20 %). A building that is
  deleted by its owner is destroyed instantly (used to "delete-wall").
- Deleting units is a command (`Delete`); it is logged and cannot be undone — a classic
  "GG" ritual is deleting your Town Center before resigning.

### 3.11 Victory conditions

- **Conquest**: a player is defeated when they have no units capable of building, training
  or fighting left (in practice: no villagers, no military, no production buildings — walls,
  houses and the like do not count). When only one player (or one allied group with Allied
  Victory ticked) remains, the game ends. Present in every mode.
- **Standard** = Conquest + **Wonder** + **Relic** victory:
  - Wonder: build it (very long build time, 1000 wood/stone/gold) and keep it standing for
    the countdown; every player is warned and shown its location when construction starts
    and when it completes.
  - Relic: hold **all** relics on the map in Monasteries for the same countdown.
  - Countdown by map size: Miniature 100 years; Tiny/Small/Medium **200 years**
    (16:40 at 1.0×, 9:48 at 1.7×); Normal 300; Large 350; Huge/Giant/Massive 400;
    Enormous/Colossal/Incredible 450; Monstrous/Ludicrous 500. Multiple wonders have
    independent timers; only the lowest is shown.
- **Time limit**: highest score when the timer expires (25 min to 2 h).
- **Score**: first to reach the target score (4,000–14,000) wins. Score = economy
  (resources gathered, trade, relics) + military (kills, razings) + technology + society
  (wonder, castles), updated continuously and shown with `F4`.
- **Last Man Standing**: Standard with Allied Victory forced off.
- Mode-specific: **Regicide** (lose your King → defeated; everyone starts with a King,
  a Castle and extra villagers), **King of the Hill** (hold the central Monument until its
  timer runs out; DE adds a resource trickle to the holder), **Capture the Relic**
  (garrison the single relic in your Monastery), **Defend the Wonder**, **Wonder Race**
  (first completed Wonder wins; teams locked), **Sudden Death** (lose your only Town
  Center → defeated; cannot build more), **Empire Wars** (start in Feudal with a working
  economy), **Battle Royale** (shrinking map), **Treaty** (no fighting for N minutes).
- The in-game dialog "You are victorious!" / "You have been defeated" ends the match;
  defeated players may keep watching.

### 3.12 Diplomacy

- Three stances per pair of players, set unilaterally: **Ally**, **Neutral**, **Enemy**.
  Your units never auto-attack allies; attack neutral *military* units (not villagers or
  buildings) automatically; attack everything of enemies. Stances are not symmetric: the
  diplomacy screen (AoC+) shows what others set toward you.
- **Allied Victory** checkbox: with it, a win by any ally (wonder, relic, conquest) is
  shared; without it, your ally winning means you did not.
- **Tribute**: send any amount of a resource to any player through the Diplomacy screen
  (30 % fee, reduced by Coinage, removed by Banking; Shift for lots of 500, Ctrl for all).
- **Cartography / shared LOS**: allies' vision is shared (automatically in DE).
- **Locked Teams** (AoC lobby option) freezes stances for the whole match — the default
  for ranked. "Team together" places teams adjacent; "Team positions" orders them around
  the map.
- Allied units may garrison in each other's buildings and repair each other's buildings;
  trade needs an allied (or your own) Market/Dock.
- Campaign AIs switch stance through triggers; skirmish AIs demand tribute, threaten
  ("It shall be war to the knife!") and switch to enemy after repeated refusals.

---

## 4. Random maps

### 4.1 The standard maps

AoK shipped with the first thirteen (plus "Highland", "Rivers", "Migration", "Team
Islands" and "Yucatán" arriving with AoK/AoC), AoC added **Arena, Ghost Lake, Mongolia,
Nomad, Oasis, Salt Marsh, Scandinavia, Yucatán** and the downloadable "ES@" maps. Each map
is a random map *script* (`.rms`); the layout below is what the script tends to produce.

| Map | Type | Layout |
|-----|------|--------|
| **Arabia** | Land, open | Dry desert/grass plains with scattered small forests, no water. Starting wood in medium clumps, gold and stone piles sometimes exposed; nothing to hide behind, so it is the archetypal "rush" map and the ranked default. 5 relics in 1v1, +1 per extra player. |
| **Archipelago** | Water | Many islands in a large ocean; players may share an island with a neighbour (friend or foe); four extra small islands, two of them rich in resources. Food comes mostly from fish. |
| **Baltic** | Hybrid | A central sea surrounded on all sides by land; teams occupy wide shores joined by thin land corridors that are easy to wall. Relics spawn on a central island with the only predators. Water control = economy + transport routes. |
| **Black Forest** | Land, closed | Dense forest everywhere with narrow winding gaps between players; clearings may hold gold/stone, boar packs or a relic. Starts in "explored" fog by default. Easy to wall, late-game sieges; relics scale to 2 per player (max 16). |
| **Coastal** | Hybrid | Roughly half land, half water: a sea around a central landmass, with occasional gulfs, lakes and shallows. Plenty of wood in pocket forests; shore fish near many starts. |
| **Continental** | Hybrid | One big landmass surrounded by water, cut by rivers/lakes and shallows, with an island or peninsula in the middle connected to everyone. Wood in piles, gold slightly above average. |
| **Crater Lake** | Hybrid | A huge central lake (about a third of the map) with an off-centre island holding three large gold piles (8–15 tiles each) guarded by wolves; players start on peninsulas around the lake. Multiplayer-only in the original. |
| **Fortress** | Land | Every player starts inside a complete stone wall with a Castle, towers and a Feudal-era base; more relics than usual; aggressive early play is impossible, so it is a booming/siege map. |
| **Gold Rush** | Land, open | Standard forested starts but almost all extra gold sits in one central desert, often in a crater ring or on a hill, guarded by wolf packs; stone around its edge. Map control decides the gold. |
| **Highland** | Hybrid | Cliffy, heavily forested land divided by one or more rivers with shallows and salmon; rivers may widen into lakes. Chokepoints and river crossings dominate. |
| **Islands** | Water | Each player on their own large island regardless of team; standard starting resources; naval map. |
| **Mediterranean** | Hybrid | A smaller, more asymmetric central sea than Baltic, ringed by land with extra herdables in groups of three; rarely has bays. |
| **Migration** | Water → land | Everyone starts on a small, poor island (no boars) and must move to the rich central continent where all relics are; forces early transports and a second town. |
| **Rivers** | Hybrid | Like Highland without cliffs: several narrower rivers with frequent shallows, sometimes a surrounding sea, islets in the middle; extra deer herds; frozen shallows on winter variants. |
| **Team Islands** | Water | One island per team (a free-for-all degenerates to Islands); large islands with pocket forests, extra herds and predators near the coast; relics split evenly between islands. |
| **Yucatán** | Land, open (AoC) | Jungle terrain with turkeys instead of sheep, extra berry bushes and a lot of food; gold a little scarcer; a fast-economy land map. |
| **Arena** (AoC) | Land, closed | Each player begins inside a ring of stone walls; the centre is a large open space ringed by resources and relics; the whole map is surrounded by trees. Everything is a Castle-Age-and-later fight. |
| **Ghost Lake** (AoC) | Land, open | A large frozen lake in the middle (no-man's land) with sheep pairs and bits of shore fish on it; gold and stone around the lake, wood scarce; mostly flat with some hills and cliffs. |
| **Mongolia** (AoC) | Land | Steppe with unusual rough cliffs and hills creating chokepoints; medium groves; extra deer herd near the start (up to 8 passive huntables). |
| **Nomad** (AoC) | Hybrid, nomadic | A Coastal-style landmass but players start with **three scattered villagers, no Town Center and no scout**, plus the TC cost (275 wood 100 stone) in their stockpile. The first decision is where to found the town. |
| **Oasis** (AoC) | Land | A single central waterhole surrounded by a massive palm forest; little wood elsewhere; extra sheep in triples. Fighting happens on the forest edges. |
| **Salt Marsh** (AoC) | Hybrid, open | Marshland of shallows and navigable water with moderate trees; very asymmetric; fish scarce; islands in the centre may hold resources. Hard to wall. |
| **Scandinavia** (AoC) | Land/hybrid | Snow map with two fjords along opposite edges full of fish; no berries but larger deer herds and a third boar; stone scarce; cliffs and hills modest. Footprints in snow; relics hard to see. |

Other families: **Real World** maps (Byzantium, Italy, Britain, Iberia, France, Central
America, Norse Lands, Texas, Sea of Japan…) have fixed, hand-shaped land; **Special** maps
(Acropolis, Budapest, Cenotes, City of Lakes, Golden Pit, Hideout, Hill Fort, Lombardia,
Steppe, Valley, MegaRandom, Michi, Socotra, Nile Delta, Water Nomad…) come from the HD/DE
eras; the "ES@" custom maps (Canyons, Mountain Pass, Moats, Pyramid, Sahara, Sherwood
Forest…) were Ensemble downloads later merged into Standard.

### 4.2 How RMS scripts work

A random map script is a plain-text `.rms` file. The parser is permissive (unknown tokens
are skipped), case-sensitive, integer-only in older versions (`rnd(min,max)` and floats in
UP/DE), and processes commands in order within each section. A script has up to seven
sections, which the generator **always executes in this order** regardless of where they
appear in the file:

```
<PLAYER_SETUP>            placement mode, game-data tweaks
<LAND_GENERATION>         base terrain, player lands, neutral lands
<ELEVATION_GENERATION>    hills
<CLIFF_GENERATION>        cliffs
<TERRAIN_GENERATION>      paint terrain clumps on top of lands
<CONNECTION_GENERATION>   carve/replace terrain between lands (paths, shallows)
<OBJECTS_GENERATION>      units, buildings, resources, gaia
```

Global syntax usable anywhere: `/* comments */`, `#const NAME N`, `#define FLAG`,
`#include_drs file`, `#includeXS file` (DE), `start_random … percent_chance N … end_random`,
`if COND … elseif … else … endif`. Predefined conditions include map sizes (`TINY_MAP` …
`LUDIKRIS_MAP`, DE `MAPSIZE_*`), player counts, game modes (`REGICIDE`, `DEATH_MATCH`,
`KING_OF_THE_HILL`, `EMPIRE_WARS`…), resource levels, starting age, version flags
(`UP_AVAILABLE`, `DE_AVAILABLE`), and anything the script `#define`s itself (seasonal
variants are usually `start_random` + `#define WINTER`).

**PLAYER_SETUP**

- `random_placement` (default): players are placed on a circle around the centre with
  some jitter; `grouped_by_team`: teammates adjacent (distance = 2 × `base_size`);
  `direct_placement` (UP/DE): lands positioned explicitly with `land_position x% y%` and
  `assign_to_player N`.
- `nomad_resources` adds a TC's cost to everyone's stockpile; `force_nomad_treaty` (DE)
  forbids fighting and building within 10 tiles of another TC until all TCs exist or 5 min
  pass.
- `ai_info_map_type TYPE isNomad isMichi showType` informs the AI.
- `effect_amount` / `effect_percent EFFECT TARGET ATTRIBUTE VALUE` (UP/DE) patch game data
  for this map only (e.g. extra starting food, 10-pop houses, renamed units);
  `guard_state OBJECT RESOURCE DELTA FLAGS` adds a "lose if you do not own X" rule or a
  resource trickle while you own X (how KotH/Regicide-style rules are scripted);
  `override_map_size`, `set_gaia_civilization`, `weather_type` (UP), `water_definition`,
  `behavior_version` (DE) also live here.

**LAND_GENERATION**

- `base_terrain T` fills the whole map (water for island maps, forest for Black Forest).
- `create_player_lands { … }` makes one land per player; `create_land { … }` a neutral
  land (optionally `assign_to_player`). Attributes: `terrain_type`, `land_percent %` or
  `number_of_tiles N`, `base_size N` (the square seed), `set_circular_base`, `land_position`,
  `circle_radius`, `left/right/top/bottom_border %`, `border_fuzziness %`,
  `clumping_factor`, `base_elevation N`, `zone N` / `set_zone_by_team` /
  `set_zone_randomly`, `other_zone_avoidance_distance N`, `min_placement_distance`,
  `land_id` (DE).
- Generation model: all land *origins* (square seeds of `base_size`) are placed first,
  then **all lands grow simultaneously** outward from their seeds until each reaches its
  tile budget or hits a border or another land. Zones decide which lands may touch.

**ELEVATION_GENERATION** — `create_elevation N { base_terrain T; number_of_tiles N;
number_of_clumps N; set_scale_by_size | set_scale_by_groups; spacing N;
enable_balanced_elevation (DE) }` raises hills up to level N in clumps on the named terrain.

**CLIFF_GENERATION** — `cliff_type T` (DE), `min/max_number_of_cliffs`,
`min/max_length_of_cliff`, `cliff_curliness %`, `min_distance_cliffs`,
`min_terrain_distance` (keep cliffs away from water/forest).

**TERRAIN_GENERATION** — `create_terrain T { base_terrain T; land_percent % |
number_of_tiles N; number_of_clumps N; clumping_factor N; spacing_to_other_terrain_types N;
set_avoid_player_start_areas; set_flat_terrain_only; height_limits; terrain_mask (DE
layering); set_scale_by_size }` paints clumps of forests, deserts, ponds etc. on top of
lands. `color_correction` picks the season shader (DE).

**CONNECTION_GENERATION** — `create_connect_all_players_land`, `create_connect_teams_lands`,
`create_connect_all_lands`, `create_connect_same_land_zones`, `create_connect_land_zones`,
`create_connect_to_nonplayer_land` (DE) with `replace_terrain FROM TO`, `terrain_cost T N`
(path-finding cost per terrain so the connection prefers e.g. shallows over deep water),
`terrain_size T width variance`, `default_terrain_replacement T`. This is how Rivers gets
its shallows and Black Forest its paths; `accumulate_connections` (DE) lets later
connections see earlier ones.

**OBJECTS_GENERATION** — `create_object OBJ { number_of_objects N; number_of_groups N;
group_variance; group_placement_radius; set_tight_grouping | set_loose_grouping;
set_place_for_every_player; set_gaia_object_only; set_scaling_to_map_size |
set_scaling_to_player_number; min_distance_to_players N; max_distance_to_players N;
min_distance_group_placement N; temp_min_distance_group_placement N; terrain_to_place_on T;
find_closest; require_path; place_on_forest_zone / avoid_forest_zone; avoid_cliff_zone;
min_distance_to_map_edge; actor_area / avoid_actor_area (DE); resource_delta (DE);
second_object; set_building_capturable; make_indestructible; force_placement }`. Walls use
`set_place_for_every_player` with equal `min/max_distance_to_players` to draw a ring with
gates; `create_object_group` (DE) places mixed groups.

Limits worth knowing: pre-DE scripts allowed only 99 `create_*` commands per section
(100,000 in DE); scaling constants must stay ≤ 9,320 or Ludicrous fails; objects placed on
per-player lands are automatically mirrored for all players.

### 4.3 Typical starting layout

The standard DE start per player (from `GeneratingObjects.inc`, as summarised by Zetnus):

- 1 **Town Center** at `max_distance_to_players 0` (the player's land origin).
- **3 Villagers** about 6 tiles away (civ bonuses add more — Chinese 6, Mayans 4; in DE
  extras spawn from the TC).
- 1 **Scout** (Scout Cavalry, Eagle Scout for Mesoamericans, Camel Scout for Gurjaras)
  7–9 tiles away; in Regicide a King replaces it (plus a Castle and 7 extra villagers).
- **6 Forage (berry) bushes** at least 10 tiles away, as one group.
- **Gold**: a group of **7** at least 10 tiles, a group of **4** at least 18 tiles, a group
  of **4** at least 21 tiles ("7+4+4"; Arabia's main gold is sometimes exposed in front).
- **Stone**: a group of **5** at least 12 tiles, a group of **4** at least 16 tiles.
- **Sheep**: a group of **4** at ~7 tiles (captured by the scout immediately) and two
  groups of **2** at a shared random distance of 14–24 tiles (cows/water buffalo: 3 + 3).
  "8 sheep" is the folk summary.
- **2 Boars** (or elephants, rhinos…) each at least 16 tiles away, individually placed.
- **Deer**: one group of 3–4 at 14–24 tiles.
- **1 predator** (wolf, jaguar, lion…) at least 34 tiles away.
- **Straggler trees**: 2 at 4–5 tiles and 3 at 6–8 tiles from the TC (so the first
  lumberjacks can drop at the TC); the main forests are terrain clumps from
  TERRAIN_GENERATION, usually within 10–15 tiles.
- Scattered map resources: relics **5/5/5/7/8** on tiny/small/medium/large/huge (effectively
  unlimited on gigantic/ludicrous, e.g. 35 on Ludicrous); extra gold groups of 3–4 and stone
  groups of 2–4 at least 40 tiles from any player; fish of several kinds; 30 scaling
  straggler trees; 4 birds.
- Starting stockpile: **200 wood, 200 food, 100 gold, 200 stone** (Low/Standard); Medium
  500/500/300/300; High 1000/1000/700/700; Ultra High / Deathmatch 20,000/20,000/10,000/
  5,000 (deathmatch is post-Imperial by default).

Design note for Wars: this layout (TC, 3 villagers, scout, "8 sheep, 2 boar, 6 berries,
7+4+4 gold, 5+4 stone, straggler trees") is the single most recognisable "feel" of an
AoE2-style start; Wars' map generator should reproduce the *distances and counts* even if
the asset names differ.

---

## 5. Scenario editor, triggers, AI scripting, cheats

### 5.1 Editor tabs

The scenario editor (absent on PS2/consoles) is a mode of the game client; DE can boot
straight into it with the `EDITOR` launch argument. Scenarios are played through
"Custom scenario" in Standard Game or with the editor's **Test** button. Load order of a
scenario: civ tech trees → team bonuses → free Dark Age techs → map → unit placement →
game start → triggers with no conditions fire.

| Tab | Contents |
|-----|----------|
| **Map** | New blank map (any terrain in DE), **Random map** from any RMS, **Seed map** (random map with a chosen RNG seed), map size (Tiny 120 … Ludicrous 480, hidden sizes with `MORE_MAP_SIZES`), AI map type hint, colour mood (DE), team positions, **Extend map** (grow only, with offset), `.xs` script file name. |
| **Terrain** | Brushes: **Map** (paint terrain; beach auto-generation, waves, eye candy, DE terrain layering), **Elevation** (levels 1–7, 1–16 since 141935; water refuses slopes), **Cliffs** (six types), **Map Copy** (copy/paste regions, with filters: everything / buildings / eye candy / gaia / terrain / units), **Erase** (same filters plus invisible objects and terrain overlays). |
| **Players** | Number of players, per-player: starting age, starting resources, population cap, colour, name / string ID, civilisation + lock, architecture, AI personality (+ lock), **base priority** for team placement, swap players. |
| **Units** | Four lists — Units, Buildings, Heroes, Other (gaia resources, flags, eye candy). Tools: Place (with rotation by scroll wheel; click a building to garrison into it), Delete, Garrison delete, Move, Rotate, Select, **Convert** (brush that re-owns objects of listed players). Wild animals and resources are under the Gaia player. A player with no objects starts with the default TC + 3 villagers + scout. |
| **Diplomacy** | Stance matrix per player (ally/neutral/enemy) and Allied Victory flags. |
| **Global Victory** | Standard / Conquest / Score (with or without time limit) / **Custom** (conquest, exploration %, relics, "all of" vs "any of"), plus secondary modes (Empire Wars, Sudden Death, Regicide, King of the Hill). |
| **Options** | Disabled units/buildings/technologies per player (the "tech tree" restrictions), full tech tree flag, starting view, lock diplomacy for humans, villagers lose carried resources on retask, units push past each other, test difficulty. |
| **Messages** | Instructions, Hints, Scouts report, Victory text, Loss text, History — shown in the pre-game screen and the Objectives panel; may reference string-table IDs. |
| **Cinematics** | Pre-game and win/lose videos (files from the game's data folder) and an instructions background image. |
| **Triggers** | The scripting system (below); DE adds copy/paste, display order, grouping and importing triggers from other scenarios. |
| **(Campaign editor)** | Chains scenarios into a `.cpn`/`.aoe2campaign` with a story order. |

### 5.2 The trigger system

A trigger is a named list of **conditions** and **effects** with a few flags:

- **Enabled** (starting state; a disabled trigger does nothing until an "Activate
  trigger" effect turns it on).
- **Looping**: after firing, the trigger re-arms and can fire again on a later evaluation
  instead of being consumed. A looping trigger with a `Timer` condition is the idiom for
  periodic events; a looping trigger with no conditions fires every evaluation.
- **Display as objective** (+ objective description and order) shows the trigger in the
  Objectives panel until it fires.
- **Short description / long description**; DE adds a **display order** independent of
  creation order, and an **inverted** flag on conditions (NOT).

Evaluation: the engine walks the trigger list in order (trigger ID order; DE's "display
order" is cosmetic but the "execution order" can be edited) once per game tick / "trigger
pass"; a trigger fires when **all** of its conditions are true (AND). OR is only available
through the DE `OR` pseudo-condition (and the `AND` grouping), or by duplicating triggers.
Effects run **in list order**, synchronously, within the same pass. Chaining is done with
"Activate trigger" / "Deactivate trigger"; the classic pattern is a loop trigger that checks
a condition and activates a one-shot trigger. Because timers are in whole seconds and
evaluation is tick-based, two triggers that become true in the same pass execute in ID
order — a well-known source of ordering bugs. Counters and variables (DE) make most
"activate a chain of 20 triggers" patterns unnecessary.

Each condition/effect carries a parameter block: amount, resource type, unit object, next
object, object list/type, source/target player, technology, timer, trigger ID, location
(x, y) and area (x1, y1, x2, y2), object group/class, AI signal, variable, and (DE) string
ID, sound name, facet, quantity op. Areas are inclusive tile rectangles.

**Conditions** (AoK/AoC core, then additions):

| Condition | Meaning |
|-----------|---------|
| Bring object to area | a specific unit is inside the area |
| Bring object to object | a specific unit is adjacent to / touching another object |
| Own objects | player owns at least N objects (optionally of a type/class) |
| Own fewer objects | player owns fewer than N |
| Objects in area | at least N objects (of type/class, for player) are inside the area |
| Destroy object | a specific object is dead/destroyed |
| Capture object | a specific object is now owned by player X |
| Accumulate attribute | player's resource/attribute (food, wood, kills, razings, population, …) ≥ N |
| Research technology | player has researched tech T |
| Timer | N seconds elapsed since the trigger became active |
| Object selected | (single-player) the player selected the object |
| AI signal | the AI script called `(set-signal N)` |
| Player defeated | player X has lost |
| Object has target | object is targeting another object |
| Object visible / not visible | object is in / out of player's LOS |
| Researching technology | tech T is currently in the queue |
| Units garrisoned | ≥ N units garrisoned in a building |
| Difficulty level | current difficulty ≤ / = level |
| Chance | random %-chance, re-rolled each pass |
| Technology state (HD+) | tech is available/disabled/researched |
| Variable value (DE) | compare a variable to N (=, <, >, ≤, ≥, ≠) |
| Object HP (DE) | object's HP compared to N |
| Diplomacy state (DE) | player X's stance toward Y |
| Script call (DE) | an XS function returned true |
| Object selected / visible (multiplayer) (DE) | per-player variants that are network-safe |
| Object has action (DE) | object is doing action A (moving, attacking, …) |
| OR / AND (DE) | boolean grouping of the following conditions |
| Building is trading / Display timer triggered / Victory timer (DE) | misc. |
| Decision triggered (DE) | player chose option N of a "Create decision" dialog |
| Object attacked / Hero power cast / Compare variables / Trigger active / Local tech researched (DE) | misc. |

**Effects** (AoK/AoC core, then additions):

| Effect | Meaning |
|--------|---------|
| Change diplomacy | set source player's stance toward target player |
| Research technology | grant tech T to a player instantly (optionally force) |
| Send chat | chat line to a player (with sound, in player colour) |
| Play sound | play a named sound file |
| Tribute | move N of a resource from source to target player (negative to take) |
| Unlock gate / Lock gate | gate state |
| Activate trigger / Deactivate trigger | enable/disable another trigger |
| AI script goal | set `(goal N value)` in the AI (via `(event-detected trigger N)`) |
| Create object | spawn a unit/building at (x, y) for a player |
| Task object | order selected objects to a location/object (gather, attack, build…) |
| Declare victory | player wins (allies share if Allied Victory) |
| Kill object | kill objects (death animation) |
| Remove object | remove objects silently |
| Change view | scroll the player's camera to (x, y) (with "scroll" option) |
| Unload | eject garrisoned units at (x, y) |
| Change ownership | give objects to another player |
| Patrol | order objects to patrol to (x, y) |
| Display instructions | show text in the top-left message area for N seconds (optional sound) |
| Clear instructions | clear it |
| Freeze unit | stop and set "no attack"; unit ignores orders until re-tasked |
| Use advanced buttons | enable the (AoK) advanced command buttons |
| Damage object | subtract N HP (negative heals) |
| Place foundation | place an unbuilt building for a player |
| Change object name / HP / attack (AoC) | rename or alter stats; the basis of custom units in campaigns |
| Stop unit (AoC) | stop current action |
| Attack move / Change armor / range / speed / Heal / Teleport / Change stance (HD+) | stat and order helpers |
| Display timer / Clear timer (HD+) | countdown in the UI |
| Enable/disable object / technology (HD+) | tech-tree edits at runtime |
| Change object cost / icon / description; Replace object; Change player name / civ name; Change train or tech location; Create garrisoned object (HD/DE) | misc. |
| Modify attribute (DE) | set/add/multiply any unit attribute (HP, attack, armour, LOS, speed, cost, train time, garrison capacity, …) for a type or class |
| Modify resource (by variable) (DE) | adjust player resources/attributes |
| Change variable / Script call (DE) | numeric variables and XS calls |
| Set building gather point; Change object player colour; Disable/enable unit targeting, selection, deletion; Change colour mood; Train unit; Initiate research; Create decision; Count units into variable; Add train location; Build object; Mirror diplomacy; Change object visibility (DE) | misc. |

Tricks that scenario designers rely on, and which tell Wars what a trigger system must
support: looping "Create object" to simulate fountains; "Change ownership" to Gaia to make
capturable buildings; "Freeze unit" + "Task object" for cutscenes; "Change view" for camera
work; "Tribute" with negative amounts as a tax; "Damage object" with negative values for
healing auras; "Activate/Deactivate" chains as a state machine. Numeric limits: resources
display breaks beyond 999,999,999 (2^31 cap since HD); unit HP/attack above 32,000 wraps.
Several AoK effects were unsafe in multiplayer (anything that reads one player's camera or
selection), which is why DE added explicit `(multiplayer)` variants.

### 5.3 AI scripting (`.ai` / `.per`)

Computer players are driven by a rule-based scripting language documented in Ensemble's
*Computer Player Strategy Builder Guide*. An AI is a pair of files in the `ai/` folder: an
empty `name.ai` marker and `name.per` with the rules (DE also accepts `.per2` include
files and XS scripts for low-level control).

- A script is a flat list of **rules**: `(defrule (fact) (fact) … => (action) (action) …)`.
  **Facts** are read-only queries (`(current-age >= feudal-age)`, `(unit-type-count
  villager < 20)`, `(resource-found wood)`, `(can-build barracks)`, `(enemy-buildings-in-town
  > 0)`, `(timer-triggered 3)`, `(taunt-detected any-ally 31)`, `(goal 7 1)`,
  `(strategic-number sn-percent-civilian-explorers > 0)`), **actions** are commands
  (`(build barracks)`, `(train militiaman)`, `(research ri-loom)`, `(attack-now)`,
  `(set-strategic-number sn-food-gatherer-percentage 50)`, `(set-goal 7 1)`,
  `(enable-timer 3 60)`, `(chat-to-all "…")`, `(taunt 14)`, `(tribute-to-player 2 gold 100)`,
  `(set-signal 5)`, `(disable-self)`).
- The engine evaluates **every rule, top to bottom, on every AI pass** (roughly once per
  game second; the rate is itself a strategic number). A rule whose facts are all true fires
  its actions; `(disable-self)` turns it into a one-shot. There are no loops, functions or
  local variables in the classic language; state lives in **goals** (an array of ~512
  integers), **timers**, and **strategic numbers** (`sn-*`, ~300 knobs such as
  `sn-maximum-town-size`, `sn-number-explore-groups`, `sn-percent-attack-soldiers`,
  `sn-minimum-attack-group-size`).
- `(defconst name value)` names goal IDs, timers and magic numbers;
  `#load-if-defined SYMBOL … #else … #end-if` conditionally compiles blocks for civ, map
  type, difficulty, resource level, game mode, player number (`DIFFICULTY-HARDEST`,
  `MAP-TYPE-ARABIA`, `DEATH-MATCH`, `GAIA-AZTECS`…).
- Economy micro is **not** scripted: the engine's built-in gatherer manager distributes
  villagers according to the `sn-*-gatherer-percentage` numbers, builds farms, hunts and
  drops off; the script only steers percentages and build orders. Attacks are group-based:
  the engine forms attack groups of `sn-minimum-attack-group-size` units and
  `(attack-now)` sends them at the current target player.
- UserPatch (and DE) extended the language with hundreds of `up-*` facts/actions
  (`up-find-local`, `up-target-point`, `up-retask-gatherers`, `up-jump-rule`, goal
  arithmetic, string formatting) that make per-unit control (DUC, "direct unit control")
  possible — this is how DE's AI can micro crossbowmen.
- Difficulty levels change behaviour through compiled-in flags (easiest AIs never attack
  first and only age up after a human does; hardest AIs cooperate, build offensive towers
  and, in AoK/AoC, received 500 of each resource per age — a bonus removed in The Forgotten).
- The scenario bridge is two-way: triggers can set AI goals ("AI script goal" effect →
  `(event-detected trigger N)`), and AIs can `(set-signal N)` for the "AI signal"
  condition.

### 5.4 Cheats in context

Cheats (listed in §1.5) are **commands in the simulation**: typing one sends a chat-like
command through the normal command path (`0x67 Diplomacy/Cheats/Change speed` in the
network protocol), so every client executes it deterministically and it appears in the
recorded game. That is why cheats need the lobby's "Allow cheats" flag and why they can be
spectated and replayed. The same path carries game-speed changes and diplomacy changes.

---

## 6. Multiplayer and netcode

### 6.1 "1500 Archers on a 28.8" — the design

Paul Bettner and Mark Terrano's 2001 Gamasutra article (now on gamedeveloper.com)
describes the networking model Ensemble built for Age of Empires (1996–97) and reused in
AoE2. The constraints: **8 players**, "epic" battles with hundreds of units per side, a
minimum machine of a Pentium 90 with 16 MB RAM on a **28.8 kbit/s modem**, a target of
**15 FPS** on that machine, and a single-threaded 2D engine (Genie) that already existed.

**Why not send unit state.** A quick calculation (x, y, status, action, facing, damage
for every moving unit, every update) capped the game at roughly **250 moving units** on a
28.8 modem. The design goal of thousands of units on screen killed any state-replication
approach.

**Synchronous simulation ("lockstep").** Instead, every machine runs **the exact same
simulation** and receives **the exact same player commands at the same simulation time**.
Only commands are sent (a few bytes each: "player 3 orders units 17, 18, 19 to move to
x, y"); the simulation itself produces identical results everywhere. Requirements:

- fully **deterministic** game code — same inputs, same outputs on every machine;
- a **seeded, synchronised random number generator** (every random roll is part of the
  simulation and must happen in the same order everywhere);
- no simulation decision may depend on local state (frame rate, free CPU time, settings,
  which part of the map is on screen, mouse position).

**Communication turns.** Time is divided into **turns, typically 200 ms long** (adjusted
dynamically). Commands issued during turn *N* are not executed immediately; they are sent
to all peers tagged with the turn in which they must execute, **two turns later
(N + 2)**, giving the network two full turns to deliver and acknowledge the packet. At any
moment three turns are "in flight": the one being executed, the one whose commands are
being received, and the one whose commands are being sent. Inside a turn the game renders
as many frames as it can; the simulation advances in sub-steps between turns.

**Speed control.** Because every machine must have everyone's commands before it can
execute a turn, the game runs at the pace of the slowest machine and worst link. Rather
than stall, the host tunes two numbers continuously:

- Each client reports, in its "Turn Done" message, its sustainable **frame rate** (an
  average over recent frames) and the **worst average ping** it measures to any other
  client (clients ping each other periodically). Overhead: **2 bytes**.
- The host computes a target frame rate and a turn length and broadcasts them. The turn
  length is roughly the round-trip latency, split into the number of frames the slowest
  machine can render in that time (e.g. 200 ms turn, 4 frames of 50 ms).
- Turn length **rises quickly** when latency spikes and **falls slowly** (a few ms per
  turn) so that players never feel a sudden change. A consistent 250 ms delay is
  unnoticed; 250–500 ms is playable; above 500 ms it becomes noticeable; an *inconsistent*
  delay (80 ms then 500 ms) feels jerky even when the average is fine.

**Latency measurement and tolerance.** Prototyping in single-player with artificial
command delays showed that ~250 ms of command latency goes unnoticed in an RTS. Players
issued a command every 1.5–2 s on average with spikes of 3–4 per second in battles; rapid
re-clicking produced command floods, so a filter dropped repeated identical commands to
the same location.

**Transport.** DirectPlay on top of **UDP**, with Ensemble's own ordering, drop detection
and resend layer: every message carries its turn number and a sequence number; messages
for turns already executed are discarded; out-of-order arrival triggers an immediate resend
request ("when in doubt, assume it dropped"), and a late acknowledgement triggers a
proactive resend. Peer-to-peer: each client connects to every other client (lower latency,
no central point of failure, but N² connections and NAT trouble — RTS3 later added TCP and
NAT handling).

**Out-of-sync detection.** Because a single divergent bit compounds forever, each client
periodically computes **checksums** of the world state (objects, pathing, targeting, the
RNG, …) and compares them; a mismatch stops the game with an "out of sync" error. The
article calls OOS the "stubborn bug" that lasted until the last day of development; sync
traces could reach 50 MB. The same mechanism is a strong anti-cheat: a modified client that
changes its simulation immediately desyncs. Sync tooling (macros tagging data by system,
compiled out of release builds; multiplayer-enabled console commands and config variables)
was listed as essential.

**Recorded games (AoE2).** Since the simulation is deterministic, a game can be stored as
*the initial state plus the stream of commands*; replaying it reproduces the exact game.
AoE2's recorded games (`.mgx`/`.mgz`, DE `.aoe2record`) are just that, and were used both
for sharing famous matches and for reproducing bugs between developers.

**"Waiting for players".** Not described in the article, but the visible consequence of
the design: when a client has not received some player's commands for the next turn by the
time it must start executing it, the game cannot proceed — it freezes and shows the
"Waiting for players…" dialog with the lagging player's name, until the commands arrive or
the player is dropped. (Speed control exists precisely to make this rare.)

**Lessons the authors list:** know the user's latency tolerance; keep human-readable
metering on all the time (removing it early hid AI race conditions and expensive paths);
educate every programmer that a command is *issued*, *received* and *executed* at
different times and must be validated at both ends; do not trust third-party "guaranteed
delivery"; build tiny stress-test apps; test on modems early.

**AoE2-specific additions:** recorded games, file transfer (scenarios/maps to peers),
persistent stats on the Microsoft Gaming Zone, lobby-based settings through DirectPlay,
and a common stats block uploaded at game end for ladders.

### 6.2 What the wire actually carries (openage reverse engineering)

openage's reverse-engineering notes (Wireshark captures of AoC 1.0c on LAN) confirm the
article and add detail:

- **TCP** is only used to join a session; everything in-game is **UDP**.
- Three packet families: **sync** (periodic turn/time/ping/checksum traffic — the bulk of
  all traffic), **chat**, and **player actions** (`0x3e`), the latter written almost
  verbatim into the recorded-game file.
- A 20-byte header: source/destination network IDs, a command byte, three option bytes,
  the **communication turn** in which the payload executes, and a per-player counter.
  Packets for a past turn are discarded.
- Periodic sync packets: a **120 ms** heartbeat with an incrementing counter (counters are
  seeded 2,000 apart per player; a non-multiple difference indicates desync); a 120 ms
  turn-sync packet carrying the last in-sync turn and two ping bytes (delta ping and a
  10–15 packet rolling average); an **8 s** time-sync request/answer with elapsed game
  milliseconds; an **8–16 s** state-check packet with several 32-bit values that must match
  on every client for turn `current − 2` — the **checksum**; and a drop/desync packet
  carrying the last synchronised turn to resume from.
- **34 action types** were identified: primary action (attack/gather/garrison), stop,
  move, AI move, resign, set waypoint, stance, guard, follow, patrol, formation, save &
  exit, coordinated move (AI), train (AI), research, build, diplomacy/cheats/speed, build
  wall, delete, attack ground, tribute, repair, ungarrison, toggle gate, flare, garrison /
  cancel training, train, rally point, sell, buy, drop relic, toggle town bell, back to
  work. Move commands carry a player ID, a selection count (0xFF = "same selection as the
  previous command", a bandwidth trick) and **float** x/y; waypoint commands carry
  **byte** tile coordinates.
- Every object has a 32-bit ID from a global counter (a felled tree is a new object), and
  players have three IDs (network ID, player ID, player *number* — the last one allows an
  undocumented co-op mode where two players share a slot).
- In a paused game only the time-sync pair keeps flowing.

### 6.3 Players, saved games, LAN / IP / Zone

- Up to **8 players** (humans or AIs) plus Gaia; AoE2 never raised this (AoM allows 12).
- Lobby options: LAN (IPX in 1999, then TCP/IP), direct IP ("Internet TCP/IP connection
  for DirectPlay" — the host's IP typed by hand, plus port forwarding), modem-to-modem, and
  the **MSN Gaming Zone** (matchmaking/chat lobby; shut down in 2006, after which the
  community moved to GameRanger, GameSpy, IGZ and finally **Voobly**, which added ratings,
  anti-cheat, spectating with a configurable delay and the UserPatch).
- **Saved games** in multiplayer serialise the full state on every machine; restoring one
  requires all original players to rejoin with their original slots — a direct
  consequence of the lockstep model (there is no server to hold the state).
- **Recorded games** are opt-in per player (the "Record game" checkbox) and can be viewed
  at any speed with free camera and player perspective switching; because they are command
  logs they break whenever the game data or version changes.
- A "lag" indicator (coloured clock next to each player's name in DE: blue/yellow/red)
  surfaces the frame-rate report from speed control.

### 6.4 How DE does it today

Confirmed from official support pages, patch notes and community documentation:

- Lobbies, matchmaking (ranked and quick play, Elo with placement matches), friends and
  cross-play across Steam, Xbox app, Xbox consoles and PS5 run on Microsoft's online
  service (the client logs call it **RelicLink**, Relic's service layer).
- Players must pass a **benchmark test** (minimum score 900 for 1v1, 1,000 for team games)
  before ranked matchmaking, so that the lockstep pace is not dragged down by slow
  machines — the article's "slowest machine" problem, solved by exclusion.
- The in-game simulation is still **deterministic lockstep with command turns**; desyncs
  are logged by the client as a **"Checksum Mismatch"** between peers, and DE still ends
  the game on OOS (the error persists on ARM-Windows / Parallels setups where floating
  point behaves differently — a reminder that determinism across CPUs is hard with floats).
- **Spectator mode** is built in: a host can allow spectators and set a **spectator delay**
  (how far behind live the spectators watch), spectators can join from a recorded-game
  link or game ID, and the stream is the command log with a delay — the same data as a
  recording. The in-game UI supports switching perspective (`Ctrl+Shift+F1…F8`, `F9` Gaia).
- **Relay servers**: forum reports and support threads refer to traffic going through
  Azure-hosted relays rather than pure peer-to-peer, and the 2026 security patch implies a
  server-side content-distribution path for maps/mods; however, no official document
  describing the relay topology was found — treat "DE uses relay servers" as
  **[unconfirmed]**. What *is* clear is that DE still has no authoritative game server:
  the simulation runs on every client and a slow or disconnecting client still produces
  the classic stalls.
- Map/mod content is downloaded from a central mod service before the game starts (the
  "Failed to download match content" error), replacing AoK's peer-to-peer file transfer.

### 6.5 openage's approach to the same problems

openage (SFTtech, C++/Python, GPLv3) is a from-scratch engine that can load the original
AoE2 assets. Its documentation describes a different synchronisation philosophy:

- **No ticks.** The simulation is **event-driven**: every change is an event scheduled at
  a simulation time (a 64-bit fixed-point seconds value with a scaling factor of 16,
  `time::time_t`), executed by an event loop (`reach_time(t)` runs all events with
  `t_old < t_event ≤ t`). Game speed is a multiplier on a real-time clock; the renderer and
  input run in their own threads and read the state asynchronously. The clock caps real
  time per update at 50 ms to survive debugger pauses.
- **Curves.** Game data is stored as *keyframes on a timeline* (inspired by Planetary
  Annihilation's "ChronoCam"): `Discrete` curves (step interpolation — HP, state),
  `Continuous` (linear — position, build progress), `Segmented` (linear with jumps —
  angles), plus `Queue`, `UnorderedMap` and `Array` containers that track each element's
  lifespan. A value can be read for *any* time, past or future, and a keyframe can be
  inserted in the past without immediately invalidating everything — the authors argue
  this makes desync recovery and asynchronous networking tractable, at the cost of memory
  and interpolation overhead. Rules of thumb: use curves for anything tracked over time,
  rarely modified, or sent over the network; do not use them for per-frame scratch data.
- **Lockstep replaced.** The 2020 "new gamestate" merge explicitly *replaced the lockstep
  implementation the engine had used until then* with the event + curve system.
- **Networking plan.** The goal architecture (not finished) is **a single authoritative
  server running the simulation**, with every client receiving only the events and
  curve keyframes **visible to it** (fog-of-war filtering on the server, which lockstep
  cannot do — in AoE2 every client knows the whole map, which is why map hacks exist).
  Scripting and networking are both designed as optional sources/sinks of events.
- **Coordinates.** Physics coordinates are fixed-point with a 16-bit fraction on (NE, SE,
  UP) axes; tiles are the integer part; 16×16-tile chunks; conversions to screen space use
  floats only on the rendering side.
- **Pathfinding.** A tile-based **flow-field** pathfinder (after Elijah Emerson's "Crowd
  Pathfinding and Steering Using Flow Field Tiles"): the map is a grid of cost cells
  (1–254, 255 impassable), split into fixed-size sectors joined by **portals**; a
  high-level A* over the portal graph picks the sectors, then integration fields and flow
  fields are computed only for those sectors, and the path is the chain of waypoints
  obtained by following the flow vectors. Flow fields are cached and shared by groups
  heading to the same target. One grid per movement class.
- **Data.** Unit/tech definitions live in **nyan**, a typed, hierarchical, moddable data
  language, converted from the original `.dat` files; game entities are component-based
  (Idle, Move, Turn, Live, …) with "activities" (node graphs) describing behaviour.

The practical lesson for Wars: openage chose *authoritative server + replication of
visible state* for security and async friendliness; AoE2 chose *lockstep + command
replication* for bandwidth, simplicity and perfect replays. Wars' locked decision is the
AoE2 model (deterministic lockstep in Rust), with openage's fixed-point coordinate
conventions and flow-field ideas as the implementation references.

---

## 7. Sound and feedback

AoE2's audio is a feedback channel, not decoration; most of its cues map directly onto
simulation events and Wars should treat them as part of the event model.

### 7.1 Unit voice lines

- Every selectable unit acknowledges **selection** and **orders** with a short line in its
  civilisation's historical language (Latin for Byzantines/Italians, Old English for
  Britons, Mongolian, Classical Nahuatl, Yucatec Maya, …). Villagers have separate lines
  for selection and for each task they are sent to do ("lumberjack", "hunter", "farmer"…),
  which is how a player hears that a click landed on a tree instead of a bush.
- Lines are picked at random from 3–5 variants per category (select, move, attack, build,
  task) so that repeated clicks do not sound like a loop. Monks have their own set, kings
  and heroes are voiced individually in campaigns.
- Reviewers singled out these "civilization sound bites" at launch; they are a large part
  of the game's identity and of its meme culture (e.g. the Teuton "Ja wohl?", the Persian
  "Baleh", the Celt "Anm?").
- Animals grunt when selected, and sheep bleat when stolen — tiny cues that turn invisible
  sim events into information.

### 7.2 Alerts

Reverse-engineered from the sound files (openage notes):

- **Horn** (`50315`): one of your *military* units is under attack.
- **Bell** (`50316`): a *villager or building* of yours is attacked — the sound players
  mean by "you are under attack". The minimap blips flash in the attacker's colour and
  the "go to last notification" key jumps there.
- **Horn + monk chant** (`50313`): one of your units is being converted.
- **Horn + wolf** (`50366`): a wild animal is attacking your unit (not played for boars,
  which you lure on purpose).
- Alerts are throttled per **battle**: a battle is a 20×20-tile area centred on the first
  hit; one alarm per battle, and it ends after 10 s without fighting (a new fight starts a
  new battle). Separate fights more than 10 tiles apart alert separately. Killing enemies
  that have not hit you does not alert.
- Other global cues: a flare sound when an ally pings the minimap; "research complete" and
  "unit trained" chimes; a building-destroyed crash; the "wonder started/completed" and
  "relic captured" fanfares that *every* player hears (with a chat line naming the player).

### 7.3 Age-up fanfare

Advancing to the next age plays a short brass fanfare on every client and sends a chat
line to all players ("<player> has advanced to the Feudal Age"). Because it is public, it
is a strategic signal (timing a rush), and it is one of the most quoted sounds of the
game. The in-game music also changes with age and intensity in DE.

### 7.4 Idle and economy cues

- The **idle villager** button lights up with a counter; there is no sound, by design —
  the button and the `.` hotkey are the cue. (AoE2 was praised for inventing this; most
  later RTS copied it.)
- A villager who cannot find a resource says a "resource depleted"-style line only in
  later versions; otherwise idleness is silent. DE adds an optional sound.
- "Not enough wood/food/gold/stone" and "you need to build more houses" are voiced
  rejections of a command — they *must* be local-only, not simulation events.
- Market trades and tribute play a coin sound; relic income has no cue.

### 7.5 Chat and taunts

Chat lines arrive in the sender's colour with a soft chime; `;` prefixes allied chat, `!`
enemies, `*` everyone (AoC). Taunts (§1.3) are typed numbers that play audio and are
printed; `!mute` silences them. In DE, team chat, pings and taunts also drive the allied
AI. For Wars, chat and taunts are *not* simulation commands (they need no determinism),
but flares are (they appear in replays).

---

## 8. Systems Wars will implement first

Locked decisions for Wars (from the project plan): deterministic `sim` crate in Rust, no
floats, no `HashMap` iteration, one seeded RNG, every surface (server, desktop, TUI,
Android, iOS) runs the same sim through `client`. The following prioritised list maps the
AoE2 systems above onto that architecture. Numbers in bold are proposals, not research.

### 8.1 Determinism foundation (week 1 — nothing else works without it)

1. **Fixed-point maths.** `i32`-backed Q16.16 ("Fix") for positions, speeds and distances
   (openage's physics coordinates use exactly this fraction width); `i64` for intermediate
   products; integer `sqrt`/`atan2` tables for distances and facings. No `f32` anywhere in
   `sim`. Positions are `(x: Fix, y: Fix)` in tile units; tile index = integer part;
   `(0.5, 0.5)` is a tile centre (openage convention).
2. **Fixed tick.** **20 Hz** (50 ms game time per tick). All rates in the data files are
   per game second and are converted to per-tick at load time with exact rationals (e.g.
   gather 0.39/s → accumulate in a Q16.16 accumulator per tick). Game speed (1.0/1.5/1.7/
   2.0) only changes how many ticks run per real second.
3. **Seeded RNG** (one `xoshiro`/`pcg` instance in the world state; every roll goes
   through it in a fixed order — conversion rolls, accuracy, RMS generation, AI).
4. **Deterministic containers**: `Vec` + stable IDs (slot maps), `BTreeMap` where a map
   is needed; units iterate in ID order. Object IDs from a global counter (like AoE2).
5. **State hash** every **20 ticks (1 s)**: a 64-bit hash over units (id, owner, pos, hp,
   action), players (resources, pop), RNG state, tick number; stored in a ring buffer.
6. **Command log = replay**: the sim takes `(tick, player, Command)`; recording the
   initial state (map seed + lobby settings) plus the command log reproduces the game.
   Unit tests replay fixtures and compare the hash sequence.

### 8.2 Lockstep multiplayer (week 2)

7. **Command delay of 3 ticks (150 ms)**: a command issued at tick N by any client is
   scheduled for N + 3 everywhere (AoE2: 2 turns of 200 ms). The server (Wars has one —
   it relays and orders commands, it does not simulate) stamps the execution tick, so
   clients never disagree on ordering; the sim executes a tick only when it holds every
   player's (possibly empty) command batch for that tick.
8. **Adaptive delay** à la speed control: measure RTT per client; raise the delay fast
   (to 4–6 ticks) when the worst RTT exceeds the window, lower it slowly (one tick at a
   time, no more often than every few seconds). Show a per-player lag clock.
9. **Stall dialog**: when a batch is missing for > 1 s show "Waiting for <player>…";
   after a timeout the server drops the player (their units go to "no AI" like AoE2's
   resign) and the others continue from the last agreed tick.
10. **Desync detection**: clients send the 1 s hash to the server; the server compares;
    on mismatch it flags the minority client as out of sync. Keep a per-system hash
    breakdown (units / players / rng / pathing) in debug builds to find the diverging
    system, exactly as the "1500 archers" team recommends (tagged sync traces).
11. **Snapshot resync**: instead of ending the game, the server (or the majority client)
    serialises the full world state at the last agreed tick and sends it to the diverging
    client, which reloads it and replays the buffered commands. The same snapshot path
    gives us saved games, mid-game join for spectators, and crash recovery.
12. **Spectators** receive the command stream with a configurable delay (DE's spectator
    delay) and run the same sim; the server never needs a special spectator model.
13. **Cheats and flares as commands** (logged, replayable, lobby-gated); chat is out of
    band.

### 8.3 World and movement (weeks 2–3)

14. **Map** as a square tile grid with per-corner elevation (start with **8 levels**,
    make it a constant), terrain IDs with passability classes (land / water / shallows /
    impassable) and a buildability flag; chunked 16×16 for rendering/culling.
15. **Isometric renderer** with 2:1 diamonds, back-to-front tile order and y-sorted
    sprites; **8 facings with 5 stored + 3 mirrored** sprite sets to halve art cost;
    per-action frame counts in data.
16. **Fog of war** with the three-state per-player tile map; LOS as integer-radius circles
    rasterised once per unit move; "shot reveals shooter"; allied vision shared.
17. **Pathfinding**: hierarchical — a coarse sector/portal graph (A*) plus per-sector
    flow fields cached per target (openage model), with tile costs 1–254/255; unit
    collision radii in Fix; group moves at the slowest member's speed; line/box/
    staggered/flank formation slots assigned deterministically by unit ID.
18. **Stances and auto-target** exactly as §3.9 (closest unit, then TC, siege only
    buildings; farms/walls/herdables never auto-targeted) and the attack-move/patrol/
    guard/follow set.

### 8.4 Economy and buildings (weeks 3–4)

19. **Gather/drop loop** with per-task rates, carry capacity, nearest-drop-site search,
    return-to-same-resource, auto-reseed farms, smart-villager auto-gather after a drop
    site completes, idle flag + idle-villager button and `.` cycling.
20. **Build with diminishing returns** `3t/(n+2)`; foundations; walls block immediately;
    **repair** at half cost charged progressively, 750/375 HP/min.
21. **Garrison** with capacities, arrow formula, eject on destroy/convert/≤ 20 % HP, town
    bell with the 25-tile rule.
22. **Starting layout generator** reproducing §4.3 (TC, 3 villagers, scout, 4+2+2 sheep,
    2 boar, 6 berries, 7+4+4 gold, 5+4 stone, 5 stragglers, 3–4 deer, 1 predator) with
    AoE2-style `min/max_distance` placement and `find_closest`, then clumped forests and
    scattered resources scaled by map area.

### 8.5 Rules that make it "an age-advancing RTS" (weeks 4–6)

23. Four ages with research gates; unit lines with upgrades; blacksmith/university techs.
24. **Conversion** as an interval lottery (1.25 s intervals, warm-up / guaranteed window,
    resistance levels, faith 0–100 % with regeneration); relics with 0.5 gold/s each.
25. **Trebuchet-style pack/unpack** as a generic two-state unit with a transition timer.
26. **Victory**: conquest, wonder/relic countdown in "years" (1 year = 5 game s; 200 years
    on ≤ 168² maps), score, time limit, last man standing, regicide, king of the hill.
27. **Diplomacy**: per-pair stance, allied victory, tribute with fee, locked teams.

### 8.6 Content tooling (after the core loop is fun)

28. **RMS-like map scripts**: a small declarative format with the same seven phases
    (player setup → lands → elevation → cliffs → terrain → connections → objects), random
    blocks and conditionals; implemented in Rust, seeded by the match seed.
29. **Triggers**: conditions and effects from §5.2 as data (start with the AoK core set:
    bring-to-area, own objects, destroy, timer, research, accumulate; effects: create,
    task, kill/remove, change ownership/diplomacy, chat, activate/deactivate, declare
    victory, change view, tribute, damage/HP/attack modifiers), evaluated in ID order
    once per tick, AND semantics, looping flag, no multiplayer-unsafe conditions.
30. **AI** as a rule system over sim queries (facts) and commands (actions), run inside
    the sim every N ticks so that it is deterministic and replayable; difficulty as
    compiled-in flags; taunt-driven ally commands.
31. **Taunts 1–42 and the cheat list** as content; the age-up fanfare, attack horn/bell
    with the 20×20 / 10 s battle throttle, villager task lines, idle counter.

### 8.7 Explicit non-goals for the first milestone

- Naval units, trade, markets, monks' healing, elevation combat bonus, walls with gates
  (walls yes, gates later), formations beyond line/box.
- Authoritative-server simulation (openage's model) — the server stays a relay/orderer.
- Mod/map transfer between peers (serve content from the server only — the 2026 DE CVE).

---

## 9. Sources

Primary and secondary sources consulted (no text copied verbatim):

- Paul Bettner & Mark Terrano, "1500 Archers on a 28.8: Network Programming in Age of
  Empires and Beyond", Gamasutra / Game Developer, 2001 —
  https://www.gamedeveloper.com/programming/1500-archers-on-a-28-8-network-programming-in-age-of-empires-and-beyond
- Wikipedia, "Age of Empires II" — https://en.wikipedia.org/wiki/Age_of_Empires_II
- Wikipedia, "Age of Empires II: Definitive Edition" —
  https://en.wikipedia.org/wiki/Age_of_Empires_II:_Definitive_Edition
- Wikipedia, "Genie Engine" — https://en.wikipedia.org/wiki/Genie_Engine
- Age of Empires Fandom wiki (ageofempires.fandom.com), pages: Taunt; Cheat code (Age of
  Empires II); Random map; Hotkey; Scenario Editor (Age of Empires II); Trigger; Line of
  Sight; Elevation; Speed; Conversion; Garrison; Repairing; Trebuchet; Unit stance; Unit
  formation; Victory; Diplomacy; Wonder (Age of Empires II); Relic (Age of Empires II);
  Player; Mini map; User interface; Town Bell; Multiplayer; Villager (Age of Empires II);
  Artificial intelligence; Campaign (Age of Empires II); Age of Empires II: The Age of
  Kings / The Conquerors / HD Edition / HD: The Forgotten / HD: The African Kingdoms / HD:
  Rise of the Rajas / Definitive Edition and its expansion pages (The Last Khans, Lords of
  the West, Dawn of the Dukes, Dynasties of India, The Mountain Royals, Victors and
  Vanquished, The Three Kingdoms, The Last Chieftains); map pages Arabia, Archipelago,
  Baltic, Black Forest, Coastal, Continental, Crater Lake, Gold Rush, Highland, Islands,
  Mediterranean, Migration, Rivers, Team Islands, Arena, Ghost Lake, Mongolia, Nomad (map),
  Oasis, Salt Marsh, Scandinavia —
  https://ageofempires.fandom.com/wiki/Age_of_Empires_II:_The_Age_of_Kings
- ageofempires.com news: "Pre-order The Last Chieftains" —
  https://www.ageofempires.com/news/pre-order-the-last-chieftains-aoeii-de/ ;
  "Celebrating Age in 2026" —
  https://ageofempires.com/news/celebrating-age-in-2026-where-were-at-and-where-were-going/
- Age of Empires support: "How do I create a multiplayer match in Age of Empires II:
  Definitive Edition" —
  https://support.ageofempires.com/hc/en-us/articles/360047306372 ; "Known Issues &
  Solutions" — https://support.ageofempires.com/hc/articles/360049490811
- Forgotten Empires, "RMS Features" (DE additions) —
  https://www.forgottenempires.net/age-of-empires-ii-definitive-edition/rms-features
- Zetnus, "Definitive Random Map Scripting Guide" (Google Doc; forum thread
  https://forums.ageofempires.com/t/definitive-random-map-scripting-guide/104902 ;
  document https://docs.google.com/document/d/1jnhZXoeL9mkRUJxcGlKnO98fIwFKStP_OBozpr0CHXo)
- `#lang aoe2-rms` documentation (Racket) — https://docs.racket-lang.org/aoe2-rms/
- openage (SFTtech) documentation — https://github.com/SFTtech/openage :
  `doc/code/architecture.md`, `doc/code/curves.md`, `doc/code/event_system.md`,
  `doc/code/time.md`, `doc/code/coordinate-systems.md`,
  `doc/code/game_simulation/README.md`, `doc/code/game_simulation/systems.md`,
  `doc/code/pathfinding/README.md`, `doc/media/terrain.md`, `doc/media/slp-files.md`,
  `doc/reverse_engineering/networking/01-general.md`, `02-header.md`, `03-sync.md`,
  `08-movement.md`, `doc/reverse_engineering/game_mechanics/pathfinding.md`,
  `monk_conversion.md`, `repair.md`, `garrison.md`, `attacking_alarm.md`, `town_bell.md`,
  `build_speed.md`, `selection.md`, `waypoints.md`, `building_placement.md`, `rates.md`
- openage blog, "New Gamestate 2020" (via planet.freegamedev.net) —
  https://planet.freegamedev.net/2020/08/15/new-gamestate-2020/
- KSneijders, AoE2ScenarioParser — trigger condition and effect datasets —
  https://github.com/KSneijders/AoE2ScenarioParser (`AoE2ScenarioParser/datasets/conditions.py`,
  `effects.py`)
- Stefan Kolb, "Age of Empires 2: The Conquerors — Savegame File Format Specification"
  (recorded-game format) — https://github.com/stefan-kolb/aoc-mgx-format
- AI Scripting Encyclopedia (airef) — https://airef.github.io/ ; Age of Empires forums,
  "Using goals to clean up AI scripts" —
  https://forums.ageofempires.com/t/using-goals-to-clean-up-ai-scripts/205241 ; "Three
  ways to get the AI to attack" —
  https://forums.ageofempires.com/t/three-ways-to-get-the-ai-to-attack/205476/4
- Elijah Emerson, "Crowd Pathfinding and Steering Using Flow Field Tiles", Game AI Pro —
  http://www.gameaipro.com/GameAIPro/GameAIPro_Chapter23_Crowd_Pathfinding_and_Steering_Using_Flow_Field_Tiles.pdf
- Glenn Fiedler, "Deterministic Lockstep" — https://gafferongames.com/post/deterministic_lockstep/
- Steam Community discussions on map sizes and sprite mirroring (community reports, used
  only for corroboration) — https://steamcommunity.com/app/221380/discussions/0/35219681616765718 ,
  https://steamcommunity.com/sharedfiles/filedetails/comments/909340743
- Liquipedia Age of Empires, "Maps" — https://liquipedia.net/ageofempires/Maps

Items marked **[unconfirmed]** in the text: the exact per-interval conversion chance and
the min/max interval counts (sources disagree, and the wiki's numeric templates did not
survive extraction); whether DE routes game traffic through relay servers; the
"8 facings from 5 stored" rule (documented by modders, not by Microsoft); exact SLP frame
counts per animation (vary per unit and are read from the `.dat`).
