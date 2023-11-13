mod app;
mod draw;
mod net;

use std::{
    cell::{Cell, RefCell},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use app::{App, AudioSystem, CanvasSettings};
use futures::Future;
use net::{fetch, request_session};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::future_to_promise;
use web_sys::{
    console, CanvasRenderingContext2d, Document, DomRect, FocusEvent, HtmlCanvasElement,
    HtmlImageElement, HtmlInputElement, KeyboardEvent, MouseEvent, Storage, TouchEvent, Window,
};

fn window() -> Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn storage() -> Option<Storage> {
    window().local_storage().unwrap_or_default()
}

#[cfg(feature = "deploy")]
pub const RESOURCE_BASE_URL: &str = ".";
#[cfg(not(feature = "deploy"))]
pub const RESOURCE_BASE_URL: &str = "";

fn init_canvas(
    canvas_settings: &CanvasSettings,
) -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), JsValue> {
    let canvas = document()
        .create_element("canvas")?
        .dyn_into::<HtmlCanvasElement>()?;

    canvas.set_width(canvas_settings.element_width());
    canvas.set_height(canvas_settings.element_height());

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    context.set_image_smoothing_enabled(false);

    Ok((canvas, context))
}

#[wasm_bindgen(start)]
async fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let container_element = document()
        .query_selector("#canvas-container")
        .unwrap()
        .unwrap();

    let device_pixel_ratio = window().device_pixel_ratio();

    let canvas_settings = CanvasSettings::new(
        384 + 16,
        256 + 16,
        256,
        256,
        2.0 * device_pixel_ratio,
        window().inner_width().unwrap().as_f64().unwrap()
            < window().inner_height().unwrap().as_f64().unwrap(),
    );

    // atlas_img.set_src(&format!("{RESOURCE_BASE_URL}/static/png/atlas.png?v=6"));

    let atlas_future = ImageFuture::new(&format!("{RESOURCE_BASE_URL}/static/png/atlas.png?v=6"));
    // let atlas_img = atlas_future.await.unwrap();
    let atlas_img: Rc<HtmlImageElement> = Rc::new(atlas_future.await.unwrap());

    let mut audio_system = AudioSystem::default();
    audio_system.populate_audio().await;

    {
        let atlas_img_a = atlas_img.clone();
        let atlas_img = atlas_img.clone();

        // let closure = Closure::<dyn FnMut(_) -> Result<(), JsValue>>::new(move |_: JsValue| {
        let (canvas, context) = init_canvas(&canvas_settings)?;
        let (interface_canvas, interface_context) = init_canvas(&canvas_settings)?;

        interface_canvas.set_id("interface-canvas");

        let text_input_element = document()
            .query_selector("#text-input")
            .unwrap()
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        let text_input_element = Rc::new(RefCell::new(text_input_element));

        container_element.append_child(&canvas)?;
        container_element.append_child(&interface_canvas)?;

        let (atlas, atlas_context) = init_canvas(&CanvasSettings {
            canvas_width: atlas_img.width(),
            canvas_height: atlas_img.height(),
            canvas_scale: 1.0,
            ..Default::default()
        })?;

        atlas_context.draw_image_with_html_image_element(&atlas_img, 0.0, 0.0)?;

        let app = App::new(&canvas_settings, audio_system.clone());

        let app = Rc::new(RefCell::new(app));

        let session_closure = {
            let app = app.clone();

            Closure::<dyn FnMut(JsValue)>::new(move |value| {
                let mut app = app.borrow_mut();
                app.on_session_response(value);
            })
        };

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        {
            let app = app.clone();
            let text_input = text_input_element.clone();

            {
                let app = app.borrow();

                if app.session_id().is_none() {
                    let _ = fetch(&request_session()).then(&session_closure);
                }
            }

            *g.borrow_mut() = Some(Closure::new(move || {
                let mut app = app.borrow_mut();
                let text_input = text_input.borrow_mut();

                {
                    app.tick(&text_input);
                    app.draw(&context, &interface_context, &atlas).unwrap();
                }

                request_animation_frame(f.borrow().as_ref().unwrap());
            }));

            request_animation_frame(g.borrow().as_ref().unwrap());
        }

        session_closure.forget();

        let canvas = Rc::new(canvas);
        let bound: Rc<RefCell<Option<DomRect>>> =
            Rc::new(RefCell::new(Some(canvas.get_bounding_client_rect())));

        {
            let canvas = canvas.clone();
            let bound = bound.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |_: JsValue| {
                bound.replace(Some(canvas.get_bounding_client_rect()));
            });
            window()
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let app = app.clone();
            let text_input = text_input_element.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: FocusEvent| {
                let mut app = app.borrow_mut();
                let text_input = text_input.borrow();
                app.on_blur(event, text_input.as_ref());
            });

            document()
                .add_event_listener_with_callback("focusout", closure.as_ref().unchecked_ref())?;

            closure.forget();
        }

        {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
                let mut app = app.borrow_mut();
                app.on_mouse_down(event);
            });
            document()
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
                let mut app = app.borrow_mut();
                app.on_mouse_up(event);
            });
            document()
                .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let app = app.clone();
            let bound = bound.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
                let mut app = app.borrow_mut();
                if let Some(bound) = bound.borrow().as_deref() {
                    app.on_mouse_move(bound, event);
                }
            });
            document()
                .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let app = app.clone();
            let bound = bound.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: TouchEvent| {
                let mut app = app.borrow_mut();
                if let Some(bound) = bound.borrow().as_deref() {
                    app.on_touch_move(bound, event);
                }
            });
            document()
                .add_event_listener_with_callback("touchmove", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let bound = bound.clone();
            let app = app.clone();

            let closure = Closure::<dyn FnMut(_)>::new(move |event: TouchEvent| {
                if let Some(bound) = bound.borrow().as_deref() {
                    let mut app = app.borrow_mut();
                    app.on_touch_start(bound, event);
                }
            });
            document()
                .add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: TouchEvent| {
                if let Some(bound) = bound.borrow().as_deref() {
                    let mut app = app.borrow_mut();
                    app.on_touch_end(bound, event);
                }
            });
            document()
                .add_event_listener_with_callback("touchend", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let app = app.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: KeyboardEvent| {
                let mut app = app.borrow_mut();
                app.on_key_down(event);
            });
            document()
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }

        {
            let closure = Closure::<dyn FnMut(_)>::new(move |event: MouseEvent| {
                event.prevent_default();
            });
            document().add_event_listener_with_callback(
                "contextmenu",
                closure.as_ref().unchecked_ref(),
            )?;
            closure.forget();
        }

        // Ok(())
        // });

        // console::log_1(&"!".into());
        // atlas_img_a.add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())?;
        // closure.forget();
    }

    Ok(())
}

