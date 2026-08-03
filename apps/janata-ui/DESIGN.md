# Janata Dioxus UI direction

## Subject, audience, and job

This integrated proof of concept is for Chinmaya Janata: a community member (or
an open-source contributor reviewing the migration) should understand the
product, move through its core destinations, and distinguish the proven
read-only Worker/private-reducer foundation from production identity and live UI
adapters that remain gated.

## Tokens

- `Jnana ink` `#191A18`: primary text and the dark surface.
- `Paper light` `#F7F8F4`: daylight background, intentionally cooler than the
  existing cream surfaces so saffron remains the visual anchor.
- `Saffron thread` `#E8862A`: the existing Janata action color and path marker.
- `Deep saffron` `#9C4317`: accessible action text, pressed states, and emphasis.
- `River blue` `#285B6D`: secondary data and community context.
- `Quiet stone` `#D9DDD6`: borders and spatial structure.

Light and dark modes use these same semantic roles. The dark mode is charcoal,
not black; orange is slightly lifted rather than converted to a neon accent.

## Typography

- **Inclusive Sans** (bundled from the reviewed Janata asset library) is the
  restrained display face for identity, page titles, and human names.
- **Inter** is the body face for explanatory copy and controls.
- Inter with tabular numerals is the utility voice for dates and benchmark data.

The scale is compact and mobile-led: 12 utility, 14 supporting, 16 body, 22
section heading, and a fluid 42–72 hero. Display weight is never used merely to
fill a card.

## Layout exploration

App shell, desktop:

```text
┌──────────────┬──────────────────────────────────────┐
│ JANATA       │ page title                  POC LIVE │
│ saffron path │──────────────────────────────────────│
│ Home         │                                      │
│ Discover     │        routed page content           │
│ Feed         │        max-width reading field       │
│ Connect      │                                      │
│ Profile      │                                      │
│──────────────│                                      │
│ Bench / Docs │                                      │
└──────────────┴──────────────────────────────────────┘
```

App shell, mobile:

```text
┌──────────────────────────────┐
│ JANATA             POC       │
│ page content                 │
│                              │
├──────────────────────────────┤
│ Home  Discover  Feed  Connect│
└──────────────────────────────┘
```

Landing route:

```text
┌─────────────────────────────────────────────────────┐
│ A community is not a feed.       portrait /         │
│ It is a practice.                 saffron path       │
│ [Enter the POC] [Read the note]                      │
├──────────────────────┬──────────────────────────────┤
│ what already works   │ what still needs integration │
└──────────────────────┴──────────────────────────────┘
```

The routed application shell is selected over a generic marketing page because
the proof needs to show information density, navigation, and mobile behavior.

## Signature

The signature is a single continuous **saffron thread**. It begins beside Swami
Chinmayananda's portrait on the landing page, becomes the active-route marker in
the shell, and reappears only where a user makes a meaningful choice. It encodes
the POC's thesis: implementation can change while purpose remains continuous.

Motion is limited to the thread entering once and short control transitions.
Reduced-motion preferences remove both.

## Self-critique before build

An early direction used a cream ground, serif quotations, and ornamental orange
cards. That combination could belong to almost any cultural non-profit and is a
known template default. It was revised to a cooler paper background, the
project's existing sans-serif identity, asymmetrical portrait crop, and one
functional orange line. Rounded cards are reserved for actual grouped controls
or data; structural areas use spacing and rules.

The portrait must provide context without implying endorsement of benchmark
claims. Measured local evidence always carries its method and limitations, and
all sample community content is visibly marked `Preview data`.

## Post-build critique checklist

- Check 360 px and 1280 px layouts for clipped navigation or horizontal scroll.
- Check every route by keyboard and verify a visible focus ring.
- Check that dark mode retains legible muted text and image treatment.
- Check `prefers-reduced-motion` and high-contrast / forced-colors behavior.
- Remove any flourish that competes with the saffron thread or portrait.

## Integrated evidence update

The benchmark screen's single job is to help a technical reviewer judge the POC
without hiding an unfavorable result. Its layout is a measured ledger rather
than a celebratory dashboard:

```text
┌───────────────────────────────────────────────────────────────┐
│ The local result is mixed.                                    │
│ Dioxus: bundle + landing LCP     Hono: health throughput/p95  │
├───────────────────────────────────────────────────────────────┤
│ Browser and bundle evidence (React │ Dioxus │ result)         │
├───────────────────────────────────────────────────────────────┤
│ Worker health evidence (Hono │ workers-rs │ result)           │
├───────────────────────────────────────────────────────────────┤
│ Limitations live beside the numbers, not in a distant note.   │
└───────────────────────────────────────────────────────────────┘
```

The risk was allowing large favorable numbers to become a generic performance
scoreboard. The revision uses tabular numerals, quiet rules, full byte counts,
and equal visual weight for the workers-rs loss. Saffron identifies the Dioxus
browser evidence; river blue identifies the Hono backend result. Neither color
means universally better.
