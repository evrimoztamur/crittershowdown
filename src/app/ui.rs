use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use super::{ClipId, Pointer};
use crate::draw::{draw_image, draw_label, draw_text, draw_text_centered};

pub enum UIEvent {
    ButtonClick(usize, Option<ClipId>),
}

pub trait UIElement {
    fn boxed(self) -> Box<dyn UIElement>;

    fn tick(&mut self, _pointer: &Pointer) -> Option<UIEvent> {
        None
    }

    fn draw(
        &self,
        context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        pointer: &Pointer,
        frame: usize,
    ) -> Result<(), JsValue>;
}

#[derive(Clone)]
pub enum Alignment {
    Start(i32),
    Center,
    // End,
}

#[derive(Clone)]
pub enum ContentElement {
    Text(String, Alignment),
    Sprite((i32, i32), (i32, i32)),
    // List(Vec<ContentElement>),
    None,
}

impl UIElement for ContentElement {
    fn boxed(self) -> Box<dyn UIElement> {
        Box::new(self)
    }

    fn draw(
        &self,
        context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        _pointer: &Pointer,
        _frame: usize,
    ) -> Result<(), JsValue> {
        context.save();

        match self {
            ContentElement::Text(text, alignment) => match alignment {
                Alignment::Center => draw_text_centered(context, atlas, 0.0, 0.0, text),
                Alignment::Start(width) => {
                    draw_text(context, atlas, -width as f64 / 2.0 + 8.0, -4.0, text)
                }
            },
            ContentElement::Sprite(position, size) => draw_image(
                context,
                atlas,
                position.0 as f64,
                position.1 as f64,
                size.0 as f64,
                size.1 as f64,
                -size.0 as f64 / 2.0,
                -size.1 as f64 / 2.0,
            ),
            ContentElement::None => Ok(())
        }?;

        context.restore();

        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub enum LabelTrim {
    Round,
    Glorious,
    Return,
}

#[derive(Clone, PartialEq)]
pub enum LabelTheme {
    Default,
    Action,
    Bright,
    Disabled,
}

#[derive(Clone)]
pub struct ButtonElement {
    position: (i32, i32),
    size: (i32, i32),
    value: usize,
    trim: LabelTrim,
    class: LabelTheme,
    content: ContentElement,
    selected: bool,
}

impl ButtonElement {
    pub fn new(
        position: (i32, i32),
        size: (i32, i32),
        value: usize,
        trim: LabelTrim,
        class: LabelTheme,
        content: ContentElement,
    ) -> ButtonElement {
        ButtonElement {
            position,
            size,
            value,
            trim,
            class,
            content,
            selected: false,
        }
    }

    fn hovered(&self, pointer: &Pointer) -> bool {
        pointer.in_region(self.position, self.size)
    }

    fn clicked(&self, pointer: &Pointer) -> bool {
        self.hovered(pointer) && pointer.clicked() && self.class != LabelTheme::Disabled
    }

    fn clip_id(&self) -> Option<ClipId> {
        match self.trim {
            LabelTrim::Round => Some(ClipId::ClickForward),
            LabelTrim::Glorious => Some(ClipId::ClickForward),
            LabelTrim::Return => Some(ClipId::ClickBack),
        }
    }
}

impl UIElement for ButtonElement {
    fn boxed(self) -> Box<dyn UIElement> {
        Box::new(self)
    }

    fn draw(
        &self,
        context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        pointer: &Pointer,
        frame: usize,
    ) -> Result<(), JsValue> {
        let color = match self.class {
            LabelTheme::Default => {
                if self.selected {
                    &"#007faa"
                } else if self.hovered(pointer) {
                    &"#2a7faa"
                } else {
                    &"#008080"
                }
            }
            LabelTheme::Action => {
                if self.selected {
                    &"#007faa"
                } else if self.hovered(pointer) {
                    &"#7f1f00"
                } else {
                    &"#aa3f00"
                }
            }
            LabelTheme::Bright => {
                if self.selected {
                    &"#d43f00"
                } else if self.hovered(pointer) {
                    &"#007faa"
                } else {
                    &"#006080"
                }
            }
            LabelTheme::Disabled => &"#005247",
        };

        match self.class {
            LabelTheme::Disabled => {
                context.save();
                draw_label(
                    context,
                    atlas,
                    self.position,
                    self.size,
                    color,
                    &self.content,
                    pointer,
                    frame,
                    &self.trim,
                    true,
                )?;
                context.restore();
            }
            _ => draw_label(
                context,
                atlas,
                self.position,
                self.size,
                color,
                &self.content,
                pointer,
                frame,
                &self.trim,
                false,
            )?,
        }

        Ok(())
    }

    fn tick(&mut self, pointer: &Pointer) -> Option<UIEvent> {
        if self.clicked(pointer) {
            Some(UIEvent::ButtonClick(self.value, self.clip_id()))
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct ConfirmButtonElement {
    button: ButtonElement,
}

impl ConfirmButtonElement {
    pub fn new(
        position: (i32, i32),
        size: (i32, i32),
        value: usize,
        trim: LabelTrim,
        class: LabelTheme,
        content: ContentElement,
    ) -> ConfirmButtonElement {
        ConfirmButtonElement {
            button: ButtonElement::new(position, size, value, trim, class, content),
        }
    }
}

impl UIElement for ConfirmButtonElement {
    fn boxed(self) -> Box<dyn UIElement> {
        Box::new(self)
    }

    fn draw(
        &self,
        context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        pointer: &Pointer,
        frame: usize,
    ) -> Result<(), JsValue> {
        context.save();

        if self.button.selected {
            context.translate(
                ((frame as i64 / 4 - 1) % 4 - 2).abs() as f64 - 1.0,
                ((frame as i64 / 2 - 1) % 4 - 2).abs() as f64 - 1.0,
            )?;
        }

        self.button.draw(context, atlas, pointer, frame)?;

        context.restore();

        Ok(())
    }

    fn tick(&mut self, pointer: &Pointer) -> Option<UIEvent> {
        if pointer.clicked() {
            if self.button.clicked(pointer) {
                if self.button.selected {
                    Some(UIEvent::ButtonClick(
                        self.button.value,
                        self.button.clip_id(),
                    ))
                } else {
                    self.button.selected = true;
                    None
                }
            } else {
                self.button.selected = false;
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct ToggleButtonElement {
    button: ButtonElement,
}

impl ToggleButtonElement {
    pub fn new(
        position: (i32, i32),
        size: (i32, i32),
        value: usize,
        trim: LabelTrim,
        class: LabelTheme,
        content: ContentElement,
    ) -> ToggleButtonElement {
        ToggleButtonElement {
            button: ButtonElement::new(position, size, value, trim, class, content),
        }
    }

    pub fn selected(&self) -> bool {
        self.button.selected
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.button.selected = selected;
    }

    pub fn toggle(&mut self) {
        self.button.selected ^= true;
    }
}

impl UIElement for ToggleButtonElement {
    fn boxed(self) -> Box<dyn UIElement> {
        Box::new(self)
    }

    fn draw(
        &self,
        context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        pointer: &Pointer,
        frame: usize,
    ) -> Result<(), JsValue> {
        self.button.draw(context, atlas, pointer, frame)
    }

    fn tick(&mut self, pointer: &Pointer) -> Option<UIEvent> {
        if self.button.clicked(pointer) {
            self.toggle();

            Some(UIEvent::ButtonClick(
                self.button.value,
                self.button.clip_id(),
            ))
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct ButtonGroupElement {
    position: (i32, i32),
    buttons: Vec<ButtonElement>,
    value: usize,
}

impl ButtonGroupElement {
    pub fn new(
        position: (i32, i32),
        buttons: Vec<ButtonElement>,
        value: usize,
    ) -> ButtonGroupElement {
        ButtonGroupElement {
            position,
            buttons,
            value,
        }
    }
}

impl UIElement for ButtonGroupElement {
    fn boxed(self) -> Box<dyn UIElement> {
        Box::new(self)
    }

    fn tick(&mut self, pointer: &Pointer) -> Option<UIEvent> {
        let pointer = pointer.teleport((-self.position.0, -self.position.1));
        let mut event = None;

        for button in self.buttons.iter_mut() {
            if let Some(child_event) = button.tick(&pointer) {
                self.value = button.value;
                event = Some(child_event);
            }

            button.selected = self.value == button.value;
        }

        event
    }

    fn draw(
        &self,
        context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        pointer: &Pointer,
        frame: usize,
    ) -> Result<(), JsValue> {
        context.save();

        context.translate(self.position.0 as f64, self.position.1 as f64)?;

        let pointer = pointer.teleport((-self.position.0, -self.position.1));

        for button in &self.buttons {
            button.draw(context, atlas, &pointer, frame)?;
        }

        context.restore();

        Ok(())
    }
}

pub struct Interface {
    children: Vec<Box<dyn UIElement>>,
}

impl Interface {
    pub fn new(children: Vec<Box<dyn UIElement>>) -> Interface {
        Interface { children }
    }
}

impl UIElement for Interface {
    fn boxed(self) -> Box<dyn UIElement> {
        Box::new(self)
    }

    fn tick(&mut self, pointer: &Pointer) -> Option<UIEvent> {
        let mut event = None;

        for child in &mut self.children {
            if let Some(child_event) = child.tick(pointer) {
                event = Some(child_event);
            }
        }

        event
    }

    fn draw(
        &self,
        context: &CanvasRenderingContext2d,
        atlas: &HtmlCanvasElement,
        pointer: &Pointer,
        frame: usize,
    ) -> Result<(), JsValue> {
        for child in &self.children {
            child.draw(context, atlas, pointer, frame)?;
        }
        Ok(())
    }
}
