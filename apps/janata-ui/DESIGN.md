# Janata Dioxus UI direction

## Subject, audience, and job

This shell is a proof of concept for Chinmaya Janata: a community member (or an
open-source contributor reviewing the migration) should understand the product,
move through its core destinations, and distinguish working local interactions
from backend-connected functionality that is still planned.

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

The portrait must provide context without implying endorsement of unmeasured
benchmark numbers. Benchmarks are labelled as methodology or pending evidence,
and all sample community content is visibly marked `Preview data`.

## Post-build critique checklist

- Check 360 px and 1280 px layouts for clipped navigation or horizontal scroll.
- Check every route by keyboard and verify a visible focus ring.
- Check that dark mode retains legible muted text and image treatment.
- Check `prefers-reduced-motion` and high-contrast / forced-colors behavior.
- Remove any flourish that competes with the saffron thread or portrait.
