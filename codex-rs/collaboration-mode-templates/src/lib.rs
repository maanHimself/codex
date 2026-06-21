use codex_protocol::prompt_overrides;

pub const PLAN: &str = include_str!("../templates/plan.md");
pub const DEFAULT: &str = include_str!("../templates/default.md");
pub const EXECUTE: &str = include_str!("../templates/execute.md");
pub const PAIR_PROGRAMMING: &str = include_str!("../templates/pair_programming.md");

pub fn plan() -> String {
    prompt_overrides::resolve_prompt(prompt_overrides::COLLABORATION_PLAN, PLAN)
}

pub fn default() -> String {
    prompt_overrides::resolve_prompt(prompt_overrides::COLLABORATION_DEFAULT, DEFAULT)
}

pub fn execute() -> String {
    prompt_overrides::resolve_prompt(prompt_overrides::COLLABORATION_EXECUTE, EXECUTE)
}

pub fn pair_programming() -> String {
    prompt_overrides::resolve_prompt(
        prompt_overrides::COLLABORATION_PAIR_PROGRAMMING,
        PAIR_PROGRAMMING,
    )
}
