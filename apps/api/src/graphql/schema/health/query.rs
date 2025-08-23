use async_graphql::Object;

#[derive(Default)]
pub struct HealthQuery;

#[Object(rename_fields = "snake_case")]
impl HealthQuery {
    async fn health(&self) -> &'static str {
        "OK"
    }
}