//! rnote-mod: colour pie menu.
//!
//! A radial menu that is drawn on top of the canvas area, centred on the pointer. It shows the nine
//! palette colours as softly rounded slices; what happens when one is picked is decided by the caller
//! (`RnAppWindow`). It is a plain `DrawingArea` (no new GObject type) that fills the overlay it is added
//! to, so it also catches the click that dismisses it.

use gtk4::glib::{self, clone};
use gtk4::{Align, DrawingArea, EventControllerMotion, GestureClick, cairo, gdk, prelude::*};
use std::cell::{Cell, RefCell};
use std::f64::consts::{FRAC_PI_2, PI};
use std::rc::Rc;

const SLICES: usize = 9;
const INNER_RADIUS: f64 = 40.0;
const OUTER_RADIUS: f64 = 120.0;
const HOVER_GROW: f64 = 10.0;
const EDGE_MARGIN: f64 = 12.0;
/// radius of the rounded corners of the slices
const CORNER_RADIUS: f64 = 8.0;

struct State {
    /// centre of the ring, in the coordinates of the drawing area
    center: Cell<(f64, f64)>,
    hovered: Cell<Option<usize>>,
    /// the slice of the colour that is currently selected, if any
    active: Cell<Option<usize>>,
    /// opacity of the slices, 0.1 - 1.0
    opacity: Cell<f64>,
    colors: RefCell<Vec<gdk::RGBA>>,
}

impl State {
    /// Slice under the given point. Slice 0 is centred at the top, the others follow clockwise.
    fn slice_at(&self, x: f64, y: f64) -> Option<usize> {
        let (cx, cy) = self.center.get();
        let (dx, dy) = (x - cx, y - cy);
        let radius = dx.hypot(dy);
        if !(INNER_RADIUS..=OUTER_RADIUS + HOVER_GROW).contains(&radius) {
            return None;
        }

        let step = 2.0 * PI / SLICES as f64;
        let angle = (dy.atan2(dx) + PI / 2.0 + step / 2.0).rem_euclid(2.0 * PI);
        Some(((angle / step) as usize) % SLICES)
    }
}

#[derive(Clone)]
pub(crate) struct PieMenu {
    area: DrawingArea,
    state: Rc<State>,
    on_pick: Rc<dyn Fn(usize)>,
}

impl PieMenu {
    /// `on_pick` is called with the slice index (0-8) when a colour is picked with the pointer.
    pub(crate) fn new(on_pick: impl Fn(usize) + 'static) -> Self {
        let state = Rc::new(State {
            center: Cell::new((0.0, 0.0)),
            hovered: Cell::new(None),
            active: Cell::new(None),
            opacity: Cell::new(0.5),
            colors: RefCell::new(Vec::new()),
        });
        let on_pick: Rc<dyn Fn(usize)> = Rc::new(on_pick);

        let area = DrawingArea::builder()
            .halign(Align::Fill)
            .valign(Align::Fill)
            .visible(false)
            .build();

        area.set_draw_func(clone!(
            #[strong]
            state,
            move |_, cr, _, _| draw(&state, cr)
        ));

        let motion_controller = EventControllerMotion::new();
        motion_controller.connect_motion(clone!(
            #[strong]
            state,
            #[weak]
            area,
            move |_, x, y| {
                let hovered = state.slice_at(x, y);
                if hovered != state.hovered.get() {
                    state.hovered.set(hovered);
                    area.queue_draw();
                }
            }
        ));
        area.add_controller(motion_controller);

        // Any press closes the menu. A press on a slice also picks it.
        let click_controller = GestureClick::builder().button(0).build();
        click_controller.connect_pressed(clone!(
            #[strong]
            state,
            #[strong]
            on_pick,
            #[weak]
            area,
            move |_, _, x, y| {
                area.set_visible(false);
                if let Some(index) = state.slice_at(x, y) {
                    on_pick(index);
                }
            }
        ));
        area.add_controller(click_controller);

        Self {
            area,
            state,
            on_pick,
        }
    }

    pub(crate) fn widget(&self) -> &DrawingArea {
        &self.area
    }

    pub(crate) fn is_open(&self) -> bool {
        self.area.is_visible()
    }

