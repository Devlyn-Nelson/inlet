- make it so combos are interrupted by other inputs.
- fix clashing:
  - Buffered clash seems to work, but Sorted currently allows lower priority bindings to receive an invalid true signal because if it is ordered to poll the clasher first

# Maybe

## Dedup input checks

The `ClashHandler` and `InputBindings` have some intermix issues:
- The `InputBindings` will be iterated through in an arbitrary order making un-buffered clash handling hard without complex ordering system.
- Inputs that clash will be checked multiple times. this is because the `ClashHandler` relies on the `InputBindings` iteration loop feeding when inputs happen.

If the `ClashHandler` held ALL possible clashables then polled states from bevy directly:
- Inputs that clash would not get polled multiple times.
- We could store a key to the bindings and their chord length to enable non-buffered clash detection to work without sorting the bindings.