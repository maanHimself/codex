use super::*;
use codex_config::CloudConfigFragment;
use codex_config::CloudConfigTomlBundle;
use codex_config::CloudRequirementsFragment;
use codex_config::CloudRequirementsTomlBundle;
use pretty_assertions::assert_eq;
use std::path::Path;
use tempfile::tempdir;

fn test_bundle() -> CloudConfigBundle {
    CloudConfigBundle {
        config_toml: CloudConfigTomlBundle {
            enterprise_managed: vec![CloudConfigFragment {
                id: "cfg_1".to_string(),
                name: "Base config".to_string(),
                contents: "model = \"gpt-5\"".to_string(),
            }],
        },
        requirements_toml: CloudRequirementsTomlBundle {
            enterprise_managed: vec![CloudRequirementsFragment {
                id: "req_1".to_string(),
                name: "Base requirements".to_string(),
                contents: "allowed_approval_policies = [\"never\"]".to_string(),
            }],
        },
    }
}

struct NeverFetcher;

#[async_trait]
impl RequirementsFetcher for NeverFetcher {
    async fn fetch_requirements(
        &self,
        _auth: &CodexAuth,
    ) -> Result<CloudConfigBundle, FetchAttemptError> {
        panic!("cache tests should not fetch from remote");
    }
}

async fn create_test_service(codex_home: &Path) -> CloudRequirementsService {
    let auth_home = tempdir().expect("tempdir");
    let auth_manager = Arc::new(
        AuthManager::new(
            auth_home.path().to_path_buf(),
            /*enable_codex_api_key_env*/ false,
            AuthCredentialsStoreMode::File,
            /*chatgpt_base_url*/ None,
        )
        .await,
    );
    CloudRequirementsService::new(
        auth_manager,
        Arc::new(NeverFetcher),
        codex_home.to_path_buf(),
        CLOUD_REQUIREMENTS_TIMEOUT,
    )
}

fn signed_cache_file(
    signed_payload: CloudRequirementsCacheSignedPayloadV1,
) -> CloudRequirementsCacheFileV1 {
    let payload_bytes = cache_payload_bytes(&signed_payload).expect("payload bytes");
    CloudRequirementsCacheFileV1 {
        signature: sign_cache_payload(&payload_bytes).expect("signature"),
        signed_payload,
    }
}

fn valid_signed_payload() -> CloudRequirementsCacheSignedPayloadV1 {
    let cached_at = Utc::now();
    CloudRequirementsCacheSignedPayloadV1 {
        version: CLOUD_CONFIG_BUNDLE_CACHE_VERSION,
        cached_at,
        expires_at: cached_at + ChronoDuration::minutes(30),
        chatgpt_user_id: Some("user-12345".to_string()),
        account_id: Some("account-12345".to_string()),
        bundle: test_bundle(),
    }
}

fn write_cache_file(cache_path: &Path, cache_file: &CloudRequirementsCacheFileV1) {
    std::fs::write(
        cache_path,
        serde_json::to_vec_pretty(cache_file).expect("serialize cache"),
    )
    .expect("write cache");
}

#[tokio::test]
async fn save_writes_signed_payload_and_loads_for_matching_identity() {
    let codex_home = tempdir().expect("tempdir");
    let service = create_test_service(codex_home.path()).await;
    let bundle = test_bundle();

    service
        .save_cache(
            Some("user-12345".to_string()),
            Some("account-12345".to_string()),
            bundle.clone(),
        )
        .await
        .expect("save cache");

    let cache_file: CloudRequirementsCacheFileV1 =
        serde_json::from_slice(&std::fs::read(&service.cache_path).expect("read cache"))
            .expect("parse cache");
    let expected_payload = CloudRequirementsCacheSignedPayloadV1 {
        version: CLOUD_CONFIG_BUNDLE_CACHE_VERSION,
        cached_at: cache_file.signed_payload.cached_at,
        expires_at: cache_file.signed_payload.expires_at,
        chatgpt_user_id: Some("user-12345".to_string()),
        account_id: Some("account-12345".to_string()),
        bundle,
    };
    let expected_cache_file = signed_cache_file(expected_payload);
    assert_eq!(cache_file, expected_cache_file);
    assert!(
        cache_file.signed_payload.expires_at
            <= cache_file.signed_payload.cached_at + ChronoDuration::minutes(30)
    );
    assert!(cache_file.signed_payload.expires_at > cache_file.signed_payload.cached_at);

    assert_eq!(
        service
            .load_cache(Some("user-12345"), Some("account-12345"))
            .await,
        Ok(cache_file.signed_payload)
    );
}

