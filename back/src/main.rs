#[macro_use]
extern crate rocket;

mod model;
mod routes;

#[launch]
async fn rocket() -> _ {
    rocket::build().mount("/", routes::routes())
}
