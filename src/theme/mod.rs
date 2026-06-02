pub mod icons;
pub mod colors;

use crate::theme::icons::IconManager;
use crate::theme::colors::ColorManager;

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
