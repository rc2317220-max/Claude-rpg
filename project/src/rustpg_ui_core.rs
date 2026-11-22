// Presentation Engine core: screen identity, palette and bounded navigation state.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Overview,
    Status,
    Trees,
    Bestiary,
    Professions,
    Equipment,
    Inventory,
    Enchantments,
    History,
}

impl Screen {
    pub const ALL: [Self; 9] = [
        Self::Overview, Self::Status, Self::Trees, Self::Bestiary,
        Self::Professions, Self::Equipment, Self::Inventory,
        Self::Enchantments, Self::History,
    ];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Resumo",
            Self::Status => "Status",
            Self::Trees => "Habilidades",
            Self::Bestiary => "Bestiário",
            Self::Professions => "Profissões",
            Self::Equipment => "Equipamento",
            Self::Inventory => "Relíquias",
            Self::Enchantments => "Encantamentos",
            Self::History => "Histórico",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub background: u32,
    pub panel: u32,
    pub panel_alt: u32,
    pub border: u32,
    pub text: u32,
    pub muted: u32,
    pub accent: u32,
    pub danger: u32,
    pub stamina: u32,
    pub vitality: u32,
}

pub const DARK_FANTASY: Palette = Palette {
    background: 0xE00B0E16,
    panel: 0xEE141A25,
    panel_alt: 0xEE202838,
    border: 0xFF4C5B78,
    text: 0xFFF1F3F7,
    muted: 0xFF9AA7BC,
    accent: 0xFFC49BFF,
    danger: 0xFFE34B5F,
    stamina: 0xFFB56CFF,
    vitality: 0xFFFF9F43,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct Navigation {
    pub screen: usize,
    pub scroll: i32,
    pub selected: usize,
    pub drag_origin: Option<[f32; 2]>,
}

impl Navigation {
    pub fn screen(&self) -> Screen {
        Screen::ALL[self.screen.min(Screen::ALL.len() - 1)]
    }
    pub fn next(&mut self) {
        self.screen = (self.screen + 1) % Screen::ALL.len();
        self.scroll = 0;
        self.selected = 0;
    }
    pub fn previous(&mut self) {
        self.screen = (self.screen + Screen::ALL.len() - 1) % Screen::ALL.len();
        self.scroll = 0;
        self.selected = 0;
    }
    pub fn select(&mut self, index: usize) {
        self.selected = index;
    }
    pub fn scroll_by(&mut self, amount: i32) {
        self.scroll = self.scroll.saturating_add(amount).max(0);
    }
}
