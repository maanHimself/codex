use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::RwLock;

/// Process-wide prompt override provider installed by embedding runtimes.
///
/// Codex owns the built-in prompt fallbacks. Embedders can install a provider
/// that returns a non-empty replacement for a stable prompt key. Missing or
/// empty values always preserve the built-in Codex prompt.
pub trait PromptOverrideProvider: Send + Sync {
    fn prompt_override(&self, key: &str) -> Option<String>;
}

static PROMPT_OVERRIDE_PROVIDER: RwLock<Option<Arc<dyn PromptOverrideProvider>>> =
    RwLock::new(None);

pub const BASE_INSTRUCTIONS_DEFAULT: &str = "base-instructions-default";
pub const MODELS_MANAGER_BASE_INSTRUCTIONS: &str = "models-manager-base-instructions";
pub const MODELS_MANAGER_PERSONALITY_HEADER: &str = "models-manager-personality-header";
pub const MODELS_MANAGER_PERSONALITY_FRIENDLY: &str = "models-manager-personality-friendly";
pub const MODELS_MANAGER_PERSONALITY_PRAGMATIC: &str = "models-manager-personality-pragmatic";
pub const MODEL_BASE_INSTRUCTIONS: &str = "model-base-instructions";
pub const MODEL_INSTRUCTIONS_TEMPLATE: &str = "model-instructions-template";
pub const MODEL_BASE_INSTRUCTIONS_PREFIX: &str = "model-base-instructions:";
pub const MODEL_INSTRUCTIONS_TEMPLATE_PREFIX: &str = "model-instructions-template:";

pub const REVIEW_SYSTEM_PROMPT: &str = "review-system-prompt";
pub const REVIEW_EXIT_SUCCESS_TEMPLATE: &str = "review-exit-success-template";
pub const REVIEW_EXIT_INTERRUPTED_TEMPLATE: &str = "review-exit-interrupted-template";
pub const REVIEW_UNCOMMITTED_PROMPT: &str = "review-uncommitted-prompt";
pub const REVIEW_BASE_BRANCH_BACKUP_PROMPT: &str = "review-base-branch-backup-prompt";
pub const REVIEW_BASE_BRANCH_PROMPT: &str = "review-base-branch-prompt";
pub const REVIEW_COMMIT_WITH_TITLE_PROMPT: &str = "review-commit-with-title-prompt";
pub const REVIEW_COMMIT_PROMPT: &str = "review-commit-prompt";

pub const COMPACT_SUMMARIZATION_PROMPT: &str = "compact-summarization-prompt";
pub const COMPACT_SUMMARY_PREFIX: &str = "compact-summary-prefix";

pub const GUARDIAN_POLICY: &str = "guardian-policy";
pub const GUARDIAN_POLICY_TEMPLATE: &str = "guardian-policy-template";

pub const REALTIME_BACKEND_PROMPT: &str = "realtime-backend-prompt";
pub const REALTIME_START_INSTRUCTIONS: &str = "realtime-start-instructions";
pub const REALTIME_END_INSTRUCTIONS: &str = "realtime-end-instructions";

pub const PERMISSIONS_APPROVAL_NEVER: &str = "permissions-approval-never";
pub const PERMISSIONS_APPROVAL_UNLESS_TRUSTED: &str = "permissions-approval-unless-trusted";
pub const PERMISSIONS_APPROVAL_ON_FAILURE: &str = "permissions-approval-on-failure";
pub const PERMISSIONS_APPROVAL_ON_REQUEST: &str = "permissions-approval-on-request";
pub const PERMISSIONS_APPROVAL_ON_REQUEST_PERMISSION: &str =
    "permissions-approval-on-request-permission";
pub const PERMISSIONS_AUTO_REVIEW_SUFFIX: &str = "permissions-auto-review-suffix";
pub const PERMISSIONS_SANDBOX_DANGER_FULL_ACCESS: &str = "permissions-sandbox-danger-full-access";
pub const PERMISSIONS_SANDBOX_WORKSPACE_WRITE: &str = "permissions-sandbox-workspace-write";
pub const PERMISSIONS_SANDBOX_READ_ONLY: &str = "permissions-sandbox-read-only";
pub const PERMISSIONS_GRANULAR_INTRO: &str = "permissions-granular-intro";
pub const PERMISSIONS_REQUEST_TOOL_SECTION: &str = "permissions-request-tool-section";
pub const CONSEQUENTIAL_TOOL_MESSAGE_TEMPLATES: &str = "consequential-tool-message-templates";

pub const AGENTS_MD_HIERARCHICAL_MESSAGE: &str = "agents-md-hierarchical-message";

