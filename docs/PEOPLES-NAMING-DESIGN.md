# Peoples Naming Design — Canon-Grounded (5-Layer Architecture)

## Problem
The "peoples who stayed" used `-kansa` and `-väki` suffixes as their primary names.
But per canon (NAME_RESOLUTION, race_naming_reconciliation), these are **Arkit
scholarly reconstructions** — not what people actually call themselves. True
endonyms are opaque worn roots from diverse Finno-Ugric branches. The TUI must
reflect the full 5-layer naming system.

## Canon Sources
- `deep-world-history/src/docs/peoples/NAME_RESOLUTION_SIX_TRUE_ENDONYMS.md`
- `deep-world-history/src/docs/peoples/race_naming_reconciliation.md`
- `deep-world-history/src/docs/peoples/the_peoples_who_stayed_deep_cultural_profiles.md`
- `deep-world-history/src/docs/peoples/naming_conventions_and_endonyms.md`
- `deep-world-history/src/docs/peoples/naming_deep_dive_endonyms_and_exonyms.md`
- `deep-world-history/src/docs/peoples/deep_linguistic_history_and_migration.md`
- Novels: *The Accidental Lord* (ch02-ch06) — characters use SAST/trade names in dialogue

## Naming Architecture (5-layer, per canon)

Each people has up to 5 name layers:

| Layer | Name | Era | Who Uses It | Character |
|-------|------|-----|-------------|-----------|
| **0** | Proto-Endonym | Pre-Migration | Linguists only | Reconstructed from roots; opaque |
| **1** | True Endonym | Migration→present | The people themselves | Opaque worn root; speakers don't parse etymology |
| **2** | Arkit Scholarly Name | Pilgrimage→present | Arkit Archive, scholars | Finnish-morphology compounds (`-kansa`, `-väki`, etc.) |
| **3** | Pilgrimage Exonym | Pilgrimage→present | Official/religious contexts | God-derived names (Keurimä, Sampsari, etc.) |
| **4** | Daily Trade Name | Pilgrimage→present | Cross-cultural contact, trade | SAST forms (Metsik, Arkit, etc.) for SAST peoples; Arkit compounds for stayed peoples |

**Novels confirm**: Characters in *The Accidental Lord* say "Keuru shrine", "Masa preserve us",
"Sampsa calls whom Sampsa calls". They use SAST names (Metsik, Arkit) in daily speech.
The true endonyms (Čyrvä, Märät) appear only in ritual/scholarly contexts.

## TUI Method Mapping

| Method | Layer | When Used | Example |
|--------|-------|-----------|---------|
| `true_endonym()` | Layer 1 | Flavor text, private rituals, NPC self-introductions among kin | "Čyrvä", "Körvä", "Tzäkhar" |
| `arkit_name()` | Layer 2 | Archive books, scholarly NPC dialogue, written documents | "Metsik", "Porokansa", "Vaskiluuri" |
| `pilgrimage_exonym()` | Layer 3 | Temple/shrine encounters, formal religious contexts | "Keurimä", "Sampsari", "Oltkartako" |
| `label()` | Layer 4 | HUD, general display, cross-cultural encounter text | "Metsik", "Porokansa", "Tzäkhar" |
| `language_family()` | — | Classification, bias calculations | "Keurish", "Sampsaran", etc. |

### Display Logic per NPC Context

- **Same-people encounter**: NPC uses `true_endonym()` ("I am Čyrvä.")
- **Archive/scholarly context**: NPC uses `arkit_name()` ("The Archive records us as Porokansa.")
- **Temple/shrine context**: NPC uses `pilgrimage_exonym()` ("We dwell in Keuru's margin.")
- **Cross-cultural trade**: NPC uses `label()` — SAST names for SAST peoples, Arkit compounds for stayed peoples
- **HUD/status**: Always `label()`

## 14 Stayed-Peoples True Endonyms

Invented per "Diverse-Branch Opacity" principle (NAME_RESOLUTION canon):
each drawn from a distinct Finno-Ugric phonological profile within the family branch.

