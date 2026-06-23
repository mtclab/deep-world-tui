# Goods & materials — toward a living market

Design note. The sim ships **25 goods** (`Food Water Coin Herb Wood Stone Cloth
Iron Branches Cordage Tinder Nails Thatch Glass Tool Bandage Trap Hide Leather Coat
Salve Clay Pottery Charcoal Ale`). That is an *abstraction*, not an economy: one
"Food", one "Wood", one "Cloth" can't feel organic or let trades specialise. A
living region — even one province of Sorethel — needs the material world broken
into a real **production DAG**: raw → processed → component → finished, plus
consumables, trade-goods, and a thin rare/fantasy layer.

Researched against northern/Finnic material culture (bog-iron, tar/*terva*,
birch-bark/*tuohi*, *sahti*, furs, soapstone, reindeer products, amber, skis &
sledges) and Sorethel canon (oltzarin, deep-light, Mëräk pearls, Häl physic).

## Design rules (so it stays a game, not inventory soup)

1. **Every good has a source and a sink** — it is produced by a trade, consumed,
   used in a recipe, equipped, or has trade-value. No orphan goods.
2. **World-depth ≠ pack-weight.** The *settlement economy* tracks and trades the
   full catalogue (stock, prices, flows — the living economy runs without the
   player). The **player's hand-inventory surfaces a manageable slice**; bulk
   moves as settlement stock and caravans, not in the backpack.
3. **Region + season + people gate availability** — coast makes fish/salt/pearls,
   forest furs/tar/game, mountain ore/metal/stone, steppe hide/dried-meat, canopy
   salves/fibre; winter shifts to furs/dried/sledge, summer to fresh/berries. Ties
   to the seasons-as-driver and people-signature systems.
4. **Split the coarse buckets, keep the codes.** The 25 stay as categories or
   alias to a representative member; the catalogue refines them.

## Catalogue (~150, by tier)

### Raw — plant
Timber/logs · planks/boards · firewood · bark · **birch-bark (tuohi)** · withies ·
flax · hemp · nettle-fibre · wool (raw) · reeds · straw · hay/fodder · thatch-reed ·
resin · **tar/pitch (terva)** · moss · reindeer-lichen · grain (rye/barley/oats) ·
mahla (birch-sap) · culinary herbs · dye-plants (woad/madder/lichen).

### Raw — animal
Hide · rawhide · **furs/pelts** (fox, marten, wolverine, bear, squirrel, ermine) ·
wool · sinew · bone · antler · horn · feathers/down · tallow/fat · beeswax · honey ·
gut · fish-skin · roe.

### Raw — mineral
**Bog-iron ore** · copper-ore · tin-ore · stone · cut-stone · clay · sand · lime ·
**salt** · whetstone · flint · ochre/pigment · soapstone · **amber** · silver · gold ·
gemstone · **freshwater pearls (Mëräk)**.

### Processed / intermediate
Flour · meal · malt · yarn/thread · linen-cloth · woolen-cloth/**wadmal** · felt ·
dye · dyed-cloth · tanned-leather · charcoal · **iron bloom → bar → steel** · copper
ingot · bronze · nails · wire · rope/cordage · twine · net-twine · glass · bricks ·
lime-mortar · pitch · soap · lye · potash · parchment · ink · pottery-clayware ·
soapstone-ware · beeswax-rendered · lamp-oil (fish/seal).

### Consumables — food & drink
Bread · porridge/gruel · **roots/turnip** · cabbage/greens · peas/beans · berries
(lingon/bilberry/cloudberry) · mushrooms · nuts · eggs · milk · butter · curd ·
cheese · fresh-fish · **salt-fish · smoked-fish** · fresh-meat · salt-meat ·
**dried/smoked meat (reindeer)** · game · fowl · fat/lard · seaweed (Mëräk) · honey ·
salt · water · **ale · sahti · mead · berry-wine · mahla · buttermilk**.

### Consumables — fuel, light, care
Firewood · charcoal · peat · tinder · tallow-candle · wax-candle · resin-torch ·
lamp-oil · **herb · salve · bandage · poultice · tonic · birch-tar balm**.

### Finished — tools & implements
Axe · adze · saw · hammer · chisel · knife · sickle · scythe · hoe · spade · plough ·
awl · needle · shears · fishhook · **net** · trap · snare · auger · file · tongs ·
quern/millstone · churn · spindle · loom-parts · whetstone.

### Finished — weapons & war
Spear · war-axe · seax/sword · knife/dagger · bow · arrows · sling · shield ·
helmet · mail · leather-armour · spearheads · arrowheads.

### Finished — clothing & wear
Tunic · trousers · dress · **coat · cloak · fur-coat** · shoes/boots · **birch-bark
shoes (virsut)** · felt-boots · hat/cap · hood · mittens · socks/hose · belt · apron ·
brooch · comb.

### Finished — containers & household
Barrel/cask · bucket · **birch-bark container (rove)** · basket · sack · crate ·
chest · clay-pot · iron-cauldron · soapstone-pot · bowl · cup/mug · plate · lamp ·
distaff.

### Finished — building & transport
Beams · shingles · daub · glass-pane · cut-stone · **boat/skiff · oar · sail** ·
cart · **sledge/sled · skis · snowshoes** · harness · saddle · yoke · horseshoe.

### Trade-goods, luxury, value
Coin · silver · gold · **amber · pearls · luxury furs** · fine linen · dyed-cloth ·
glassware · soapstone-ware · jewellery/brooch · incense · spices (traded) · wax ·
honey · salt (as wealth) · **the Väylä have no universal coin — much trade is in
kind** (the goods *are* the money on most of the map).

### Knowledge & cult
Parchment · ink · rune-stone · carving · **cairn-stone (Khör)** · amulet/charm ·
incense · offering · candle · the Archive's tablets/records.

### Rare / fantasy layer (Conservation-bound — scarce, costly, god-touched)
**Oltzarin** ("the metal that remembers" — Sepät/Tzäkhar deep-iron) ·
**syvävalo** ("deep-light" phosphor mineral — Tzäkhar) · Mëräk deep-water sponges &
rare shells · Häl rare physic-herbs & potent salves · Khör härkä-hide · uncanny
spoils from uncanny beasts (ghost-stag antler, mire-light essence) — **never common,
always paid for** in the body or the rite; they are the edges of the world, not a
shop stock.

## How it plugs into the sim

- `ItemType` grows (or a `Good { kind, grade }` with a richer kind-set); the
  existing 25 become categories/aliases so saves and current recipes survive.
- The **recipe/production graph** (the living economy) gains the chains above —
  this is what makes the trades from `PROFESSIONS.md` meaningful (a charcoal-burner
  feeds the smith feeds the cutler; flax → spinner → weaver → dyer → tailor).
- **Settlements** stock/price/trade the full set by region+season+people; the
  player touches a slice. Pricing already exists per good — extend it.
- Suggested phasing: (1) split food/wood/cloth/iron into their members + add the
  core chains (~60 goods) — biggest realism win; (2) finished tools/clothing/
  containers as craftables; (3) trade-goods & luxury for the value economy;
  (4) the rare/fantasy layer last, kept scarce.

**Bottom line:** ~25 is a placeholder. A livable province wants on the order of
**120–160 goods** moving through real chains — most as settlement stock and trade,
a curated slice in the player's hands — gated by region, season, and people, with a
thin, costly fantasy layer at the edges.