pub const GOAL_CONTINUATION_PROMPT: &str = "goal-continuation-prompt";
pub const GOAL_BUDGET_LIMIT_PROMPT: &str = "goal-budget-limit-prompt";
pub const GOAL_OBJECTIVE_UPDATED_PROMPT: &str = "goal-objective-updated-prompt";

pub const AGENT_ROLE_EXPLORER: &str = "agent-role-explorer";
pub const AGENT_ROLE_AWAITER: &str = "agent-role-awaiter";

pub const COLLABORATION_PLAN: &str = "collaboration-plan";
pub const COLLABORATION_DEFAULT: &str = "collaboration-default";
pub const COLLABORATION_EXECUTE: &str = "collaboration-execute";
pub const COLLABORATION_PAIR_PROGRAMMING: &str = "collaboration-pair-programming";

pub const MEMORY_READ_PATH: &str = "memory-read-path";
pub const MEMORY_CONSOLIDATION: &str = "memory-consolidation";
pub const MEMORY_STAGE_ONE_INPUT: &str = "memory-stage-one-input";
pub const MEMORY_STAGE_ONE_SYSTEM: &str = "memory-stage-one-system";
pub const MEMORY_EXTENSION_AD_HOC_INSTRUCTIONS: &str = "memory-extension-ad-hoc-instructions";
pub const MEMORY_EXTENSIONS_FOLDER_STRUCTURE: &str = "memory-extensions-folder-structure";
pub const MEMORY_EXTENSIONS_PRIMARY_INPUTS: &str = "memory-extensions-primary-inputs";

pub const IMAGE_GENERATION_TOOL_DESCRIPTION: &str = "image-generation-tool-description";
pub const WEB_RUN_TOOL_DESCRIPTION: &str = "web-run-tool-description";
pub const TOOL_SEARCH_DESCRIPTION: &str = "tool-search-description";
pub const LIST_AVAILABLE_PLUGINS_DESCRIPTION: &str = "list-available-plugins-description";
pub const REQUEST_PLUGIN_INSTALL_DESCRIPTION: &str = "request-plugin-install-description";
pub const SKILL_MCP_DEPENDENCY_INSTALL_QUESTION: &str = "skill-mcp-dependency-install-question";
pub const SKILL_MCP_DEPENDENCY_INSTALL_DESCRIPTION: &str =
    "skill-mcp-dependency-install-description";
pub const SKILL_MCP_DEPENDENCY_SKIP_DESCRIPTION: &str = "skill-mcp-dependency-skip-description";

pub const SKILL_DESCRIPTION_TRUNCATED_WARNING: &str = "skill-description-truncated-warning";
pub const SKILL_DESCRIPTION_TRUNCATED_WARNING_WITH_PERCENT: &str =
    "skill-description-truncated-warning-with-percent";
pub const SKILL_DESCRIPTIONS_REMOVED_WARNING_PREFIX: &str =
    "skill-descriptions-removed-warning-prefix";
pub const SKILLS_INTRO_WITH_ABSOLUTE_PATHS: &str = "skills-intro-with-absolute-paths";
pub const SKILLS_INTRO_WITH_ALIASES: &str = "skills-intro-with-aliases";
pub const SKILLS_HOW_TO_USE_WITH_ABSOLUTE_PATHS: &str = "skills-how-to-use-with-absolute-paths";
pub const SKILLS_HOW_TO_USE_WITH_ALIASES: &str = "skills-how-to-use-with-aliases";
pub const SKILL_DESCRIPTION_PREFIX: &str = "skill-description:";
pub const SKILL_SHORT_DESCRIPTION_PREFIX: &str = "skill-short-description:";
pub const SKILL_INTERFACE_SHORT_DESCRIPTION_PREFIX: &str = "skill-interface-short-description:";
pub const SKILL_DEFAULT_PROMPT_PREFIX: &str = "skill-default-prompt:";
pub const SKILL_DEPENDENCY_DESCRIPTION_PREFIX: &str = "skill-dependency-description:";
pub const SKILL_BODY_PREFIX: &str = "skill-body:";