### Keurish Family (Sámic-profile: affricates, front-rounded vowels, open finals)

| Variant | Arkit Name | True Endonym | IPA | Phonological Notes |
|---------|-----------|-------------|-----|-------------------|
| Varhaiset | Varhaiset | **Körvä** | /ˈkør.væ/ | Front-rounded /ø/ + open /æ/; Sámic markers |
| Metsareunat | Metsäreunat | **Pyršä** | /ˈpyr.ʃæ/ | /y/ + postalveolar /ʃ/; northern forest sound |
| Porokansa | Porokansa | **Tuorva** | /ˈtwor.va/ | Labiovelar /w/ onset; cold-country vocalism |
| Koskimetsä | Koskimetsä | **Jälky** | /ˈjæl.ky/ | /æ/ + /y/; rapids-river phonology |

### Sampsaran Family (Mordvinic-profile: front /æ/, final /t/)

| Variant | Arkit Name | True Endonym | IPA | Phonological Notes |
|---------|-----------|-------------|-----|-------------------|
| Muistikansa | Muistikansa | **Särät** | /ˈsæ.ræt/ | Double /æ/, dental /t/; Volgaic endonym shape |
| Taulukansa | Taulukansa | **Velmät** | /ˈvel.mæt/ | /e/+/æ/; Mordvinic collective marker |
| Kirjakansa | Kirjakansa | **Tärent** | /ˈtæ.rent/ | Front /æ/; /nt/ cluster; river-valley dialect |

### Oltkar Family (Ugric/Khanty-profile: /w/ onset, /y/, final dental)

| Variant | Arkit Name | True Endonym | IPA | Phonological Notes |
|---------|-----------|-------------|-----|-------------------|
| Takoväki | Takoväki | **Wonśyt** | /ˈwon.ʃyt/ | /w/ + /y/; eastern Ugric markers |

### Masaran Family (Permic/Komi-profile: /v/, /y/, final /i/)

| Variant | Arkit Name | True Endonym | IPA | Phonological Notes |
|---------|-----------|-------------|-----|-------------------|
| Rantaväki | Rantaväki | **Vylri** | /ˈvyl.ri/ | /v/+/y/+/i/; Permic endonym ending |
| Saariväki | Saariväki | **Kylmi** | /ˈkyl.mi/ | /y/+/i/; island-dialect Permic |
| Hiekkakävelijät | Hiekkakävelijät | **Tyrväi** | /ˈtyr.væi/ | Permic with southern coastal blend |

### Kukresh Family (Permic/Udmurt-profile: /k/, /ʃ/, /m/)

| Variant | Arkit Name | True Endonym | IPA | Phonological Notes |
|---------|-----------|-------------|-----|-------------------|
| Härämäki | Härämäki | **Kišmäs** | /ˈkiʃ.mæs/ | Postalveolar /ʃ/; front /æ/; Udmurt shape |
| Jämäväki | Jämäväki | **Hoskam** | /ˈhoʃ.kam/ | /ʃ/ + /m/ final; deep-valley vocalism |
| Pohjaväki | Pohjaväki | **Väškam** | /ˈvæʃ.kam/ | Front /æ/ + /ʃ/ + /m/; depth-register Udmurt |

## Complete PeopleKind Table (25 variants)

### 6 SAST Peoples
| Variant | label() | true_endonym() | arkit_name() | pilgrimage_exonym() | language_family() | Patron |
|---------|---------|----------------|--------------|---------------------|-------------------|--------|
| Metsik | Metsik | Čyrvä | Metsik | Keurimä | Keurish | Keuru |
| Arkit | Arkit | Märät | Arkit | Sampsari | Sampsaran | Sampsa |
| Sepat | Sepät | Wosyt | Sepät | Sepät | Oltkar | Oltzed |
| Ahjo | Ahjo | Njumka | Ahjo | Iltkari | Oltkar | Oltzed |
| Vayla | Väylä | Vylti | Väylä | Masari | Masaran | Masa |
| Laakso | Laakso | Kiškam | Laakso | Kukreva | Kukresh | Kukri |

