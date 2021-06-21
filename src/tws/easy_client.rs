use super::client::ClientImpl;

struct EasyClient {
    client: ClientImpl,
}

/* struct SecurityExpiration {
    strikes: std::collections::HashSet<>
} */

struct SecurityDefinition {}

impl EasyClient {
    fn new(client: ClientImpl) -> EasyClient {
        EasyClient { client }
    }
}