pub const APPLY_PATCH_TOOL_DESCRIPTION: &str = "apply-patch-tool-description";
pub const EXEC_COMMAND_TOOL_DESCRIPTION: &str = "exec-command-tool-description";
pub const WRITE_STDIN_TOOL_DESCRIPTION: &str = "write-stdin-tool-description";
pub const SHELL_COMMAND_TOOL_DESCRIPTION: &str = "shell-command-tool-description";
pub const REQUEST_PERMISSIONS_TOOL_DESCRIPTION: &str = "request-permissions-tool-description";
pub const REQUEST_USER_INPUT_TOOL_DESCRIPTION: &str = "request-user-input-tool-description";
pub const PLAN_TOOL_DESCRIPTION: &str = "plan-tool-description";
pub const GET_GOAL_TOOL_DESCRIPTION: &str = "get-goal-tool-description";
pub const CREATE_GOAL_TOOL_DESCRIPTION: &str = "create-goal-tool-description";
pub const UPDATE_GOAL_TOOL_DESCRIPTION: &str = "update-goal-tool-description";
pub const MCP_LIST_RESOURCES_TOOL_DESCRIPTION: &str = "mcp-list-resources-tool-description";
pub const MCP_LIST_RESOURCE_TEMPLATES_TOOL_DESCRIPTION: &str =
    "mcp-list-resource-templates-tool-description";
pub const MCP_READ_RESOURCE_TOOL_DESCRIPTION: &str = "mcp-read-resource-tool-description";
pub const VIEW_IMAGE_TOOL_DESCRIPTION: &str = "view-image-tool-description";
pub const SPAWN_AGENTS_ON_CSV_TOOL_DESCRIPTION: &str = "spawn-agents-on-csv-tool-description";
pub const REPORT_AGENT_JOB_RESULT_TOOL_DESCRIPTION: &str =
    "report-agent-job-result-tool-description";
pub const CODE_MODE_EXEC_TOOL_DESCRIPTION: &str = "code-mode-exec-tool-description";
pub const CODE_MODE_WAIT_TOOL_DESCRIPTION: &str = "code-mode-wait-tool-description";
pub const MULTI_AGENT_V1_NAMESPACE_DESCRIPTION: &str = "multi-agent-v1-namespace-description";
pub const SPAWN_AGENT_TOOL_DESCRIPTION: &str = "spawn-agent-tool-description";
pub const SPAWN_AGENT_TOOL_DESCRIPTION_V2: &str = "spawn-agent-tool-description-v2";
pub const SEND_INPUT_TOOL_DESCRIPTION: &str = "send-input-tool-description";
pub const SEND_MESSAGE_TOOL_DESCRIPTION: &str = "send-message-tool-description";
pub const ASSIGN_TASK_TOOL_DESCRIPTION: &str = "assign-task-tool-description";
pub const RESUME_AGENT_TOOL_DESCRIPTION: &str = "resume-agent-tool-description";
pub const WAIT_AGENT_TOOL_DESCRIPTION: &str = "wait-agent-tool-description";
pub const WAIT_AGENT_TOOL_DESCRIPTION_V2: &str = "wait-agent-tool-description-v2";
pub const LIST_AGENTS_TOOL_DESCRIPTION: &str = "list-agents-tool-description";
pub const CLOSE_AGENT_TOOL_DESCRIPTION: &str = "close-agent-tool-description";
pub const EXEC_POLICY_PROMPT_CONFLICT_REASON: &str = "exec-policy-prompt-conflict-reason";
pub const UNIX_ESCALATION_PROMPT_CONFLICT_REASON: &str = "unix-escalation-prompt-conflict-reason";
pub const INIT_COMMAND_PROMPT: &str = "init-command-prompt";
pub const SIDE_CONVERSATION_BOUNDARY_PROMPT: &str = "side-conversation-boundary-prompt";
pub const SIDE_CONVERSATION_DEVELOPER_INSTRUCTIONS: &str =
    "side-conversation-developer-instructions";
pub const IDE_CONTEXT_REQUEST_MARKER: &str = "ide-context-request-marker";
pub const DYNAMIC_TOOLS_SOURCE_DESCRIPTION: &str = "dynamic-tools-source-description";
pub const MULTI_AGENT_TOOL_SEARCH_SOURCE_DESCRIPTION: &str =
    "multi-agent-tool-search-source-description";
pub const TEST_SYNC_TOOL_DESCRIPTION: &str = "test-sync-tool-description";

pub fn set_prompt_override_provider(provider: Arc<dyn PromptOverrideProvider>) {
    if let Ok(mut guard) = PROMPT_OVERRIDE_PROVIDER.write() {
        *guard = Some(provider);
    }
}

pub fn clear_prompt_override_provider() {
    if let Ok(mut guard) = PROMPT_OVERRIDE_PROVIDER.write() {
        *guard = None;
    }
}

pub fn prompt_override(key: &str) -> Option<String> {
    let provider = PROMPT_OVERRIDE_PROVIDER
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().map(Arc::clone))?;
    provider
        .prompt_override(key)
        .filter(|value| !value.is_empty())
}

