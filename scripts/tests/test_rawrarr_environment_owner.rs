use super::*;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::str::FromStr;

#[test]
fn owner_request_rejects_recipient_private_material_and_urls() {
    assert!(serde_json::from_str::<ProvisionRequest>(r#"{"nodeId":"fossil"}"#).is_ok());
    for extra in ["ageRecipient", "identity", "url"] {
        let body = serde_json::json!({"nodeId":"fossil", extra:"untrusted"});
        assert!(serde_json::from_value::<ProvisionRequest>(body).is_err());
    }
}

#[tokio::test]
#[ignore = "requires RAWARR_TEST_DATABASE_URL ending in _test and age v1.3.2"]
async fn environment_owner_database_and_api() {
    let url = env::var("RAWARR_TEST_DATABASE_URL").expect("disposable PostgreSQL URL");
    let parsed = reqwest::Url::parse(&url).unwrap();
    assert!(parsed.path().ends_with("_test"), "refuse non-test database");
    let admin = PgPool::connect(&url).await.unwrap();
    let schema = format!("owner_{}", Uuid::new_v4().simple());
    sqlx::query(&format!("CREATE SCHEMA {schema}"))
        .execute(&admin)
        .await
        .unwrap();
    let options = PgConnectOptions::from_str(&url)
        .unwrap()
        .options([("search_path", schema.as_str())]);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM rawrarr_environment_owner")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "fresh migration must not infer ownership");
    // Reapply just the new migration after adding legacy account state.
    sqlx::query("DROP TABLE rawrarr_environment_owner")
        .execute(&pool)
        .await
        .unwrap();
    let user = Uuid::new_v4();
    sqlx::query("INSERT INTO rawrarr_users(id,username,display_name) VALUES($1,'owner','Owner')")
        .bind(user)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::raw_sql(include_str!(
        "../../rust-arr/migrations/0012_environment_owner.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    let account: Uuid = sqlx::query_scalar("SELECT id FROM rawrarr_users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(account, user);
    sqlx::query("INSERT INTO rawrarr_nodes(id,role,token_sha256,origin,rp_id,user_id) VALUES('fossil','metadata',$1,'https://fossil.example','example',$2),('storage','storage',$3,'https://storage.example','example',$2)")
        .bind("a".repeat(64)).bind(user).bind("b".repeat(64)).execute(&pool).await.unwrap();
    let webauthn = webauthn_rs::prelude::WebauthnBuilder::new(
        "example",
        &reqwest::Url::parse("https://coordinator.example").unwrap(),
    )
    .unwrap()
    .build()
    .unwrap();
    let auth = AuthState {
        pool: pool.clone(),
        webauthn: webauthn.clone(),
        secure_cookie: true,
        node_id: "coordinator".to_owned(),
        next_path: "/",
    };
    let mut tx = pool.begin().await.unwrap();
    let token = crate::auth::issue_session(&mut tx, user, "coordinator")
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        HeaderValue::from_str(&format!("rawrarr_session={}", token.as_str())).unwrap(),
    );
    headers.insert(
        header::ORIGIN,
        HeaderValue::from_static("https://coordinator.example"),
    );
    let state = CoordinatorState {
        pool: pool.clone(),
        coordinator_auth: auth,
        webauthn,
        web_origin: "https://coordinator.example".to_owned(),
        relay_targets: HashMap::new(),
        direct_targets: HashMap::new(),
        relay_client: reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap(),
        approved: Arc::new(RwLock::new(vec![])),
        stripe_billing: stripe::BillingState::from_env().unwrap(),
        avatar_cache: Arc::new(RwLock::new(HashMap::new())),
    };
    let req = || {
        Json(ProvisionRequest {
            node_id: "fossil".to_owned(),
        })
    };
    assert_eq!(
        provision(State(state.clone()), HeaderMap::new(), req())
            .await
            .unwrap_err()
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        get(State(state.clone()), headers.clone())
            .await
            .unwrap_err()
            .0,
        StatusCode::NOT_FOUND
    );
    let mut cross_site = headers.clone();
    cross_site.insert(
        header::ORIGIN,
        HeaderValue::from_static("https://evil.example"),
    );
    assert_eq!(
        provision(State(state.clone()), cross_site, req())
            .await
            .unwrap_err()
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        provision(
            State(state.clone()),
            headers.clone(),
            Json(ProvisionRequest {
                node_id: "storage".to_owned()
            })
        )
        .await
        .unwrap_err()
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        provision(State(state.clone()), headers.clone(), req())
            .await
            .unwrap_err()
            .0,
        StatusCode::SERVICE_UNAVAILABLE
    );

    let generated = std::process::Command::new("age-keygen")
        .arg("-pq")
        .output()
        .unwrap();
    assert!(generated.status.success());
    let identity = zeroize::Zeroizing::new(String::from_utf8(generated.stdout).unwrap());
    let mut keygen = std::process::Command::new("age-keygen")
        .arg("-y")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    keygen
        .stdin
        .take()
        .unwrap()
        .write_all(identity.as_bytes())
        .unwrap();
    let recipient = String::from_utf8(keygen.wait_with_output().unwrap().stdout)
        .unwrap()
        .trim()
        .to_owned();
    let reply = Arc::new(RwLock::new((StatusCode::OK, recipient.clone())));
    let reply_state = reply.clone();
    let mock_pool = pool.clone();
    let fossil = Router::new().route("/api/metadata/export/owner-recipient", axum::routing::get(move |headers: HeaderMap| {
        let reply = reply_state.clone();
        let pool = mock_pool.clone();
        async move {
            let token = crate::auth::cookie_token(&headers).expect("delegated fossil session");
            let valid: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM rawrarr_sessions WHERE token_hash=$1 AND node_id='fossil' AND expires_at>now())")
                .bind(Sha256::digest(token.as_bytes()).to_vec()).fetch_one(&pool).await.unwrap();
            assert!(valid);
            reply.read().await.clone()
        }
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, fossil).await.unwrap();
    });
    let mut state = state;
    state.relay_targets.insert(
        "fossil".to_owned(),
        format!("http://{address}/").parse().unwrap(),
    );
    for (upstream, expected) in [
        (StatusCode::NOT_FOUND, StatusCode::PRECONDITION_REQUIRED),
        (StatusCode::LOCKED, StatusCode::LOCKED),
        (StatusCode::UNAUTHORIZED, StatusCode::BAD_GATEWAY),
    ] {
        *reply.write().await = (upstream, String::new());
        assert_eq!(
            provision(State(state.clone()), headers.clone(), req())
                .await
                .unwrap_err()
                .0,
            expected
        );
    }
    for invalid in [
        "AGE-SECRET-KEY-PQ-1private".to_owned(),
        "x".repeat(4097),
        "age1pq1".to_owned() + &"a".repeat(32),
    ] {
        *reply.write().await = (StatusCode::OK, invalid);
        assert_eq!(
            provision(State(state.clone()), headers.clone(), req())
                .await
                .unwrap_err()
                .0,
            StatusCode::BAD_GATEWAY
        );
    }
    *reply.write().await = (StatusCode::OK, recipient.clone());
    let (first, competing) = tokio::join!(
        pin(&pool, user, "fossil", &recipient),
        pin(&pool, user, "fossil", "age1pq1competing000000000000")
    );
    assert_eq!(
        usize::from(first.is_ok()) + usize::from(competing.is_ok()),
        1
    );
    let conflict = if let Err(error) = first {
        error
    } else {
        competing.err().unwrap()
    };
    assert_eq!(conflict.0, StatusCode::CONFLICT);
    sqlx::query("DELETE FROM rawrarr_environment_owner")
        .execute(&pool)
        .await
        .unwrap();
    for _ in 0..2 {
        let response = provision(State(state.clone()), headers.clone(), req())
            .await
            .unwrap();
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
    }
    let (a, b) = tokio::join!(
        pin(&pool, user, "fossil", &recipient),
        pin(&pool, user, "fossil", "age1pq1conflict000000000000")
    );
    assert!(a.is_ok());
    assert_eq!(b.err().unwrap().0, StatusCode::CONFLICT);
    server.abort();
    assert!(server.await.unwrap_err().is_cancelled());
    assert_eq!(
        provision(State(state.clone()), headers.clone(), req())
            .await
            .unwrap_err()
            .0,
        StatusCode::BAD_GATEWAY
    );
    let other = Uuid::new_v4();
    sqlx::query("INSERT INTO rawrarr_users(id,username,display_name) VALUES($1,'other','Other')")
        .bind(other)
        .execute(&pool)
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let other_token = crate::auth::issue_session(&mut tx, other, "coordinator")
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let mut other_headers = headers.clone();
    other_headers.insert(
        header::COOKIE,
        HeaderValue::from_str(&format!("rawrarr_session={}", other_token.as_str())).unwrap(),
    );
    assert_eq!(
        get(State(state.clone()), other_headers.clone())
            .await
            .unwrap_err()
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        provision(State(state.clone()), other_headers, req())
            .await
            .unwrap_err()
            .0,
        StatusCode::FORBIDDEN
    );
    sqlx::query("UPDATE rawrarr_nodes SET user_id=$1 WHERE id='fossil'")
        .bind(other)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        pin(&pool, user, "fossil", &recipient)
            .await
            .err()
            .unwrap()
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        pin(&pool, other, "fossil", &recipient)
            .await
            .err()
            .unwrap()
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        provision(State(state.clone()), headers.clone(), req())
            .await
            .unwrap_err()
            .0,
        StatusCode::FORBIDDEN
    );
    sqlx::query("DELETE FROM rawrarr_nodes WHERE id='fossil'")
        .execute(&pool)
        .await
        .unwrap();
    let response = get(State(state), headers).await.unwrap();
    let body = to_bytes(response.into_body(), 8192).await.unwrap();
    let pinned: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(pinned["ageRecipient"], recipient);
    assert_eq!(pinned["ownerUserId"], user.to_string());
    assert_eq!(pinned["sourceNodeId"], "fossil");
    assert!(!String::from_utf8(body.to_vec()).unwrap().contains("SECRET"));
    sqlx::query("DELETE FROM rawrarr_environment_owner")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        pin(&pool, other, "storage", &recipient)
            .await
            .err()
            .unwrap()
            .0,
        StatusCode::FORBIDDEN
    );
    sqlx::query("UPDATE rawrarr_nodes SET role='metadata', user_id=$1 WHERE id='storage'")
        .bind(other)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        pin(&pool, other, "storage", &recipient)
            .await
            .err()
            .unwrap()
            .0,
        StatusCode::CONFLICT
    );
    sqlx::query("INSERT INTO rawrarr_instance_signing_root(singleton,algorithm,public_key,fingerprint,owner_user_id) VALUES(TRUE,'ML-DSA-65',$1,$2,$3)")
        .bind(vec![0u8; 1952]).bind("f".repeat(64)).bind(user).execute(&pool).await.unwrap();
    assert_eq!(
        pin(&pool, other, "storage", &recipient)
            .await
            .err()
            .unwrap()
            .0,
        StatusCode::FORBIDDEN
    );
    pool.close().await;
    sqlx::query(&format!("DROP SCHEMA {schema} CASCADE"))
        .execute(&admin)
        .await
        .unwrap();
    admin.close().await;
}
