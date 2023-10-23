use crate::libs::structs::Name;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct UsersData {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub username: String,
    pub pubkey: String,
    #[serde(rename = "lightningAddress")]
    pub lnurl: String,
    #[serde(rename = "registeredAt")]
    pub date: String,
}

#[derive(Debug, Serialize)]
pub struct NostrUser {
    pub names: Value,
}

pub async fn get_username(db: web::Data<Client>, name: &str) -> std::io::Result<Option<UsersData>> {
    let collections: mongodb::Collection<UsersData> =
        db.database("nostr_users").collection("registered");
    let user = collections
        .find_one(doc! { "username": name}, None)
        .await
        .unwrap();
    Ok(user)
}

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::html()).body(
        // at vercel link here as iframe
        "
            <head>
  <title>siamstr</title>
             <link rel=\"stylesheet\" href=\"assets/style.css\">
</head>
<iframe width=\"100%\" height=\"100%\" src=\"https://siamnostr-vazw.vercel.app\" title=\"siamstr.com\" style=\"border:none;\"></iframe>
",
    )
}

#[get("/nostr.json")]
pub async fn verify(db: web::Data<Client>, payload: web::Query<Name>) -> impl Responder {
    let user = get_username(db, &payload.name).await.unwrap();
    match user {
        Some(user) => {
            let user_respon = format!("{{\"{}\":\"{}\"}}", user.username, user.pubkey);
            let respon: NostrUser = NostrUser {
                names: serde_json::from_str(&user_respon).unwrap(),
            };
            HttpResponse::Ok().json(respon)
        }
        None => HttpResponse::NotFound()
            .json(serde_json::from_str::<Value>("{\"status\":404}").unwrap()),
    }
}
// {"names":{"vazw":"58f5a23008ba5a8730a435f68f18da0b10ce538c6e2aa5a1b7812076304d59f7"}}

#[get("/lnurlp/{name}")]
pub async fn lnurl(db: web::Data<Client>, payload: web::Path<String>) -> impl Responder {
    let user = get_username(db, &payload).await.unwrap();
    match user {
        Some(user) => {
            if user.lnurl.is_empty() {
                return HttpResponse::NotFound()
                    .json(serde_json::from_str::<Value>("{\"status\":404}").unwrap());
            };
            let user_domain: Vec<&str> = user.lnurl.split("@").collect();
            let respon = reqwest::get(format!(
                "https://{}/.well-known/lnurlp/{}",
                user_domain[1], user_domain[0]
            ))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
            let json_respon = serde_json::from_str::<Value>(&respon);
            match json_respon {
                Ok(expr) => {
                    return HttpResponse::Ok().json(expr);
                }
                Err(expr) => {
                    println!("{:#?}", expr);
                    return HttpResponse::NotFound().json(
                        serde_json::from_str::<Value>("{{\"status\":400,\"message\":\"Error\"}")
                            .unwrap(),
                    );
                }
            }
        }
        None => HttpResponse::NotFound()
            .json(serde_json::from_str::<Value>("{\"status\":404}").unwrap()),
    }
}
// {"status":"OK","tag":"payRequest","commentAllowed":255,"callback":"https://getalby.com/lnurlp/vazw/callback","metadata":"[[\"text/identifier\",\"vazw@getalby.com\"],[\"text/plain\",\"Sats for Vaz\"]]","minSendable":1000,"maxSendable":11000000000,"payerData":{"name":{"mandatory":false},"email":{"mandatory":false},"pubkey":{"mandatory":false}},"nostrPubkey":"79f00d3f5a19ec806189fcab03c1be4ff81d18ee4f653c88fac41fe03570f432","allowsNostr":true}
