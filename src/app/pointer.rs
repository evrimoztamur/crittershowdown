use super::CanvasSettings;

#[derive(Clone, Default)]
pub struct Pointer {
    previous: Option<Box<Pointer>>,
    pub real: (i32, i32),
    pub location: (i32, i32),
    pub button: bool,
    pub alt_button: bool,
}

impl Pointer {
    pub fn new(canvas_settings: &CanvasSettings) -> Pointer {
        let midpoint = (
            canvas_settings.canvas_width as i32 / 2,
            canvas_settings.canvas_height as i32 / 2,
        );

        Pointer {
            real: midpoint,
            location: Pointer::location_from_real(canvas_settings, midpoint),
            ..Default::default()
        }
    }

    pub fn clicked(&self) -> bool {
        match &self.previous {
            Some(pointer) => self.button && !pointer.button,
            None => self.button,
        }
    }

    pub fn alt_clicked(&self) -> bool {
        match &self.previous {
            Some(pointer) => self.alt_button && !pointer.alt_button,
            None => self.alt_button,
        }
    }

    pub fn swap(&mut self) {
        self.previous.take(); // Must explicitly drop old Pointer from heap
        self.previous = Some(Box::new(self.clone()));
    }

    pub fn teleport(&self, location: (i32, i32)) -> Pointer {
        let mut returned = self.clone();

        returned.location.0 += location.0;
        returned.location.1 += location.1;
        returned
    }

    pub fn location_from_real(canvas_settings: &CanvasSettings, real: (i32, i32)) -> (i32, i32) {
        let flip = (
            canvas_settings.interface_width as i32,
            canvas_settings.interface_height as i32,
        );

        let padding = canvas_settings.padding();

        if canvas_settings.orientation {
            (real.1 - padding.0, flip.0 - (real.0 - padding.1))
        } else {
            (real.0 - padding.0, real.1 - padding.1)
        }
    }

    pub fn in_region(&self, position: (i32, i32), size: (i32, i32)) -> bool {
        self.location.0 >= position.0
            && self.location.0 < position.0 + size.0
            && self.location.1 >= position.1
            && self.location.1 < position.1 + size.1
    }
}
