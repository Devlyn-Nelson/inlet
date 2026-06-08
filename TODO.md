- Interrupting Combos when an invalid button is pressed, with setting to disable interrupts.
- fix unbuffered clashing. (Dedup Input Checks) could make this easier and better.
- Add mock value bindings.

# Maybe

## Release to lower priority

Using the `clash-visualizer` example if you:
- hold the `A` key ->  left box go green.
- hold the `S` key -> left box red, middle box green.
- released `S` key -> all red.

adding a setting that allows us to keep current functionality above and add the option for the last step (released `S` key) to cause the middle box to go red and the left box to go green again.

## Dedup Input Checks

The `ClashHandler` and `InputBindings` have some intermix issues:
- The `InputBindings` will be iterated through in an arbitrary order making un-buffered clash handling hard without complex ordering system.
- Inputs that clash will be checked multiple times. this is because the `ClashHandler` relies on the `InputBindings` iteration loop feeding when inputs happen.

If the `ClashHandler` held ALL possible clashables then polled states from bevy directly:
- Inputs that clash would not get polled multiple times.
- We could store a key to the bindings and their chord length to enable non-buffered clash detection to work without sorting the bindings.