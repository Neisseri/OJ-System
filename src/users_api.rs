use actix_web::{post, get, web, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use crate::global::{USER_LIST, User};
use crate::error::Error;
use crate::response;

#[derive(Serialize, Deserialize, Clone)]
pub struct PostUser {
    pub id: Option<usize>,
    pub name: String,
}

#[post("/users")]
async fn put_users(body: web::Json<PostUser>) -> impl Responder {

    let mut lock = USER_LIST.lock().unwrap();
    let mut name_repeated: bool = false;
    for i in 0..(*lock).len() {
        if &(*lock)[i].name == &body.name {
            name_repeated = true;
            break;
        }
    } // check if the name is repeated
    if name_repeated == true {
        return HttpResponse::BadRequest().json(Error {
            code: 1,
            reason: "ERR_INVALID_ARGUMENT".to_string(),
            message: format!("User name '{}' already exists.",
                body.name.clone()).clone()
        });
    } // return the BadRequest response of repeated names

    let response: User;
    let len = (*lock).len();
    if body.id.is_some() == true {
        let user_id = body.id.clone().unwrap();
        if user_id + 1 > len {
            return HttpResponse::NotFound().json(Error {
                code: 3,
                reason: "ERR_NOT_FOUND".to_string(),
                message: format!("User {} not found.",
                    body.id.clone().unwrap()).clone()
            });
        } // the user Id is invalid
        (*lock)[user_id].name = body.name.clone();
        // rename the user

        response = User { id: user_id, name: body.name.clone(), };

    } else { // create a new user
        (*lock).push(User { 
            id: len, 
            name: body.name.clone(), 
        });

        response = User {
            id: len, 
            name: body.name.clone(), 
        };

    }

    HttpResponse::Ok().json(response)
}

#[get("/users")]
async fn get_users() -> impl Responder {
    let lock = USER_LIST.lock().unwrap();
    let response = (*lock).clone();
    HttpResponse::Ok().json(response)
}
