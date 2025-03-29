use diesel_async::{AsyncConnection, AsyncPgConnection};

use crate::{
    models::NewUser,
    repositories::{RoleRepository, UserRepository},
};

async fn load_db_connection() -> AsyncPgConnection {
    let database_url = std::env::var("DATABASE_URL").expect("Cannot load DB url environment");
    AsyncPgConnection::establish(&database_url)
        .await
        .expect("Cannot connect to Postgres")
}

pub async fn create_user(username: String, password: String, role_codes: Vec<String>) {
    let mut c = load_db_connection().await;

    let new_user = NewUser { username, password };
    let user = UserRepository::create(&mut c, new_user, role_codes)
        .await
        .unwrap();
    println!("User created {:?}", user);
    let roles = RoleRepository::find_by_user(&mut c, &user).await.unwrap();
    println!("Roles assigned {:?}", roles);
}

pub async fn list_users() {
    let mut c = load_db_connection().await;

    let users = UserRepository::find_with_roles(&mut c).await.unwrap();
    for user in users {
        print!("{:?} ", user);
    }
}

pub async fn delete_user(id: i32) {
    let mut c = load_db_connection().await;

    UserRepository::delete(&mut c, id)
        .await
        .unwrap();
}
