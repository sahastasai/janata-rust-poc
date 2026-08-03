# Add or change a screen

1. Find the reference route in `docs/parity/screens.csv`.
2. Add the Dioxus route and keep data access behind a typed service or signal.
3. Use the shared color/type/spacing tokens; do not introduce a one-off visual
   system for a single screen.
4. Add accessible names, focus order, reduced-motion behavior, and responsive
   layouts for narrow and wide viewports.
5. Add an interaction test that covers its primary action and failure state.
6. Capture desktop, mobile-web, Android, and iOS screenshots when applicable.

Keep route components small. Extract a component when it communicates a real
product concept—event card, center picker, message composer—not merely to move
lines into another file.
