# Wars — Game Design

What Wars replicates from the classic age-advancing RTS, with the numbers the code
uses. Research notes with the original values are in `docs/research/`. The
authoritative tables are `crates/data/src/{objects,techs,civs}.rs`; this document
explains the rules around them and the simplifications of the first version.

## The match

- **Ages**: Dark → Feudal (500 food, 130 s) → Castle (800 food 200 gold, 160 s) →
  Imperial (1000 food 800 gold, 190 s), researched at the town centre. Age-ups have no
  building prerequisites in v1 (the classic "two buildings of the age" rule is a later
  refinement).
- **Resources**: food, wood, gold, stone. Villagers carry 10 (+3 Wheelbarrow, +4 Hand
  Cart, +1 Heavy Plow) and drop off at the town centre, mill (food), lumber camp
  (wood), mining camp (gold/stone) or dock (fish). Base gather rates per second:
  berries 0.31, sheep 0.33, hunt 0.41, farm 0.32, fish 0.43, wood 0.39, gold 0.38,
  stone 0.36. Trees hold 100 wood, gold piles 800, stone 350, bushes 125, sheep 100,
  deer 140, boar 340 (boars fight back), farms 175 food and auto-reseed for 60 wood.
- **Population**: 200 cap (+10 Goths). Houses +5, town centre +5 (+5 more for Chinese),
  castle +20.
- **Starting layout** (random maps): town centre, 3 villagers (+3 Chinese), a scout,
  8 sheep, 2 boars, 4 deer, 6 berry bushes, gold 7+4, stone 5+4, two forests and
  stragglers. Start stockpile 200/200/100/200 (deathmatch 20000/20000/10000/5000,
  regicide 500/500/150/150 + king + castle + 4 villagers).
- **Victory**: standard = conquest or wonder (10 minutes after completion) or all
  relics held for 10 minutes; conquest only; score at a time limit; scenario triggers.
  A player is out when they have no villagers, no military and no production
  buildings (regicide: when the king dies). Teams win together.
- **Game modes**: random map, deathmatch, regicide, scenario. Speeds 25–300 %
  (host-controlled online). Cheats when the host allows them: `cheese steak jimmy's`,
  `lumberjack`, `robin hood`, `rock on`, `marco`/`polo`, `aegis`, `i r winner`,
  `torpedo`, `how do you turn this on` (spawns the Cobra Car).

## Simulation

- 20 ticks/second, fixed-point 16.16 world coordinates in tiles, units move on a
  tile grid with sub-tile positions and 8-connected A* paths (recomputed every 1.5 s or
  when blocked, capped at 12,000 expansions). Units do not block each other; buildings,
  trees and resource piles do.
- **Combat**: `damage = max(1, Σ_classes max(0, attack_c − armour_c))` over the
  armour classes the target has (melee, pierce and the bonus classes: infantry,
  cavalry, archer, siege, building, ship, camel, spearman, monk, unique unit, eagle,
  ram, elephant, gunpowder, cavalry archer, skirmisher, tree). Ranged attacks fly as
  projectiles; a moving target is hit with `accuracy/2` % unless the shooter has
  Ballistics. Mangonels, trebuchets and bombards deal area damage (radius 1 tile).
  Minimum range applies. Buildings under construction have no armour. Garrisoned
  town centres, castles and towers fire one extra (half-strength) arrow per unit
  inside; garrisoned units heal 2 HP/s.
- **Villagers**: gather → drop off → continue; when a node runs out they pick the
  nearest node of the same kind within 10 tiles. Hunting attacks the animal first;
  sheep are captured by the first player unit within 4 tiles. Building speed: one
  builder builds in the listed time; extra builders add diminishing returns.
  Repairing costs half the wood/stone over the repair.
- **Monks** convert in 4–10 s (plus the target's resistance) with a 28 % roll each
  second, rest 60 s afterwards; Redemption/Atonement gate buildings/siege/monks; monks
  heal nearby allies when idle; relics go to the monastery and yield 30 gold/min each.
- **Trade**: carts/cogs between your market/dock and an ally's; gold per trip ≈
  ¾ of the distance in tiles (cap 120).
- **Market**: 100 units per transaction, 30 % fee (5 % Saracens, Guilds 15 %), prices
  move 3 per transaction (floor 20). Tribute has no fee.
- **Fog of war**: explored/visible per player, recomputed every 5 ticks, allies share
  vision. Buildings under construction see half their line of sight.
- **Stances**: aggressive (chase anything in line of sight), defensive (engage within
  range, return), stand ground, no attack. Attack-move, patrol, follow, guard,
  garrison, ungarrison, rally points (point or object), shift-queued commands.
- **Formations** are cosmetic in v1 (a square spread on group moves).

## Content

- **13 civilisations** with bonuses, team bonus, unique unit (+ elite), one unique
  technology and tech-tree gaps: Britons, Byzantines, Celts, Chinese, Franks, Goths,
  Japanese, Mongols, Persians, Saracens, Teutons, Turks, Vikings. Values are in
  `civs.rs`; age-staged bonuses are flattened to a single value.
- **Units**: villager, trade cart, king; militia line (5), spearman line (3), archer
  line (3), skirmisher line (2), cavalry archer line (2), scout line (3), knight line
  (3), camel line (2), monk, battering ram line (3), mangonel line (3), scorpion line
  (2), bombard cannon, trebuchet; fishing ship, transport, trade cog, galley line (3),
  fire ship line (2), cannon galleon, longboat; 13 unique units with elite versions;
  the Cobra Car.
- **Buildings**: town centre, house, mill, farm, lumber camp, mining camp, dock,
  barracks, archery range, stable, blacksmith, market, monastery, siege workshop,
  university, castle, outpost, watch tower line (3), bombard tower, palisade wall,
  stone wall line (2), gate, wonder.
- **Technologies** (~90): the three ages, blacksmith (15), economy (12), barracks
  (8), archery range (6), stable (7), siege workshop (5), dock (6), market (4),
  monastery (10), university (12), castle (3 + 13 elite upgrades + 14 unique techs).

## Simplifications in this version

Documented so nobody mistakes them for bugs:

1. Gates are 1×1 and walkable by everyone (walls still block); no fortified gate.
2. No elevation or cliffs in gameplay (Highland shows hills visually only); no
   trample damage; no demolition ships, petards, fish traps or farm queues (farms
   reseed automatically instead).
3. Market prices are per player, not global; tribute has no fee.
4. Shared allied vision from the start (no Cartography); no diplomacy changes after
   start when teams are locked (default).
5. Age-up prerequisites and the "unique unit at the castle only" rule are enforced,
   but the two-buildings rule for ages is not.
6. Conversion uses a simplified faith model; Heresy adds resistance instead of
   killing converted units.
7. Chu Ko Nu fire one stronger bolt instead of a volley; Berserk regeneration and
   Logistica trample are approximated by HP/attack bonuses.
8. Formations are cosmetic; units do not collide with each other.
9. Civ bonuses that scale per age are applied at a flat average value.

## Controls (desktop)

Classic layout: resources top-left, minimap bottom-right, command grid bottom-left,
selection panel bottom-centre, idle-villager button. Hotkeys: `H` town centre, `.`
idle villager, `B` build menu, `A` attack-move, `S` stop, `D` delete (with
confirmation), `G` garrison, `Ctrl+1..9` set group, `1..9` recall, `Space` go to last
event, `F3` pause, `+`/`-` speed, `F11` quick save, `Enter` chat, `Esc` cancel.
