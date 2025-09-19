use actix_web::web::Scope;

pub trait LegacyPlugin {
    fn name(&self) -> &'static str;
    fn register(&self, scope: Scope) -> Scope;
}

#[macro_export]
macro_rules! register_plugins {
    ($scope:expr, [ $( $p:expr ),+ $(,)? ]) => {{
        let mut s = $scope;
        $( s = $p.register(s); )+
        s
    }};
}

