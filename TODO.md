- It seems like the BufferClashable sometimes buffers non-clashing inputs (press F1 in visualizer a couple times, must pass BufferAll setting at least once).
- Combo progression None doesn't activate combos if the previous buttons are still held.

# Maybe

## Release to lower priority

Using the `visualizer` example if you:
- hold the `A` key ->  left box go green.
- hold the `S` key -> left box red, middle box green.
- released `S` key -> all red.

adding a setting that allows us to keep current functionality above and add the option for the last step (released `S` key) to cause the middle box to go red and the left box to go green again.
