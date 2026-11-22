// Claude RPG 3.2: persistent server-authoritative quests.
// No client-provided counters; callers update progress only from game hooks.
// Daily quests use elapsed UTC seconds (24h since issued), not device-local midnight.

pub const DAILY_SECONDS: u64 = 86_400;
const MINE_GOAL: u32 = 24;
const KILL_GOAL: u32 = 10;
const FISH_GOAL: u32 = 3;
const CRAFT_GOAL: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activity { Mine, Kill, Fish, Craft }

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum StoryAction { Log, Workbench, WoodenPickaxe, Cobblestone, StonePickaxe, Ore, Monster, Iron }

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct StoryGoal {pub action:StoryAction,pub name:&'static str,pub amount:u32}
pub const STORY_GOALS:[StoryGoal;8]=[
    StoryGoal{action:StoryAction::Log,name:"Corte toras de árvores",amount:16},
    StoryGoal{action:StoryAction::Workbench,name:"Fabrique uma bancada de trabalho",amount:1},
    StoryGoal{action:StoryAction::WoodenPickaxe,name:"Fabrique uma picareta de madeira",amount:1},
    StoryGoal{action:StoryAction::Cobblestone,name:"Extraia pedra para calçamento",amount:32},
    StoryGoal{action:StoryAction::StonePickaxe,name:"Fabrique uma picareta de pedra",amount:1},
    StoryGoal{action:StoryAction::Ore,name:"Minere minérios naturais",amount:24},
    StoryGoal{action:StoryAction::Monster,name:"Derrote criaturas",amount:10},
    StoryGoal{action:StoryAction::Iron,name:"Recolha lingotes de ferro",amount:8},
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct QuestState {
    pub issued_at: u64,
    pub mined: u32,
    pub killed: u32,
    pub fished: u32,
    pub crafted: u32,
    pub rewarded_today: bool,
    pub failed_days: u32,
    pub completed_days: u32,
    // Normal multi-stage quest, independently persisted and finite.
    pub chapter: u8,
    pub chapter_mined: u32,
    pub chapter_killed: u32,
    pub story_stage:u8,
    pub story_progress:u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestEvent { DailyReward, DailyFailed, ChapterReward(u8), StoryReward(u8) }

impl QuestState {
    pub fn initialize(&mut self, now: u64) {
        if self.issued_at == 0 { self.issued_at = now.max(1); }
    }
    pub fn seconds_left(&self, now: u64) -> u64 {
        if self.issued_at == 0 { return DAILY_SECONDS; }
        DAILY_SECONDS.saturating_sub(now.saturating_sub(self.issued_at))
    }
    pub fn daily_complete(&self) -> bool {
        self.mined >= MINE_GOAL && self.killed >= KILL_GOAL &&
            self.fished >= FISH_GOAL && self.crafted >= CRAFT_GOAL
    }
    pub fn all_daily(&self) -> [(Activity, u32, u32); 4] {
        [
            (Activity::Mine, self.mined, MINE_GOAL),
            (Activity::Kill, self.killed, KILL_GOAL),
            (Activity::Fish, self.fished, FISH_GOAL),
            (Activity::Craft, self.crafted, CRAFT_GOAL),
        ]
    }
    /// Rolls at most one expiry per invocation: avoids huge skip-forward loops.
    /// Returning DailyFailed applies a real penalty in the profile adapter.
    pub fn refresh(&mut self, now: u64) -> Option<QuestEvent> {
        self.initialize(now);
        if self.seconds_left(now) != 0 { return None; }
        let failed = !self.rewarded_today;
        if failed { self.failed_days = self.failed_days.saturating_add(1); }
        self.issued_at = now.max(1);
        self.mined = 0;
        self.killed = 0;
        self.fished = 0;
        self.crafted = 0;
        self.rewarded_today = false;
        if failed { Some(QuestEvent::DailyFailed) } else { None }
    }
    /// Caller must refresh first; expired objectives never receive credit.
    pub fn record(&mut self, activity: Activity, amount: u32) -> Vec<QuestEvent> {
        let amount = amount.min(1024);
        match activity {
            Activity::Mine => {
                self.mined = self.mined.saturating_add(amount).min(MINE_GOAL);
                self.chapter_mined = self.chapter_mined.saturating_add(amount);
            }
            Activity::Kill => {
                self.killed = self.killed.saturating_add(amount).min(KILL_GOAL);
                self.chapter_killed = self.chapter_killed.saturating_add(amount);
            }
            Activity::Fish => self.fished = self.fished.saturating_add(amount).min(FISH_GOAL),
            Activity::Craft => self.crafted = self.crafted.saturating_add(amount).min(CRAFT_GOAL),
        }
        let mut events = Vec::new();
        if self.daily_complete() && !self.rewarded_today {
            self.rewarded_today = true;
            self.completed_days = self.completed_days.saturating_add(1);
            events.push(QuestEvent::DailyReward);
        }
        // Three escalating normal quests, each one paid just once.
        if self.chapter < 3 {
            let step = u32::from(self.chapter) + 1;
            if self.chapter_mined >= step * 32 && self.chapter_killed >= step * 8 {
                self.chapter = self.chapter.saturating_add(1);
                self.chapter_mined = 0;
                self.chapter_killed = 0;
                events.push(QuestEvent::ChapterReward(self.chapter));
            }
        }
        events
    }
    pub fn story_goal(&self)->Option<StoryGoal>{
        STORY_GOALS.get(self.story_stage as usize).copied()
    }
    /// A story step accepts only the named, independently observed event.
    /// The same event never completes two consecutive steps automatically.
    pub fn record_story(&mut self,action:StoryAction,amount:u32)->Option<QuestEvent>{
        let goal=self.story_goal()?;
        if goal.action != action || amount==0 {return None;}
        self.story_progress=self.story_progress.saturating_add(amount.min(1024)).min(goal.amount);
        if self.story_progress<goal.amount {return None;}
        self.story_stage=self.story_stage.saturating_add(1);
        self.story_progress=0;
        Some(QuestEvent::StoryReward(self.story_stage))
    }
    pub fn chapter_goals(&self) -> (u32, u32) {
        let step = u32::from(self.chapter.min(2)) + 1;
        (step * 32, step * 8)
    }
    /// Semi-colon separated ASCII scalars, no untrusted strings.
    pub fn encode(&self) -> String {
        format!("{};{};{};{};{};{};{};{};{};{};{};{};{};{}",
            self.issued_at, self.mined, self.killed, self.fished, self.crafted,
            u8::from(self.rewarded_today), self.failed_days, self.completed_days,
            self.chapter, self.chapter_mined, self.chapter_killed, 3,
            self.story_stage,self.story_progress)
    }
    pub fn decode(src: &str) -> Option<Self> {
        let f: Vec<&str> = src.split(';').collect();
        if !(f.len()==12 && f[11]=="2" || f.len()==14 && f[11]=="3") {return None;}
        let n = |idx: usize| f[idx].parse::<u64>().ok();
        let flag = n(5)?;
        if flag > 1 { return None; }
        Some(Self {
            issued_at: n(0)?,
            mined: n(1)?.min(MINE_GOAL as u64) as u32,
            killed: n(2)?.min(KILL_GOAL as u64) as u32,
            fished: n(3)?.min(FISH_GOAL as u64) as u32,
            crafted: n(4)?.min(CRAFT_GOAL as u64) as u32,
            rewarded_today: flag == 1,
            failed_days: n(6)?.min(u32::MAX as u64) as u32,
            completed_days: n(7)?.min(u32::MAX as u64) as u32,
            chapter: n(8)?.min(3) as u8,
            chapter_mined: n(9)?.min(u32::MAX as u64) as u32,
            chapter_killed: n(10)?.min(u32::MAX as u64) as u32,
            story_stage:if f.len()==14 {n(12)?.min(STORY_GOALS.len() as u64) as u8}
                else {0},
            story_progress:if f.len()==14 {n(13)?.min(1024) as u32} else {0},
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn daily_one_reward_only_and_rollover_failure() {
        let mut s = QuestState::default();
        s.initialize(10);
        assert_eq!(s.record(Activity::Mine, 24), vec![]);
        assert_eq!(s.record(Activity::Kill, 10), vec![]);
        assert_eq!(s.record(Activity::Fish, 3), vec![]);
        assert_eq!(s.record(Activity::Craft, 8), vec![QuestEvent::DailyReward]);
        assert_eq!(s.record(Activity::Craft, 8), vec![]);
        assert_eq!(s.refresh(10 + DAILY_SECONDS), None);
        assert_eq!(s.refresh(20 + DAILY_SECONDS * 2), Some(QuestEvent::DailyFailed));
    }
    #[test]
    fn chapter_rewards_once_and_records_persist() {
        let mut s = QuestState::default();
        s.record(Activity::Mine, 32);
        assert_eq!(s.record(Activity::Kill, 8), vec![QuestEvent::ChapterReward(1)]);
        assert_eq!(s.record(Activity::Kill, 5), vec![]);
        assert_eq!(QuestState::decode(&s.encode()), Some(s));
    }
    #[test]
    fn story_progress_cannot_skip_tasks_and_saves(){
        let mut q=QuestState::default();
        assert_eq!(q.record_story(StoryAction::Monster,100),None);
        assert_eq!(q.story_stage,0);
        assert_eq!(q.record_story(StoryAction::Log,15),None);
        assert_eq!(q.record_story(StoryAction::Log,1),Some(QuestEvent::StoryReward(1)));
        assert_eq!(q.story_stage,1);
        assert_eq!(QuestState::decode(&q.encode()),Some(q));
    }
    #[test]
    fn ignores_malformed_and_clamps_overflows() {
        assert!(QuestState::decode("not a quest").is_none());
        let mut s = QuestState::default();
        s.record(Activity::Mine, u32::MAX);
        assert_eq!(s.mined, 24);
    }
}
