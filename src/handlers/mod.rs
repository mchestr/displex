use actix_web::web;

mod discord;
mod plex;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/discord")
            .service(web::resource("/linked-role").route(web::get().to(discord::linked_role)))
            .service(web::resource("/callback").route(web::get().to(discord::callback))),
    )
    .service(
        web::scope("/plex")
            .service(web::resource("/callback").route(web::get().to(plex::callback))),
    );
}
