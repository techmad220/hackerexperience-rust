//! Macros for plugin registration
//!
//! This module provides convenient macros for registering plugins
//! and defining plugin configurations.

/// Macro for registering multiple plugins at once
///
/// # Example
///
/// ```rust
/// register_plugins! {
///     manager,
///     AuthPlugin::new(),
///     ProcessPlugin::new(),
///     HardwarePlugin::new(),
///     InternetPlugin::new(),
///     SoftwarePlugin::new()
/// }
/// ```
#[macro_export]
macro_rules! register_plugins {
    ($manager:expr, $($plugin:expr),* $(,)?) => {
        {
            let mut _result = Ok(());
            $(
                if let Err(e) = $manager.register(Box::new($plugin)) {
                    tracing::error!("Failed to register plugin: {}", e);
                    _result = Err(e);
                }
            )*
            _result
        }
    };
}

/// Macro for defining a plugin with minimal boilerplate
///
/// # Example
///
/// ```rust
/// define_plugin! {
///     name: "auth",
///     version: "1.0.0",
///     description: "Authentication and session management",
///     dependencies: [],
///     routes: |scope| {
///         scope
///             .route("/login", web::post().to(login))
///             .route("/logout", web::post().to(logout))
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_plugin {
    (
        name: $name:expr,
        version: $version:expr,
        description: $description:expr,
        dependencies: [$($dep:expr),* $(,)?],
        routes: $routes:expr $(,)?
    ) => {
        {
            struct GeneratedPlugin;

            #[async_trait::async_trait]
            impl $crate::plugins::Plugin for GeneratedPlugin {
                fn name(&self) -> &'static str {
                    $name
                }

                fn version(&self) -> &'static str {
                    $version
                }

                fn description(&self) -> &'static str {
                    $description
                }

                fn dependencies(&self) -> Vec<&'static str> {
                    vec![$($dep),*]
                }

                fn register_routes(&self, scope: actix_web::Scope) -> actix_web::Scope {
                    $routes(scope)
                }
            }

            GeneratedPlugin
        }
    };
}

/// Macro for creating a plugin with full customization
///
/// # Example
///
/// ```rust
/// create_plugin! {
///     struct MyPlugin {
///         config: MyConfig,
///     }
///
///     impl Plugin for MyPlugin {
///         name: "my_plugin",
///         version: "1.0.0",
///         description: "My custom plugin",
///
///         async fn initialize(&self, context: &mut PluginContext) -> Result<(), PluginError> {
///             // Custom initialization
///             Ok(())
///         }
///
///         fn register_routes(&self, scope: Scope) -> Scope {
///             scope.route("/my-route", web::get().to(my_handler))
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! create_plugin {
    (
        struct $name:ident {
            $($field:ident: $field_type:ty),* $(,)?
        }

        impl Plugin for $name {
            name: $plugin_name:expr,
            version: $version:expr,
            description: $description:expr,

            $(async fn initialize(&$init_self:ident, $ctx:ident: &mut PluginContext) -> Result<(), PluginError> $init_body:block)?

            $(fn register_routes(&$routes_self:ident, $scope:ident: Scope) -> Scope $routes_body:block)?

            $(fn register_middleware(&$middleware_self:ident, $cfg:ident: &mut web::ServiceConfig) $middleware_body:block)?

            $(async fn shutdown(&$shutdown_self:ident) -> Result<(), PluginError> $shutdown_body:block)?
        }
    ) => {
        pub struct $name {
            $(pub $field: $field_type),*
        }

        #[async_trait::async_trait]
        impl $crate::plugins::Plugin for $name {
            fn name(&self) -> &'static str {
                $plugin_name
            }

            fn version(&self) -> &'static str {
                $version
            }

            fn description(&self) -> &'static str {
                $description
            }

            $(
                async fn initialize(&$init_self, $ctx: &mut $crate::plugins::PluginContext) -> Result<(), $crate::plugins::PluginError> $init_body
            )?

            $(
                fn register_routes(&$routes_self, $scope: actix_web::Scope) -> actix_web::Scope $routes_body
            )?

            $(
                fn register_middleware(&$middleware_self, $cfg: &mut actix_web::web::ServiceConfig) $middleware_body
            )?

            $(
                async fn shutdown(&$shutdown_self) -> Result<(), $crate::plugins::PluginError> $shutdown_body
            )?
        }
    };
}

/// Macro for batch plugin configuration
///
/// # Example
///
/// ```rust
/// configure_plugins! {
///     app_config,
///     plugins: [
///         AuthPlugin::new(),
///         ProcessPlugin::new(),
///         HardwarePlugin::new(),
///     ],
///     middleware: [
///         RateLimitMiddleware::new(),
///         LoggingMiddleware::new(),
///     ]
/// }
/// ```
#[macro_export]
macro_rules! configure_plugins {
    (
        $cfg:expr,
        plugins: [$($plugin:expr),* $(,)?],
        middleware: [$($middleware:expr),* $(,)?] $(,)?
    ) => {
        {
            let mut manager = $crate::plugins::PluginManager::new();

            // Register plugins
            $(
                manager.register(Box::new($plugin))
                    .expect(concat!("Failed to register plugin"));
            )*

            // Configure routes
            manager.configure_routes($cfg);

            // Configure middleware
            $(
                $cfg.wrap($middleware);
            )*
            manager.configure_middleware($cfg);

            manager
        }
    };
}

/// Macro for creating plugin groups
///
/// # Example
///
/// ```rust
/// plugin_group! {
///     name: "core",
///     plugins: [
///         AuthPlugin::new(),
///         SessionPlugin::new(),
///         UserPlugin::new(),
///     ]
/// }
/// ```
#[macro_export]
macro_rules! plugin_group {
    (
        name: $name:expr,
        plugins: [$($plugin:expr),* $(,)?] $(,)?
    ) => {
        {
            pub struct PluginGroup {
                pub name: &'static str,
                pub plugins: Vec<Box<dyn $crate::plugins::Plugin>>,
            }

            impl PluginGroup {
                pub fn new() -> Self {
                    Self {
                        name: $name,
                        plugins: vec![
                            $(Box::new($plugin)),*
                        ],
                    }
                }

                pub fn register_all(&self, manager: &mut $crate::plugins::PluginManager) -> Result<(), $crate::plugins::PluginError> {
                    for plugin in &self.plugins {
                        // Note: This would need proper cloning/Arc in real implementation
                        manager.register(plugin)?;
                    }
                    Ok(())
                }
            }

            PluginGroup::new()
        }
    };
}

/// Macro for conditional plugin loading
///
/// # Example
///
/// ```rust
/// load_plugin_if! {
///     condition: cfg.enable_auth,
///     plugin: AuthPlugin::new(),
///     manager: plugin_manager
/// }
/// ```
#[macro_export]
macro_rules! load_plugin_if {
    (
        condition: $condition:expr,
        plugin: $plugin:expr,
        manager: $manager:expr $(,)?
    ) => {
        {
            if $condition {
                $manager.register(Box::new($plugin))
                    .map_err(|e| {
                        tracing::warn!("Conditional plugin load failed: {}", e);
                        e
                    })
            } else {
                Ok(())
            }
        }
    };
}