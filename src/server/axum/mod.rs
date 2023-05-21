


use crate::config::ServerArgs;

mod routes;
mod session;


pub async fn run(config: ServerArgs) {
    let app = routes::configure()
        .layer(session::configure(&config.session));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}