#[tokio::test]
async fn load_rejects_missing_request_identity_before_reading_cache_file() {
    let codex_home = tempdir().expect("tempdir");
    let service = create_test_service(codex_home.path()).await;

    assert_eq!(
        service
            .load_cache(/*chatgpt_user_id*/ None, Some("account-12345"))
            .await,
        Err(CacheLoadStatus::AuthIdentityIncomplete)
    );
    assert_eq!(
        service
            .load_cache(Some("user-12345"), /*account_id*/ None)
            .await,
        Err(CacheLoadStatus::AuthIdentityIncomplete)
    );
}

#[tokio::test]
async fn load_reports_missing_and_malformed_cache_files() {
    let codex_home = tempdir().expect("tempdir");
    let service = create_test_service(codex_home.path()).await;

    assert_eq!(
        service
            .load_cache(Some("user-12345"), Some("account-12345"))
            .await,
        Err(CacheLoadStatus::CacheFileNotFound)
    );

    std::fs::write(&service.cache_path, "{").expect("write malformed cache");
    assert!(matches!(
        service
            .load_cache(Some("user-12345"), Some("account-12345"))
            .await,
        Err(CacheLoadStatus::CacheParseFailed(_))
    ));
}

#[tokio::test]
async fn load_rejects_tampered_payload() {
    let codex_home = tempdir().expect("tempdir");
    let service = create_test_service(codex_home.path()).await;
    let mut cache_file = signed_cache_file(valid_signed_payload());
    cache_file
        .signed_payload
        .bundle
        .requirements_toml
        .enterprise_managed[0]
        .contents = "allowed_approval_policies = [\"on-request\"]".to_string();
    write_cache_file(&service.cache_path, &cache_file);

    assert_eq!(
        service
            .load_cache(Some("user-12345"), Some("account-12345"))
            .await,
        Err(CacheLoadStatus::CacheSignatureInvalid)
    );
}

#[tokio::test]
async fn load_rejects_cache_for_incomplete_or_different_identity() {
    let codex_home = tempdir().expect("tempdir");
    let service = create_test_service(codex_home.path()).await;
    let cache_file = signed_cache_file(valid_signed_payload());
    write_cache_file(&service.cache_path, &cache_file);

    assert_eq!(
        service
            .load_cache(Some("user-99999"), Some("account-12345"))
            .await,
        Err(CacheLoadStatus::CacheIdentityMismatch)
    );

    let mut signed_payload = valid_signed_payload();
    signed_payload.chatgpt_user_id = None;
    write_cache_file(&service.cache_path, &signed_cache_file(signed_payload));

    assert_eq!(
        service
            .load_cache(Some("user-12345"), Some("account-12345"))
            .await,
        Err(CacheLoadStatus::CacheIdentityIncomplete)
    );
}

#[tokio::test]
async fn load_rejects_expired_cache() {
    let codex_home = tempdir().expect("tempdir");
    let service = create_test_service(codex_home.path()).await;
    let mut signed_payload = valid_signed_payload();
    signed_payload.expires_at = Utc::now() - ChronoDuration::seconds(1);
    write_cache_file(&service.cache_path, &signed_cache_file(signed_payload));

    assert_eq!(
        service
            .load_cache(Some("user-12345"), Some("account-12345"))
            .await,
        Err(CacheLoadStatus::CacheExpired)
    );
}

#[tokio::test]
async fn load_rejects_unsupported_cache_version() {
    let codex_home = tempdir().expect("tempdir");
    let service = create_test_service(codex_home.path()).await;
    let mut signed_payload = valid_signed_payload();
    signed_payload.version = 2;
    write_cache_file(&service.cache_path, &signed_cache_file(signed_payload));

    assert_eq!(
        service
            .load_cache(Some("user-12345"), Some("account-12345"))
            .await,
        Err(CacheLoadStatus::CacheVersionUnsupported(2))
    );
}