    /// Opens the menu centred on `pointer`, moved inward if it would stick out of `bounds`
    /// (the size of the overlay). `active` is the slice of the selected colour, `opacity` the opacity
    /// of the slices (0.1 - 1.0).
    pub(crate) fn open(
        &self,
        pointer: (f64, f64),
        bounds: (f64, f64),
        colors: Vec<gdk::RGBA>,
        active: Option<usize>,
        opacity: f64,
    ) {
        let reach = OUTER_RADIUS + HOVER_GROW + EDGE_MARGIN;
        let clamp = |value: f64, size: f64| {
            if size > 2.0 * reach {
                value.clamp(reach, size - reach)
            } else {
                size / 2.0
            }
        };
        // no pointer position seen yet: use the middle of the overlay
        let pointer = if pointer == (0.0, 0.0) {
            (bounds.0 / 2.0, bounds.1 / 2.0)
        } else {
            pointer
        };

        self.state
            .center
            .set((clamp(pointer.0, bounds.0), clamp(pointer.1, bounds.1)));
        self.state.hovered.set(None);
        self.state.active.set(active);
        self.state.opacity.set(opacity.clamp(0.1, 1.0));
        self.state.colors.replace(colors);
        self.area.set_visible(true);
        self.area.queue_draw();
    }

    pub(crate) fn close(&self) {
        self.area.set_visible(false);
    }

    /// Closes the menu and reports the slice as picked (used by the number keys).
    pub(crate) fn pick(&self, index: usize) {
        self.close();
        (self.on_pick)(index);
    }
}

/// Adds the outline of a ring slice with rounded corners to the path of `cr`. The slice spans the
/// angles `start..end` (radians) between the radii `inner` and `outer`.
fn slice_path(cr: &cairo::Context, center: (f64, f64), inner: f64, outer: f64, start: f64, end: f64) {
    let (cx, cy) = center;
    // never rounder than the ring is thick
    let radius = CORNER_RADIUS.min((outer - inner) / 2.0 - 0.5);
    // the four corners are circles that touch the slice's radial edge and its inner / outer arc
    let outer_centre = outer - radius;
    let inner_centre = inner + radius;
    let outer_offset = (radius / outer_centre).asin();
    let inner_offset = (radius / inner_centre).asin();

    let corner = |centre_radius: f64, centre_angle: f64, from: f64, to: f64| {
        cr.arc(
            cx + centre_radius * centre_angle.cos(),
            cy + centre_radius * centre_angle.sin(),
            radius,
            from,
            to,
        );
    };

    cr.new_path();
    // outer arc
    cr.arc(cx, cy, outer, start + outer_offset, end - outer_offset);
    // outer corner at the end edge
    corner(outer_centre, end - outer_offset, end - outer_offset, end + FRAC_PI_2);
    // inner corner at the end edge
    corner(inner_centre, end - inner_offset, end + FRAC_PI_2, end - inner_offset + PI);
    // inner arc, backwards
    cr.arc_negative(cx, cy, inner, end - inner_offset, start + inner_offset);
    // inner corner at the start edge
    corner(inner_centre, start + inner_offset, start + inner_offset + PI, start + 3.0 * FRAC_PI_2);
    // outer corner at the start edge
    corner(outer_centre, start + outer_offset, start - FRAC_PI_2, start + outer_offset);
    cr.close_path();
}

fn draw(state: &State, cr: &cairo::Context) {
    // faint scrim: shows that the menu is modal
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.12);
    let _ = cr.paint();

    let center = state.center.get();
    let step = 2.0 * PI / SLICES as f64;
    let colors = state.colors.borrow();
    let hovered = state.hovered.get();
    let active = state.active.get();
    let opacity = state.opacity.get();

    let outer_radius_of = |index: usize| {
        if hovered == Some(index) {
            OUTER_RADIUS + HOVER_GROW
        } else {
            OUTER_RADIUS
        }
    };
    let slice_angles = |index: usize| {
        let middle = -PI / 2.0 + index as f64 * step;
        (middle - step / 2.0, middle + step / 2.0)
    };

    // all slices first ...
    for (index, color) in colors.iter().enumerate().take(SLICES) {
        let (start, end) = slice_angles(index);
        slice_path(cr, center, INNER_RADIUS, outer_radius_of(index), start, end);

        cr.set_source_rgba(
            f64::from(color.red()),
            f64::from(color.green()),
            f64::from(color.blue()),
            f64::from(color.alpha()) * opacity,
        );
        let _ = cr.fill_preserve();

        cr.set_source_rgba(0.0, 0.0, 0.0, 0.35);
        cr.set_line_width(1.5);
        let _ = cr.stroke();
    }

    // ... then the outline of the selected one on top, so it overlaps both neighbours equally
    if let Some(index) = active.filter(|index| *index < colors.len().min(SLICES)) {
        let (start, end) = slice_angles(index);
        slice_path(cr, center, INNER_RADIUS, outer_radius_of(index), start, end);
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.8);
        cr.set_line_width(7.0);
        let _ = cr.stroke_preserve();
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.set_line_width(3.5);
        let _ = cr.stroke();
    }
}
