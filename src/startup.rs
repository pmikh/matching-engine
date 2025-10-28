use crate::matching::state::AppState;
use crate::routes::health_check::health_check;
use crate::routes::orders::{add_orders, remove_orders, update_orders};
use crate::routes::ws::ws_handler;
use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer, web};
use std::net::TcpListener;

pub fn run(listener: TcpListener, state: AppState) -> Result<Server, std::io::Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    let matching_ch = Data::new(state);
    let server = HttpServer::new(move || {
        App::new()
            .service(add_orders)
            .service(remove_orders)
            .service(update_orders)
            .app_data(matching_ch.clone())
            .route("/health_check", web::get().to(health_check))
            .route("/ws", web::get().to(ws_handler))
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