pub fn resolve_prompt(key: &str, built_in: &str) -> String {
    prompt_override(key).unwrap_or_else(|| built_in.to_string())
}

pub fn resolve_prompt_str<'a>(key: &str, built_in: &'a str) -> std::borrow::Cow<'a, str> {
    match prompt_override(key) {
        Some(override_prompt) => std::borrow::Cow::Owned(override_prompt),
        None => std::borrow::Cow::Borrowed(built_in),
    }
}

pub fn all_prompt_override_keys() -> BTreeSet<&'static str> {
    [
        BASE_INSTRUCTIONS_DEFAULT,
        MODELS_MANAGER_BASE_INSTRUCTIONS,
        MODELS_MANAGER_PERSONALITY_HEADER,
        MODELS_MANAGER_PERSONALITY_FRIENDLY,
        MODELS_MANAGER_PERSONALITY_PRAGMATIC,
        MODEL_BASE_INSTRUCTIONS,
        MODEL_INSTRUCTIONS_TEMPLATE,
        REVIEW_SYSTEM_PROMPT,
        REVIEW_EXIT_SUCCESS_TEMPLATE,
        REVIEW_EXIT_INTERRUPTED_TEMPLATE,
        REVIEW_UNCOMMITTED_PROMPT,
        REVIEW_BASE_BRANCH_BACKUP_PROMPT,
        REVIEW_BASE_BRANCH_PROMPT,
        REVIEW_COMMIT_WITH_TITLE_PROMPT,
        REVIEW_COMMIT_PROMPT,
        COMPACT_SUMMARIZATION_PROMPT,
        COMPACT_SUMMARY_PREFIX,
        GUARDIAN_POLICY,
        GUARDIAN_POLICY_TEMPLATE,
        REALTIME_BACKEND_PROMPT,
        REALTIME_START_INSTRUCTIONS,
        REALTIME_END_INSTRUCTIONS,
        PERMISSIONS_APPROVAL_NEVER,
        PERMISSIONS_APPROVAL_UNLESS_TRUSTED,
        PERMISSIONS_APPROVAL_ON_FAILURE,
        PERMISSIONS_APPROVAL_ON_REQUEST,
        PERMISSIONS_APPROVAL_ON_REQUEST_PERMISSION,
        PERMISSIONS_AUTO_REVIEW_SUFFIX,
        PERMISSIONS_SANDBOX_DANGER_FULL_ACCESS,
        PERMISSIONS_SANDBOX_WORKSPACE_WRITE,
        PERMISSIONS_SANDBOX_READ_ONLY,
        PERMISSIONS_GRANULAR_INTRO,
        PERMISSIONS_REQUEST_TOOL_SECTION,
        CONSEQUENTIAL_TOOL_MESSAGE_TEMPLATES,
        AGENTS_MD_HIERARCHICAL_MESSAGE,
        GOAL_CONTINUATION_PROMPT,
        GOAL_BUDGET_LIMIT_PROMPT,
        GOAL_OBJECTIVE_UPDATED_PROMPT,
        AGENT_ROLE_EXPLORER,
        AGENT_ROLE_AWAITER,
        COLLABORATION_PLAN,
        COLLABORATION_DEFAULT,
        COLLABORATION_EXECUTE,
        COLLABORATION_PAIR_PROGRAMMING,
        MEMORY_READ_PATH,
        MEMORY_CONSOLIDATION,
        MEMORY_STAGE_ONE_INPUT,
        MEMORY_STAGE_ONE_SYSTEM,
        MEMORY_EXTENSION_AD_HOC_INSTRUCTIONS,
        MEMORY_EXTENSIONS_FOLDER_STRUCTURE,
        MEMORY_EXTENSIONS_PRIMARY_INPUTS,
        IMAGE_GENERATION_TOOL_DESCRIPTION,
        WEB_RUN_TOOL_DESCRIPTION,
        TOOL_SEARCH_DESCRIPTION,
        LIST_AVAILABLE_PLUGINS_DESCRIPTION,
        REQUEST_PLUGIN_INSTALL_DESCRIPTION,
        SKILL_MCP_DEPENDENCY_INSTALL_QUESTION,
        SKILL_MCP_DEPENDENCY_INSTALL_DESCRIPTION,
        SKILL_MCP_DEPENDENCY_SKIP_DESCRIPTION,
        SKILL_DESCRIPTION_TRUNCATED_WARNING,
        SKILL_DESCRIPTION_TRUNCATED_WARNING_WITH_PERCENT,
        SKILL_DESCRIPTIONS_REMOVED_WARNING_PREFIX,
        SKILLS_INTRO_WITH_ABSOLUTE_PATHS,
        SKILLS_INTRO_WITH_ALIASES,
        SKILLS_HOW_TO_USE_WITH_ABSOLUTE_PATHS,
        SKILLS_HOW_TO_USE_WITH_ALIASES,
        APPLY_PATCH_TOOL_DESCRIPTION,
        EXEC_COMMAND_TOOL_DESCRIPTION,
        WRITE_STDIN_TOOL_DESCRIPTION,
        SHELL_COMMAND_TOOL_DESCRIPTION,
        REQUEST_PERMISSIONS_TOOL_DESCRIPTION,
        REQUEST_USER_INPUT_TOOL_DESCRIPTION,
        PLAN_TOOL_DESCRIPTION,
        GET_GOAL_TOOL_DESCRIPTION,
        CREATE_GOAL_TOOL_DESCRIPTION,
        UPDATE_GOAL_TOOL_DESCRIPTION,
        MCP_LIST_RESOURCES_TOOL_DESCRIPTION,
        MCP_LIST_RESOURCE_TEMPLATES_TOOL_DESCRIPTION,
        MCP_READ_RESOURCE_TOOL_DESCRIPTION,
        VIEW_IMAGE_TOOL_DESCRIPTION,
        SPAWN_AGENTS_ON_CSV_TOOL_DESCRIPTION,
        REPORT_AGENT_JOB_RESULT_TOOL_DESCRIPTION,
        CODE_MODE_EXEC_TOOL_DESCRIPTION,
        CODE_MODE_WAIT_TOOL_DESCRIPTION,
        MULTI_AGENT_V1_NAMESPACE_DESCRIPTION,
        SPAWN_AGENT_TOOL_DESCRIPTION,
        SPAWN_AGENT_TOOL_DESCRIPTION_V2,
        SEND_INPUT_TOOL_DESCRIPTION,
        SEND_MESSAGE_TOOL_DESCRIPTION,
        ASSIGN_TASK_TOOL_DESCRIPTION,
        RESUME_AGENT_TOOL_DESCRIPTION,
        WAIT_AGENT_TOOL_DESCRIPTION,
        WAIT_AGENT_TOOL_DESCRIPTION_V2,
        LIST_AGENTS_TOOL_DESCRIPTION,
        CLOSE_AGENT_TOOL_DESCRIPTION,
        EXEC_POLICY_PROMPT_CONFLICT_REASON,
        UNIX_ESCALATION_PROMPT_CONFLICT_REASON,
        INIT_COMMAND_PROMPT,
        SIDE_CONVERSATION_BOUNDARY_PROMPT,
        SIDE_CONVERSATION_DEVELOPER_INSTRUCTIONS,
        IDE_CONTEXT_REQUEST_MARKER,
        DYNAMIC_TOOLS_SOURCE_DESCRIPTION,
        MULTI_AGENT_TOOL_SEARCH_SOURCE_DESCRIPTION,
        TEST_SYNC_TOOL_DESCRIPTION,
    ]
    .into_iter()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    static PROMPT_OVERRIDE_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct MapProvider {
        values: BTreeMap<String, String>,
    }

    impl PromptOverrideProvider for MapProvider {
        fn prompt_override(&self, key: &str) -> Option<String> {
            self.values.get(key).cloned()
        }
    }

    #[test]
    fn empty_and_missing_values_fall_back_to_built_in_prompt() {
        let _guard = PROMPT_OVERRIDE_TEST_LOCK.lock().unwrap();
        let mut values = BTreeMap::new();
        values.insert(BASE_INSTRUCTIONS_DEFAULT.to_string(), String::new());
        set_prompt_override_provider(Arc::new(MapProvider { values }));

        assert_eq!(
            resolve_prompt(BASE_INSTRUCTIONS_DEFAULT, "built in"),
            "built in"
        );
        assert_eq!(resolve_prompt(REVIEW_SYSTEM_PROMPT, "review"), "review");

        clear_prompt_override_provider();
    }

    #[test]
    fn non_empty_value_overrides_built_in_prompt() {
        let _guard = PROMPT_OVERRIDE_TEST_LOCK.lock().unwrap();
        let mut values = BTreeMap::new();
        values.insert(
            BASE_INSTRUCTIONS_DEFAULT.to_string(),
            "override prompt".to_string(),
        );
        set_prompt_override_provider(Arc::new(MapProvider { values }));

        assert_eq!(
            resolve_prompt(BASE_INSTRUCTIONS_DEFAULT, "built in"),
            "override prompt"
        );

        clear_prompt_override_provider();
    }
}
