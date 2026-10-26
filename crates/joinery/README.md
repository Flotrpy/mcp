# joinery

An MCP server that does the arithmetic of woodworking: parametric furniture, cut lists, sheet nesting and shelf-sag checks. It is fully offline and deterministic. The model proposes a design, and joinery makes sure it can actually be cut and won't sag.

## Install

```sh
cargo install --git https://github.com/Flotrpy/mcp joinery
```

```json
{ "mcpServers": { "joinery": { "command": "joinery-mcp" } } }
```

## Tools

| Tool | What one call gives you |
|---|---|
| `design_furniture` | A bookcase, cabinet or drawer from outside dimensions: every part sized for its dados, rabbets and grooves, a merged cut list with board feet, a glue-up order, and a sag check for every shelf |
| `check_shelf_sag` | Deflection (immediate and long-term creep) against the 1/32″-per-foot limit, plus the longest span and thinnest stock that would pass |
| `nest_parts` | Guillotine layout on plywood sheets with saw kerf and grain direction, yield %, and a to-scale SVG cut diagram |
| `list_species` | The species and sheet goods it knows, with stiffness (MOE) and creep factors |

## What makes it different
- **It checks physics, not just geometry.** Shelf sag uses beam theory, `δ = 5WL³/384EI`, with species stiffness from the USDA Wood Handbook and a creep factor for long-term loads. When a shelf fails, it tells you the fix.
- **Its layouts are ones a saw can cut.** Every cut in a guillotine layout runs edge to edge. Kerf is reserved between parts, and grain-locked parts are never rotated.
- **Glue-up order comes from the joint graph.** Grooves go before the box closes, the back goes in after squaring, and shelf pins go in last.
- **Every length uses one `units` argument** (`mm` or `in`), for input and output alike.

## Example

> "Design a 36 × 72 × 12 inch bookcase in MDF with four adjustable shelves."

`design_furniture` answers: *"bookcase with 9 parts. 1 shelf type(s) will sag visibly in MDF; see shelf_checks advice."* The advice suggests a shorter span, thicker stock or a hardwood edge strip, and notes that MDF creeps.
