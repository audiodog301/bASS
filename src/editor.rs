use nih_plug::prelude::Editor;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};

use nih_plug_vizia::vizia::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg::ImageId;
use nih_plug_vizia::vizia::vg::RenderTarget;
use nih_plug_vizia::vizia::vg::{Paint, Path};
use nih_plug_vizia::vizia::vg::ImageFlags;

use std::cell::RefCell;
use std::sync::Arc;
use std::rc::Rc;

use crate::BassParams;

struct BackgroundImage {
    image: Rc<RefCell<Option<ImageId>>>,
}

impl BackgroundImage {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {
            image: Rc::new(RefCell::new(None)),
        }
        .build(cx, |_| {})
    }
}

impl View for BackgroundImage {
    fn draw(&self, cx: &mut DrawContext<'_>, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        let image_id = if let Some(image_id) = *self.image.borrow() {
            image_id
        } else {
            canvas.load_image_mem(include_bytes!("gradient.png"), ImageFlags::empty()).expect("error loading background image")
        };

        *self.image.borrow_mut() = Some(image_id);
        canvas.set_render_target(RenderTarget::Image(image_id));

        let mut path = Path::new();
        path.rect(bounds.x, bounds.y, bounds.w, bounds.h);

        canvas.fill_path(
            &mut path,
            &Paint::image(image_id, bounds.x, bounds.y, bounds.w, bounds.h, 0.0, 1.0),
        );
    }
}

#[derive(Lens)]
struct Data {
    params: Arc<BassParams>,
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::from_size(200, 225)
}

pub(crate) fn create(
    params: Arc<BassParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone(),
        }
        .build(cx);

        ResizeHandle::new(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "bASS!~")
                .font(assets::NOTO_SANS_THIN)
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));

            // NOTE: VIZIA adds 1 pixel of additional height to these labels, so we'll need to
            //       compensate for that
            Label::new(cx, "skrunkle").bottom(Pixels(-1.0));
            ParamSlider::new(cx, Data::params, |params| &params.skrunkle);

            Label::new(cx, "gate?????").bottom(Pixels(0.0));
            ParamSlider::new(cx, Data::params, |params| &params.threshold);

            Label::new(cx, "output gain").bottom(Pixels(0.0));
            ParamSlider::new(cx, Data::params, |params| &params.output_gain);

            BackgroundImage::new(cx);
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));
    })
}
