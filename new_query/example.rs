use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn, instrument};

mod query;
use query::*;

mod tracing_setup;
use tracing_setup::*;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct Post {
    id: u32,
    user_id: u32,
    title: String,
    body: String,
}

#[derive(Clone, Serialize)]
struct CreateUserInput {
    name: String,
    email: String,
}

fn App() -> Element {
    let mut user_id = use_signal(|| {
        info!("Initializing user_id signal with default value 1");
        1u32
    });
    
    // ===== BASIC QUERY WITH REACTIVE KEY =====
    let user_key = QueryKey::new(&["user", &user_id().to_string()]);
    debug!(key = %user_key, "Creating user query key");
    
    let (user_query, refetch_user) = use_query(
        user_key.clone(),
        move || {
            info!(user_id = user_id(), "User fetcher called");
            async move {
                let url = format!("https://jsonplaceholder.typicode.com/users/{}", user_id());
                debug!(url = %url, "Fetching user data");
                
                let res = reqwest::get(&url)
                    .await
                    .map_err(|e| {
                        error!(error = %e, url = %url, "HTTP request failed");
                        e.to_string()
                    })?;
                
                let user = res.json::<User>().await.map_err(|e| {
                    error!(error = %e, url = %url, "Failed to parse JSON response");
                    e.to_string()
                })?;
                
                info!(user_id = user.id, user_name = %user.name, "User fetched successfully");
                Ok(user)
            }
        },
        QueryOptions {
            stale_time: Some(30_000),
            retry: Some(2),
            ..Default::default()
        },
    );

    // ===== DEPENDENT QUERY (only runs when user loads) =====
    let posts_key = QueryKey::new(&["posts", &user_id().to_string()]);
    debug!(key = %posts_key, enabled = user_query.data.is_some(), "Creating posts query key");
    
    let (posts_query, _) = use_query(
        posts_key.clone(),
        move || {
            info!(user_id = user_id(), "Posts fetcher called");
            async move {
                let url = format!(
                    "https://jsonplaceholder.typicode.com/posts?userId={}",
                    user_id()
                );
                debug!(url = %url, "Fetching posts data");
                
                let res = reqwest::get(&url)
                    .await
                    .map_err(|e| {
                        error!(error = %e, url = %url, "HTTP request failed");
                        e.to_string()
                    })?;
                
                let posts = res.json::<Vec<Post>>().await.map_err(|e| {
                    error!(error = %e, url = %url, "Failed to parse JSON response");
                    e.to_string()
                })?;
                
                info!(user_id = user_id(), post_count = posts.len(), "Posts fetched successfully");
                Ok(posts)
            }
        },
        QueryOptions {
            enabled: user_query.data.is_some(), // Only fetch when user exists
            stale_time: Some(60_000),
            ..Default::default()
        },
    );

    // ===== MUTATION WITH CALLBACKS =====
    let (create_mutation, mutate_create) = use_mutation(
        |input: CreateUserInput| {
            info!(name = %input.name, email = %input.email, "Create mutation called");
            async move {
                debug!("Simulating user creation API call");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                
                let user = User {
                    id: 999,
                    name: input.name.clone(),
                    email: input.email.clone(),
                };
                
                info!(user_id = user.id, user_name = %user.name, "User creation simulated");
                Ok(user)
            }
        },
        MutationCallbacks {
            on_success: Some(Rc::new(|user: User| {
                info!(
                    user_id = user.id,
                    user_name = %user.name,
                    "on_success: User created successfully"
                );
            })),
            on_error: Some(Rc::new(|err: String| {
                error!(error = %err, "on_error: User creation failed");
            })),
            on_settled: Some(Rc::new(|| {
                debug!("on_settled: Mutation completed");
            })),
            invalidate_keys: vec![QueryKey::new(&["user", "1"])],
            optimistic_update: None,
        },
    );

    // ===== UPDATE MUTATION WITH OPTIMISTIC UPDATE =====
    let (update_mutation, mutate_update) = use_mutation(
        move |new_name: String| {
            info!(new_name = %new_name, "Update mutation called");
            async move {
                debug!("Simulating user update API call");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                
                let user = User {
                    id: user_id(),
                    name: new_name.clone(),
                    email: "updated@example.com".to_string(),
                };
                
                info!(user_id = user.id, new_name = %user.name, "User update simulated");
                Ok(user)
            }
        },
        MutationCallbacks {
            invalidate_keys: vec![user_key.clone()],
            optimistic_update: Some((
                user_key.clone(),
                User {
                    id: user_id(),
                    name: "Optimistic Update...".to_string(),
                    email: "optimistic@example.com".to_string(),
                },
            )),
            on_success: Some(Rc::new(|user: User| {
                info!(user_name = %user.name, "User updated successfully");
            })),
            ..Default::default()
        },
    );

    rsx! {
        div { style: "padding: 20px; font-family: sans-serif;",
            
            h1 { "Enhanced Dioxus Query with Tracing" }
            
            // Tracing Info Panel
            section { style: "border: 2px solid #4CAF50; padding: 15px; margin: 10px 0; background: #f0f8f0;",
                h3 { "📊 Tracing Information" }
                p { "Check your console and ./logs/dioxus_query.log for detailed logs!" }
                ul {
                    li { code { "TRACE" } " - Very detailed execution flow" }
                    li { code { "DEBUG" } " - Cache hits/misses, state changes" }
                    li { code { "INFO" } " - Query lifecycle, mutations" }
                    li { code { "WARN" } " - Retries, potential issues" }
                    li { code { "ERROR" } " - Failures, exceptions" }
                }
                button {
                    onclick: move |_| {
                        let (size, keys) = get_cache_stats();
                        info!(cache_size = size, "Current cache stats");
                        debug!(cache_keys = ?keys, "Cache keys");
                    },
                    "Log Cache Stats"
                }
            }
            
            // User ID Selector
            div { style: "margin: 20px 0;",
                label { "Select User ID: " }
                select {
                    value: "{user_id}",
                    onchange: move |evt| {
                        if let Ok(id) = evt.value().parse::<u32>() {
                            info!(old_id = user_id(), new_id = id, "User ID changed");
                            user_id.set(id);
                        } else {
                            warn!(value = %evt.value(), "Invalid user ID entered");
                        }
                    },
                    for id in 1..=10 {
                        option { value: "{id}", "{id}" }
                    }
                }
            }

            // User Query Display
            section { style: "border: 1px solid #ccc; padding: 15px; margin: 10px 0;",
                h2 { "User Data" }
                
                if user_query.is_loading {
                    p { "Loading user..." }
                } else if user_query.is_fetching {
                    p { "🔄 Refetching in background..." }
                } else if user_query.is_error {
                    p { style: "color: red;",
                        "Error: {user_query.error.as_ref().unwrap()}"
                    }
                } else if let Some(user) = &user_query.data {
                    div {
                        p { strong { "Name: " } "{user.name}" }
                        p { strong { "Email: " } "{user.email}" }
                        p { strong { "ID: " } "{user.id}" }
                    }
                }

                div { style: "margin-top: 10px;",
                    button {
                        onclick: move |_| {
                            info!("Manual refetch button clicked");
                            refetch_user();
                        },
                        "Manual Refetch"
                    }
                    button {
                        style: "margin-left: 10px;",
                        onclick: move |_| {
                            info!(key = %user_key, "Invalidate cache button clicked");
                            invalidate_query(&user_key);
                        },
                        "Invalidate Cache"
                    }
                }
            }

            // Posts Query Display (dependent)
            section { style: "border: 1px solid #ccc; padding: 15px; margin: 10px 0;",
                h2 { "User's Posts" }
                
                if posts_query.is_loading {
                    p { "Loading posts..." }
                } else if posts_query.is_error {
                    p { style: "color: red;",
                        "Error: {posts_query.error.as_ref().unwrap()}"
                    }
                } else if let Some(posts) = &posts_query.data {
                    p { "Found {posts.len()} posts:" }
                    ul {
                        for post in posts.iter().take(3) {
                            li { key: "{post.id}",
                                strong { "{post.title}" }
                                p { style: "font-size: 0.9em; color: #666;",
                                    "{post.body}"
                                }
                            }
                        }
                    }
                }
            }

            // Create Mutation
            section { style: "border: 1px solid #ccc; padding: 15px; margin: 10px 0;",
                h2 { "Create User Mutation" }
                
                button {
                    disabled: create_mutation.is_loading,
                    onclick: move |_| {
                        info!("Create user button clicked");
                        mutate_create(CreateUserInput {
                            name: "New User".to_string(),
                            email: "new@example.com".to_string(),
                        });
                    },
                    if create_mutation.is_loading {
                        "Creating..."
                    } else {
                        "Create New User"
                    }
                }

                if create_mutation.is_success {
                    p { style: "color: green;",
                        "✓ User created successfully!"
                    }
                }
                
                if create_mutation.is_error {
                    p { style: "color: red;",
                        "Error: {create_mutation.error.as_ref().unwrap()}"
                    }
                }
            }

            // Update Mutation with Optimistic Update
            section { style: "border: 1px solid #ccc; padding: 15px; margin: 10px 0;",
                h2 { "Optimistic Update Mutation" }
                
                button {
                    disabled: update_mutation.is_loading,
                    onclick: move |_| {
                        info!("Update user button clicked (with optimistic update)");
                        mutate_update("Updated Name (Optimistic)".to_string());
                    },
                    if update_mutation.is_loading {
                        "Updating..."
                    } else {
                        "Update User Name (with optimistic update)"
                    }
                }

                if update_mutation.is_success {
                    p { style: "color: green;",
                        "✓ User updated!"
                    }
                }
            }

            // Global Actions
            section { style: "border: 1px solid #ccc; padding: 15px; margin: 10px 0;",
                h2 { "Global Cache Actions" }
                
                button {
                    onclick: move |_| {
                        info!("Invalidating all user queries");
                        invalidate_queries_by_prefix("user");
                    },
                    "Invalidate All User Queries"
                }
                
                button {
                    style: "margin-left: 10px;",
                    onclick: move |_| {
                        warn!("Clearing all caches - this will trigger refetches!");
                        invalidate_all_queries();
                    },
                    "Clear All Caches"
                }
            }
        }
    }
}

fn main() {
    // Initialize tracing before starting the app
    // Choose the appropriate initialization for your environment:
    
    // Development: verbose console logs
    init_tracing_dev();
    
    // Production: structured JSON logs to file
    // init_tracing_prod();
    
    // Default: console + file with INFO level
    // init_tracing();
    
    info!("Starting Dioxus application with query system");
    dioxus::launch(App);
}