pub struct ImageFuture {
    image: Option<HtmlImageElement>,
    load_failed: Rc<Cell<bool>>,
}

impl ImageFuture {
    pub fn new(path: &str) -> Self {
        let image = HtmlImageElement::new().unwrap();
        image.set_src(path);
        ImageFuture {
            image: Some(image),
            load_failed: Rc::new(Cell::new(false)),
        }
    }
}

impl Future for ImageFuture {
    type Output = Result<HtmlImageElement, ()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &self.image {
            Some(image) if image.complete() => {
                let image = self.image.take().unwrap();
                let failed = self.load_failed.get();

                if failed {
                    Poll::Ready(Err(()))
                } else {
                    Poll::Ready(Ok(image))
                }
            }
            Some(image) => {
                let waker = cx.waker().clone();
                let on_load_closure = Closure::wrap(Box::new(move || {
                    waker.wake_by_ref();
                }) as Box<dyn FnMut()>);
                image.set_onload(Some(on_load_closure.as_ref().unchecked_ref()));
                on_load_closure.forget();

                let waker = cx.waker().clone();
                let failed_flag = self.load_failed.clone();
                let on_error_closure = Closure::wrap(Box::new(move || {
                    failed_flag.set(true);
                    waker.wake_by_ref();
                }) as Box<dyn FnMut()>);
                image.set_onerror(Some(on_error_closure.as_ref().unchecked_ref()));
                on_error_closure.forget();

                Poll::Pending
            }
            _ => Poll::Ready(Err(())),
        }
    }
}

#[macro_export]
macro_rules! tuple_as {
    ($t: expr, $ty: ident) => {{
        let (a, b) = $t;
        (a as $ty, b as $ty)
    }};
}
