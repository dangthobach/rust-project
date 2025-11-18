///! Integration tests for user management, RBAC, and audit logging
///! Tests admin operations, role-based access control, and audit trail

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use serde_json::{json, Value};
    use tower::ServiceExt;
    
    // Helper function to create test app
    async fn create_test_app() -> axum::Router {
        use sqlx::sqlite::SqlitePoolOptions;
        use crm_backend::app_state::AppState;
        use crm_backend::config::Config;
        use crm_backend::routes::create_router_with_state;

        // Initialize test database (in-memory)
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        // Create test config
        let config = Config {
            database_url: "sqlite::memory:".to_string(),
            jwt_secret: "test-secret-key-for-testing-only-32chars".to_string(),
            jwt_expiration: 86400,
            host: "127.0.0.1".to_string(),
            port: 3000,
            cors_origin: "http://localhost:5173".to_string(),
            max_file_size: 10485760,
            upload_dir: "./test_uploads".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
        };

        // Create AppState (will use InMemoryEventBus since Redis not available in tests)
        let state = AppState::new(pool, config)
            .await
            .expect("Failed to create AppState");

        // Create router
        create_router_with_state(state)
    }
    
    // Helper to create authenticated request with role
    fn create_auth_request(
        method: &str,
        path: &str,
        role: &str,
        body: Option<String>,
    ) -> Request<Body> {
        let token = generate_test_jwt(role);
        
        let mut req = Request::builder()
            .method(method)
            .uri(path)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json");
        
        if let Some(body_str) = body {
            req.body(Body::from(body_str)).unwrap()
        } else {
            req.body(Body::empty()).unwrap()
        }
    }
    
    // Helper to generate test JWT token
    fn generate_test_jwt(role: &str) -> String {
        use jsonwebtoken::{encode, EncodingKey, Header};
        use serde::{Deserialize, Serialize};
        use uuid::Uuid;

        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String,  // user_id
            email: String,
            role: String,
            exp: usize,
        }

        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            email: format!("test_{}@example.com", role),
            role: role.to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
        };

        let secret = "test-secret-key-for-testing-only-32chars";
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("Failed to generate test JWT")
    }
    
    #[tokio::test]
    async fn test_rbac_admin_access_admin_routes() {
        let app = create_test_app().await;
        
        // Admin should access admin routes
        let req = create_auth_request("GET", "/api/admin/users?page=1&limit=10", "admin", None);
        let response = app.oneshot(req).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_rbac_manager_cannot_access_admin_routes() {
        let app = create_test_app().await;
        
        // Manager should NOT access admin routes
        let req = create_auth_request("GET", "/api/admin/users", "manager", None);
        let response = app.oneshot(req).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    
    #[tokio::test]
    async fn test_rbac_user_cannot_access_admin_routes() {
        let app = create_test_app().await;
        
        // User should NOT access admin routes
        let req = create_auth_request("GET", "/api/admin/users", "user", None);
        let response = app.oneshot(req).await.unwrap();
        
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    
    #[tokio::test]
    async fn test_admin_create_user() {
        let app = create_test_app().await;
        
        let user_data = json!({
            "email": "newuser@example.com",
            "name": "New User",
            "password": "SecurePass123!",
            "role": "user"
        });
        
        let req = create_auth_request(
            "POST",
            "/api/admin/users",
            "admin",
            Some(user_data.to_string()),
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        
        // Verify response contains user data
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let user: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(user["email"], "newuser@example.com");
        assert_eq!(user["name"], "New User");
        assert_eq!(user["role"], "user");
    }
    
    #[tokio::test]
    async fn test_admin_update_user() {
        let app = create_test_app().await;
        
        // First create a user
        let user_data = json!({
            "email": "updateuser@example.com",
            "name": "Update User",
            "password": "Pass123!",
            "role": "user"
        });
        
        let create_req = create_auth_request(
            "POST",
            "/api/admin/users",
            "admin",
            Some(user_data.to_string()),
        );
        
        let create_response = app.clone().oneshot(create_req).await.unwrap();
        let body = to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
        let created_user: Value = serde_json::from_slice(&body).unwrap();
        let user_id = created_user["id"].as_str().unwrap();
        
        // Now update the user
        let update_data = json!({
            "name": "Updated Name",
            "role": "manager"
        });
        
        let update_req = create_auth_request(
            "PATCH",
            &format!("/api/admin/users/{}", user_id),
            "admin",
            Some(update_data.to_string()),
        );
        
        let response = app.oneshot(update_req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let updated_user: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(updated_user["name"], "Updated Name");
        assert_eq!(updated_user["role"], "manager");
    }
    
    #[tokio::test]
    async fn test_admin_delete_user() {
        let app = create_test_app().await;
        
        // Create a user to delete
        let user_data = json!({
            "email": "deleteuser@example.com",
            "name": "Delete User",
            "password": "Pass123!",
            "role": "user"
        });
        
        let create_req = create_auth_request(
            "POST",
            "/api/admin/users",
            "admin",
            Some(user_data.to_string()),
        );
        
        let create_response = app.clone().oneshot(create_req).await.unwrap();
        let body = to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
        let created_user: Value = serde_json::from_slice(&body).unwrap();
        let user_id = created_user["id"].as_str().unwrap();
        
        // Delete the user
        let delete_req = create_auth_request(
            "DELETE",
            &format!("/api/admin/users/{}", user_id),
            "admin",
            None,
        );
        
        let response = app.oneshot(delete_req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_admin_bulk_delete_users() {
        let app = create_test_app().await;
        
        // Create multiple users
        let mut user_ids = Vec::new();
        for i in 0..3 {
            let user_data = json!({
                "email": format!("bulkuser{}@example.com", i),
                "name": format!("Bulk User {}", i),
                "password": "Pass123!",
                "role": "user"
            });
            
            let req = create_auth_request(
                "POST",
                "/api/admin/users",
                "admin",
                Some(user_data.to_string()),
            );
            
            let response = app.clone().oneshot(req).await.unwrap();
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let user: Value = serde_json::from_slice(&body).unwrap();
            user_ids.push(user["id"].as_str().unwrap().to_string());
        }
        
        // Bulk delete
        let bulk_data = json!({
            "user_ids": user_ids
        });
        
        let req = create_auth_request(
            "POST",
            "/api/admin/users/bulk-delete",
            "admin",
            Some(bulk_data.to_string()),
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(result["deleted"], 3);
    }
    
    #[tokio::test]
    async fn test_admin_bulk_update_role() {
        let app = create_test_app().await;
        
        // Create multiple users
        let mut user_ids = Vec::new();
        for i in 0..3 {
            let user_data = json!({
                "email": format!("roleuser{}@example.com", i),
                "name": format!("Role User {}", i),
                "password": "Pass123!",
                "role": "user"
            });
            
            let req = create_auth_request(
                "POST",
                "/api/admin/users",
                "admin",
                Some(user_data.to_string()),
            );
            
            let response = app.clone().oneshot(req).await.unwrap();
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let user: Value = serde_json::from_slice(&body).unwrap();
            user_ids.push(user["id"].as_str().unwrap().to_string());
        }
        
        // Bulk update role
        let bulk_data = json!({
            "user_ids": user_ids,
            "role": "manager"
        });
        
        let req = create_auth_request(
            "POST",
            "/api/admin/users/bulk-update-role",
            "admin",
            Some(bulk_data.to_string()),
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(result["updated"], 3);
    }
    
    #[tokio::test]
    async fn test_audit_log_created_on_admin_action() {
        // This test would require database access to verify audit logs
        // Placeholder for audit log verification
        
        let app = create_test_app().await;
        
        // Perform admin action
        let user_data = json!({
            "email": "audituser@example.com",
            "name": "Audit User",
            "password": "Pass123!",
            "role": "user"
        });
        
        let req = create_auth_request(
            "POST",
            "/api/admin/users",
            "admin",
            Some(user_data.to_string()),
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        
        // TODO: Query audit_logs table to verify log entry exists
        // Should verify:
        // - action = "create_user"
        // - resource = "user"
        // - details contains user email
        // - user_id matches admin user
        // - ip_address is recorded
    }
    
    #[tokio::test]
    async fn test_user_profile_update() {
        let app = create_test_app().await;
        
        let update_data = json!({
            "name": "Updated Profile Name"
        });
        
        let req = create_auth_request(
            "PATCH",
            "/api/users/me/profile",
            "user",
            Some(update_data.to_string()),
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let user: Value = serde_json::from_slice(&body).unwrap();
        
        assert_eq!(user["name"], "Updated Profile Name");
    }
    
    #[tokio::test]
    async fn test_user_password_change() {
        let app = create_test_app().await;
        
        let password_data = json!({
            "old_password": "OldPass123!",
            "new_password": "NewPass123!"
        });
        
        let req = create_auth_request(
            "POST",
            "/api/users/me/password",
            "user",
            Some(password_data.to_string()),
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    #[tokio::test]
    async fn test_user_stats_retrieval() {
        let app = create_test_app().await;
        
        let req = create_auth_request("GET", "/api/users/me/stats", "user", None);
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let stats: Value = serde_json::from_slice(&body).unwrap();
        
        assert!(stats.get("tasks").is_some());
        assert!(stats.get("clients").is_some());
        assert!(stats.get("files").is_some());
    }
    
    #[tokio::test]
    async fn test_admin_search_users() {
        let app = create_test_app().await;
        
        // Create user with searchable name
        let user_data = json!({
            "email": "searchable@example.com",
            "name": "Searchable User Name",
            "password": "Pass123!",
            "role": "user"
        });
        
        let create_req = create_auth_request(
            "POST",
            "/api/admin/users",
            "admin",
            Some(user_data.to_string()),
        );
        
        app.clone().oneshot(create_req).await.unwrap();
        
        // Search for user
        let search_req = create_auth_request(
            "GET",
            "/api/admin/users/search?q=Searchable",
            "admin",
            None,
        );
        
        let response = app.oneshot(search_req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&body).unwrap();
        
        let users = result["data"].as_array().unwrap();
        assert!(users.len() > 0);
        assert!(users.iter().any(|u| u["name"].as_str().unwrap().contains("Searchable")));
    }
    
    #[tokio::test]
    async fn test_admin_filter_users_by_role() {
        let app = create_test_app().await;
        
        let req = create_auth_request(
            "GET",
            "/api/admin/users?role=admin&page=1&limit=10",
            "admin",
            None,
        );
        
        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&body).unwrap();
        
        let users = result["data"].as_array().unwrap();
        // Verify all returned users have admin role
        for user in users {
            assert_eq!(user["role"], "admin");
        }
    }
}
