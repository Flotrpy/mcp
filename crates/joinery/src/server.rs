//! MCP tool surface for `joinery`.
//!
//! Four tools, each designed so that one call answers a whole question:
//! `design_furniture` returns parts, cut list, assembly order *and* a sag
//! check for every shelf, so the model does not need to chain calls.

use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ServerConfig},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    assembly, cutlist,
    furniture::{self, Carcass, Design},
    model::{self, Part, SPECIES, Units},
    nesting::{self, Sheet},
    sag::{self, LoadPreset, SagReport, Shelf},
    svg,
};
use mcp_kit::result::{invalid_params, structured, tool_error};

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FurnitureKind {
    Bookcase,
    Cabinet,
    Drawer,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DesignArgs {
    /// What to build.
    pub kind: FurnitureKind,
    /// Outside width.
    pub width: f64,
    /// Outside height.
    pub height: f64,
    /// Outside depth.
    pub depth: f64,
    /// Thickness of the main stock, e.g. 18 (mm) or 0.75 (in).
    pub stock_thickness: f64,
    /// Back panel (or drawer bottom) thickness. Defaults to a third of the stock.
    pub back_thickness: Option<f64>,
    /// Bookcase only: shelves glued into dados.
    #[serde(default)]
    pub fixed_shelves: u32,
    /// Bookcase only: shelves on pins.
    #[serde(default)]
    pub adjustable_shelves: u32,
    /// Unit for every length in and out. Default `mm`.
    #[serde(default)]
    pub units: Units,
    /// Species id for the shelf sag check (see `list_species`). Default `birch-ply`.
    pub species: Option<String>,
    /// What the shelves will hold, for the sag check. Default `books`.
    pub shelf_load: Option<LoadPreset>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SagArgs {
    /// Distance between supports.
    pub span: f64,
    /// Front-to-back depth of the shelf.
    pub depth: f64,
    pub thickness: f64,
    #[serde(default)]
    pub units: Units,
    /// Species id, e.g. `red-oak`, `birch-ply`, `mdf`.
    pub species: String,
    /// Total load in kilograms. Give this or `load_preset`.
    pub load_kg: Option<f64>,
    /// Typical contents; the load scales with the span.
    pub load_preset: Option<LoadPreset>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PartInput {
    pub name: String,
    /// Along the grain.
    pub length: f64,
    pub width: f64,
    #[serde(default = "one")]
    pub quantity: u32,
    /// Keep grain along the sheet length (no rotation). Default true.
    #[serde(default = "yes")]
    pub grain_locked: bool,
}

fn one() -> u32 {
    1
}
fn yes() -> bool {
    true
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NestArgs {
    pub parts: Vec<PartInput>,
    /// Sheet length (along the face grain). Default 2440 mm / 96 in.
    pub sheet_length: Option<f64>,
    /// Default 1220 mm / 48 in.
    pub sheet_width: Option<f64>,
    /// Saw blade kerf. Default 3.2 mm / 1/8 in.
    pub kerf: Option<f64>,
    #[serde(default)]
    pub units: Units,
    /// Include a to-scale SVG cut diagram in the result. Default true.
    #[serde(default = "yes")]
    pub include_svg: bool,
}

#[derive(Debug, Serialize)]
struct DesignOutput {
    design: Design,
    cut_list: cutlist::CutList,
    assembly: Vec<assembly::Step>,
    shelf_checks: Vec<ShelfCheck>,
}

#[derive(Debug, Serialize)]
struct ShelfCheck {
    part: String,
    report: SagReport,
}

#[derive(Debug, Clone)]
pub struct Joinery {
    tool_router: ToolRouter<Self>,
}

impl Default for Joinery {
    fn default() -> Self {
        Self::new()
    }
}

fn positive(name: &str, v: f64) -> Result<f64, ErrorData> {
    if v.is_finite() && v > 0.0 {
        Ok(v)
    } else {
        Err(invalid_params(format!("`{name}` must be a positive number, got {v}")))
    }
}

fn lookup_species(id: &str) -> Result<&'static model::Species, CallToolResult> {
    model::species(id).ok_or_else(|| {
        let known: Vec<_> = SPECIES.iter().map(|s| s.id).collect();
        tool_error(format!("Unknown species `{id}`. Use one of: {}.", known.join(", ")))
    })
}

#[tool_router]
impl Joinery {
    pub fn new() -> Self {
        Self { tool_router: Self::tool_router() }
    }

    #[tool(
        description = "Design a bookcase, cabinet or drawer from outside dimensions. Returns every part sized for its joinery, a merged cut list with board feet, a glue-up order, and a sag check for each shelf.",
        annotations(title = "Design furniture", read_only_hint = true, open_world_hint = false)
    )]
    pub fn design_furniture(&self, Parameters(a): Parameters<DesignArgs>) -> Result<CallToolResult, ErrorData> {
        let u = a.units;
        let mm = |name: &str, v: f64| positive(name, v).map(|v| u.to_mm(v));
        let stock = mm("stock_thickness", a.stock_thickness)?;
        let back = match a.back_thickness {
            Some(b) => mm("back_thickness", b)?,
            None => stock / 3.0,
        };
        let (w, h, d) = (mm("width", a.width)?, mm("height", a.height)?, mm("depth", a.depth)?);
        if w <= 2.0 * stock || d <= back {
            return Ok(tool_error("The case is too small for its own stock: width must exceed two stock thicknesses and depth must exceed the back."));
        }
        let carcass = Carcass { width_mm: w, height_mm: h, depth_mm: d, stock_mm: stock, back_mm: back };
        let design = match a.kind {
            FurnitureKind::Bookcase => furniture::bookcase(carcass, a.fixed_shelves, a.adjustable_shelves),
            FurnitureKind::Cabinet => furniture::cabinet(carcass),
            FurnitureKind::Drawer => furniture::drawer(w, h, d, stock, back),
        };

        let species = match lookup_species(a.species.as_deref().unwrap_or("birch-ply")) {
            Ok(s) => s,
            Err(e) => return Ok(e),
        };
        let preset = a.shelf_load.unwrap_or(LoadPreset::Books);
        let shelf_checks = design
            .parts
            .iter()
            .filter(|p| p.name.contains("shelf"))
            .map(|p| {
                let span_ft = p.length_mm / model::MM_PER_INCH / 12.0;
                ShelfCheck {
                    part: p.name.clone(),
                    report: sag::check(
                        Shelf {
                            span_mm: p.length_mm,
                            depth_mm: p.width_mm,
                            thickness_mm: p.thickness_mm,
                            load_kg: preset.lb_per_ft() * span_ft / sag::LB_PER_KG,
                        },
                        species,
                    ),
                }
            })
            .collect::<Vec<_>>();

        let failing = shelf_checks.iter().filter(|c| !c.report.passes).count();
        let summary = format!(
            "{} with {} parts. {}",
            design.kind,
            design.parts.iter().map(|p| p.quantity).sum::<u32>(),
            if shelf_checks.is_empty() {
                "No shelves to check.".to_string()
            } else if failing == 0 {
                format!("All {} shelf types pass the 1/32\" per foot sag limit in {}.", shelf_checks.len(), species.name)
            } else {
                format!("{failing} shelf type(s) will sag visibly in {}; see shelf_checks advice.", species.name)
            }
        );
        let out = DesignOutput {
            cut_list: cutlist::build(&design.parts, u == Units::In),
            assembly: assembly::steps(&design),
            shelf_checks,
            design,
        };
        structured(summary, &out)
    }

    #[tool(
        description = "Check whether a shelf will sag visibly (limit 1/32\" per foot, including long-term creep). Returns deflection, utilisation and the longest span / thinnest stock that would pass.",
        annotations(title = "Check shelf sag", read_only_hint = true, open_world_hint = false)
    )]
    pub fn check_shelf_sag(&self, Parameters(a): Parameters<SagArgs>) -> Result<CallToolResult, ErrorData> {
        let u = a.units;
        let span = u.to_mm(positive("span", a.span)?);
        let depth = u.to_mm(positive("depth", a.depth)?);
        let thickness = u.to_mm(positive("thickness", a.thickness)?);
        let species = match lookup_species(&a.species) {
            Ok(s) => s,
            Err(e) => return Ok(e),
        };
        let load_kg = match (a.load_kg, a.load_preset) {
            (Some(kg), _) => positive("load_kg", kg)?,
            (None, Some(p)) => p.lb_per_ft() * (span / model::MM_PER_INCH / 12.0) / sag::LB_PER_KG,
            (None, None) => return Ok(tool_error("Pass `load_kg` (total weight) or `load_preset` (books, dishes, records, decor).")),
        };
        let r = sag::check(Shelf { span_mm: span, depth_mm: depth, thickness_mm: thickness, load_kg }, species);
        let summary = format!(
            "{}: {:.2} mm long-term sag vs {:.2} mm limit ({:.0}% of limit).",
            if r.passes { "Passes" } else { "Fails" },
            r.long_term_deflection_mm,
            r.limit_mm,
            r.utilisation * 100.0
        );
        structured(summary, &r)
    }

    #[tool(
        description = "Nest rectangular parts onto stock sheets with saw kerf and grain direction respected. Returns placements, sheets used, yield % and an optional to-scale SVG cut diagram.",
        annotations(title = "Nest parts on sheets", read_only_hint = true, open_world_hint = false)
    )]
    pub fn nest_parts(&self, Parameters(a): Parameters<NestArgs>) -> Result<CallToolResult, ErrorData> {
        if a.parts.is_empty() {
            return Ok(tool_error("Pass at least one part in `parts`."));
        }
        let u = a.units;
        let (default_l, default_w, default_kerf) = match u {
            Units::Mm => (2440.0, 1220.0, 3.2),
            Units::In => (96.0, 48.0, 0.125),
        };
        let sheet = Sheet {
            length_mm: u.to_mm(positive("sheet_length", a.sheet_length.unwrap_or(default_l))?),
            width_mm: u.to_mm(positive("sheet_width", a.sheet_width.unwrap_or(default_w))?),
        };
        let kerf = u.to_mm(a.kerf.unwrap_or(default_kerf).max(0.0));
        let mut parts = Vec::with_capacity(a.parts.len());
        for p in &a.parts {
            let mut part = Part::new(
                p.name.clone(),
                u.to_mm(positive("length", p.length)?),
                u.to_mm(positive("width", p.width)?),
                0.0,
                p.quantity,
            );
            part.grain_locked = p.grain_locked;
            parts.push(part);
        }
        let layout = nesting::nest(&parts, sheet, kerf);
        let summary = format!(
            "{} sheet(s), {:.1}% yield{}.",
            layout.sheets_used,
            layout.yield_percent,
            if layout.unplaced.is_empty() {
                String::new()
            } else {
                format!("; {} part(s) larger than a sheet: {}", layout.unplaced.len(), layout.unplaced.join(", "))
            }
        );
        let mut value = serde_json::to_value(&layout).map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        if a.include_svg {
            value["svg"] = serde_json::Value::String(svg::render(&layout));
        }
        structured(summary, &value)
    }

    #[tool(
        description = "List the wood species and sheet goods the sag check knows, with stiffness (MOE) and creep factor.",
        annotations(title = "List species", read_only_hint = true, open_world_hint = false)
    )]
    pub fn list_species(&self) -> Result<CallToolResult, ErrorData> {
        structured(format!("{} species and sheet goods.", SPECIES.len()), &SPECIES)
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Joinery {
    fn get_info(&self) -> ServerConfig {
        mcp_kit::server_config(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            "Joinery — woodworking design",
            "Use design_furniture first: it returns parts, cut list, glue-up order and shelf sag checks in one call. \
             Use nest_parts to lay the cut list onto plywood sheets, and check_shelf_sag for one-off shelves. \
             All lengths use the `units` argument (mm by default).",
        )
    }
}
