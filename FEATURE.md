# Notes

- Implement esc cancel union operation for convenience (DONE)
- Tidy up the shader



## What to do with colors?

Options are as follows:

- Each operation gets its own color, which overrides the colors of the individual primatives. 
The downside of this are some wasted bytes (colors aren't needed on primatives).
- Initially set the colors the same, but blend the colors for union operations. This lets the 
user do some cool things, but is a little more challenging.

What happens for other CSG operations?

- For subtracts the primative that is subtracted has no color (it should have its color
field removed in the editor).

Given that subtracts require us to 'overwrite' the color information of primatives
it probably makes sense to delegate color logic to the CSG operation.
