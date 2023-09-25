pub mod libs;
use self::libs::route::{hello, lnurl, verify};
use actix_files::Files;
use actix_web::{middleware::Logger, web, App, HttpServer};
use env_logger::Env;
use mongodb::Client;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let uri = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let client = Client::with_uri_str(uri).await.expect("failed to connect");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(hello)
            .service(web::scope("/.well-known").service(verify).service(lnurl))
            .service(Files::new("/assets", "./assets/").show_files_listing())
    })
    .bind(("0.0.0.0", 8008))?
    .run()
    .await
}
