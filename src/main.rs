mod models;
mod repositories;
mod rocket_routes;
mod schema;

use rocket_db_pools::Database;

#[derive(Database)]
#[database("postgres")]
struct DbConn(rocket_db_pools::diesel::PgPool);

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .mount(
            "/",
            rocket::routes![
                rocket_routes::rustaceans::get_rustaceans,
                rocket_routes::rustaceans::view_rustaceans,
                rocket_routes::rustaceans::create_rustaceans,
                rocket_routes::rustaceans::update_rustaceans,
                rocket_routes::rustaceans::delete_rustaceans,
            ],
        )
        .attach(DbConn::init())
        .launch()
        .await;
}
