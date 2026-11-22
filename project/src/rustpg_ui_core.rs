// Presentation Engine core: screen identity, palette and bounded navigation state.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Overview,
    Character,
    Status,
    Attributes,
    Trees,
    Professions,
    Equipment,
    Accessories,
    Enchantments,
    Skills,
    Bestiary,
    EvilAtt,
    DailyQuests,
    Quests,
    Achievements,
    History,
}

impl Screen {
    pub const ALL: [Self; 16] = [
        Self::Overview,
        Self::Character,
        Self::Status,
        Self::Attributes,
        Self::Trees,
        Self::Professions,
        Self::Equipment,
        Self::Accessories,
        Self::Enchantments,
        Self::Skills,
        Self::Bestiary,
        Self::EvilAtt,
        Self::DailyQuests,
        Self::Quests,
        Self::Achievements,
        Self::History,
    ];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Visão geral",
            Self::Character => "Personagem",
            Self::Status => "Status",
            Self::Attributes => "Atributos",
            Self::Trees => "Skill Trees",
            Self::Professions => "Profissões",
            Self::Equipment => "Equipamentos",
            Self::Accessories => "Acessórios",
            Self::Enchantments => "Encantamentos",
            Self::Skills => "Habilidades",
            Self::Bestiary => "Bestiário",
            Self::EvilAtt => "Evil Att.",
            Self::DailyQuests => "Quests diárias",
            Self::Quests => "Quests",
            Self::Achievements => "Conquistas",
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