### 14 Stayed Peoples
| Variant | label() | true_endonym() | arkit_name() | pilgrimage_exonym() | language_family() | Patron |
|---------|---------|----------------|--------------|---------------------|-------------------|--------|
| Varhaiset | Varhaiset | Körvä | Varhaiset | Perikansan | Keurish | All Five |
| Metsareunat | Metsäreunat | Pyršä | Metsäreunat | Keurunreunat | Keurish | Keuru |
| Porokansa | Porokansa | Tuorva | Porokansa | Keuruporo | Keurish | Keuru |
| Koskimetsa | Koskimetsä | Jälky | Koskimetsä | Keurukoski | Keurish | Keuru |
| Muistikansa | Muistikansa | Särät | Muistikansa | Sampsamuisti | Sampsaran | Sampsa |
| Taulukansa | Taulukansa | Velmät | Taulukansa | Sampsataulu | Sampsaran | Sampsa |
| Kirjakansa | Kirjakansa | Tärent | Kirjakansa | Sampsakirja | Sampsaran | Sampsa |
| Takovaki | Takoväki | Wonśyt | Takoväki | Oltkartako | Oltkar | Oltzed |
| Rantavaki | Rantaväki | Vylri | Rantaväki | Masaranta | Masaran | Masa |
| Saarivaki | Saariväki | Kylmi | Saariväki | Masasaari | Masaran | Masa |
| Hiekkakavelijat | Hiekkakävelijät | Tyrväi | Hiekkakävelijät | Masahiekka | Masaran | Masa |
| Haramaki | Härämäki | Kišmäs | Härämäki | Kukriharma | Kukresh | Kukri |
| Jamavaki | Jämäväki | Hoskam | Jämäväki | Kukrijämä | Kukresh | Kukri |
| Pohjavaki | Pohjaväki | Väškam | Pohjaväki | Kukripohja | Kukresh | Kukri |

### 5 Non-Human Peoples
| Variant | label() | true_endonym() | arkit_name() | pilgrimage_exonym() | language_family() | Patron |
|---------|---------|----------------|--------------|---------------------|-------------------|--------|
| Tzakhar | Tzäkhar | Tzäkhar | Vaskiluuri | Vaskiluuri | Deep-Isolate | Kukri |
| Merak | Mëräk | Mëräk | Iltäkälä | Iltäkälä | Coastal-Isolate | Masa |
| Shear | She'ar | She'ar | Muraskala | Muraskala | Desert-Isolate | None |
| Hal | Häl | Häl | Khör | Khör | Canopy-Isolate | Keuru |
| Khor | Khör | Khör | Khmört | Khmört | Steppe-Isolate | Sampsa |

## Population Weights (charts.ron)

| Category | Target % | Rationale |
|----------|----------|-----------|
| SAST god-peoples | ~15-20% | Canon says minorities |
| Peoples who stayed | ~70-80% | Canon says majority common folk |
| Non-human | ~5-10% | Canon says rare, biome-dependent |

Within "stayed": extinct peoples weighted lower (5-8%), endangered/thriving higher (15-25%).

## Implementation Notes

- `PeopleKind` enum has ASCII-safe variant names (no ä, ö, š, etc.)
- `label()` returns the daily-use trade name (Layer 4 for SAST, Layer 2 for stayed, Layer 1 for non-human)
- `true_endonym()` returns the opaque self-name root (Layer 1) for ALL peoples
- `arkit_name()` returns the Arkit Archive scholarly compound name (Layer 2)
- `pilgrimage_exonym()` returns the god-derived formal name (Layer 3)
- `language_family()` returns the linguistic classification string
- `from_name()` matches all name layers (ASCII and Unicode)
- `description()` uses English short-forms from canon
- `greeting_to()` uses context-appropriate names per NPC encounter
- `bias_toward()` uses `effective_bias()` — never raw `bias_toward()`
- `patron_god()` returns `None` for She'ar (canon: no patron god)