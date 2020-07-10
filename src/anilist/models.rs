use graphql_client::*;
// use serde::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "res/graphql/schema.json",
    query_path = "res/graphql/user.gql",
    response_derives = "Debug",
)]
pub struct User;