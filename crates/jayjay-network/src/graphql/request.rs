use serde::Serialize;

#[derive(Serialize)]
pub struct GraphQlRequest<'a, V> {
    pub query: &'a str,
    pub variables: V,
}
