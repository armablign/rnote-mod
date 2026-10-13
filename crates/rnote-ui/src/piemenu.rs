//! rnote-mod: colour pie menu.
//!
//! A radial menu that is drawn on top of the canvas area, centred on the pointer. It shows the nine
//! palette colours; what happens when one is picked is decided by the caller (`RnAppWindow`).
//! It is a plain `DrawingArea` (no new GObject type) that fills the overlay it is added to, so it also
//! catches the click that dismisses it.

use gtk4::glib::{self, clone};
use gtk4::{Align, DrawingArea, EventControllerMotion, GestureClick, cairo, gdk, prelude::*};
use std::cell::{Cell, RefCell};
use std::f64::consts::PI;
use std::rc::Rc;

const SLICES: usize = 9;
const INNER_RADIUS: f64 = 40.0;
const OUTER_RADIUS: f64 = 120.0;
const HOVER_GROW: f64 = 10.0;
const EDGE_MARGIN: f64 = 12.0;

struct State {
    /// centre of the ring, in the coordinates of the drawing area
    center: Cell<(f64, f64)>,
    hovered: Cell<Option<usize>>,
    /// the slice of the colour that is currently selected, if any
    active: Cell<Option<usize>>,
    /// the colour of the selected pad, drawn in the middle of the ring
    current: RefCell<gdk::RGBA>,
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
            current: RefCell::new(gdk::RGBA::new(0.0, 0.0, 0.0, 1.0)),
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
    /// (the size of the overlay). `active` is the slice of the selected colour, `current` the colour
    /// that is drawn in the middle.
    pub(crate) fn open(
        &self,
        pointer: (f64, f64),
        bounds: (f64, f64),
        colors: Vec<gdk::RGBA>,
        active: Option<usize>,
        current: gdk::RGBA,
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
        self.state.current.replace(current);
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

fn draw(state: &State, cr: &cairo::Context) {
    // faint scrim: shows that the menu is modal
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.12);
    let _ = cr.paint();

    let (cx, cy) = state.center.get();
    let step = 2.0 * PI / SLICES as f64;
    let colors = state.colors.borrow();
    let hovered = state.hovered.get();
    let active = state.active.get();

    cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    cr.set_font_size(16.0);

    for (index, color) in colors.iter().enumerate().take(SLICES) {
        let middle = -PI / 2.0 + index as f64 * step;
        let outer = if hovered == Some(index) {
            OUTER_RADIUS + HOVER_GROW
        } else {
            OUTER_RADIUS
        };

        cr.new_path();
        cr.arc(cx, cy, outer, middle - step / 2.0, middle + step / 2.0);
        cr.arc_negative(cx, cy, INNER_RADIUS, middle + step / 2.0, middle - step / 2.0);
        cr.close_path();

        let (red, green, blue) = (
            f64::from(color.red()),
            f64::from(color.green()),
            f64::from(color.blue()),
        );
        cr.set_source_rgb(red, green, blue);
        let _ = cr.fill_preserve();

        if active == Some(index) {
            cr.set_source_rgb(0.0, 0.0, 0.0);
            cr.set_line_width(7.0);
            let _ = cr.stroke_preserve();
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.set_line_width(3.5);
            let _ = cr.stroke();
        } else {
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.6);
            cr.set_line_width(1.5);
            let _ = cr.stroke();
        }

        // number label: also the key that picks this colour while the menu is open
        let label = (index + 1).to_string();
        let label_radius = (INNER_RADIUS + outer) / 2.0;
        let (label_x, label_y) = (
            cx + label_radius * middle.cos(),
            cy + label_radius * middle.sin(),
        );
        if 0.299 * red + 0.587 * green + 0.114 * blue > 0.55 {
            cr.set_source_rgb(0.0, 0.0, 0.0);
        } else {
            cr.set_source_rgb(1.0, 1.0, 1.0);
        }
        if let Ok(extents) = cr.text_extents(&label) {
            cr.move_to(
                label_x - extents.width() / 2.0 - extents.x_bearing(),
                label_y - extents.height() / 2.0 - extents.y_bearing(),
            );
            let _ = cr.show_text(&label);
        }
    }

    // middle of the ring: the colour that is currently selected
    let current = state.current.borrow();
    cr.new_path();
    cr.arc(cx, cy, INNER_RADIUS - 8.0, 0.0, 2.0 * PI);
    cr.set_source_rgba(
        f64::from(current.red()),
        f64::from(current.green()),
        f64::from(current.blue()),
        f64::from(current.alpha()),
    );
    let _ = cr.fill_preserve();
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.7);
    cr.set_line_width(2.0);
    let _ = cr.stroke();
}
