pub mod colors;
pub mod icons;

use crate::theme::colors::ColorManager;
use crate::theme::icons::IconManager;

pub struct Theme {
    pub icons: IconManager,
    pub colors: ColorManager,
}

impl Theme {
    pub fn new() -> Self {
        Self {
            icons: IconManager::new(),
            colors: ColorManager::new(),
        }
    }
}
