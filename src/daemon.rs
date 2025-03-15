use std::{collections::HashMap, str::FromStr};

use axum::{extract::{Path, Query}, http::{ HeaderName, HeaderValue, StatusCode}, response::{IntoResponse, Response}, routing::{get, post}, Json, Router};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::uds::{self};


#[derive(Debug, Serialize, Deserialize)]
struct Request {
    content:String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    code:u8,
    data:Value,
    msg:String
}

impl Message {
    pub fn new<T: AsRef<str>>(code:u8,msg:T)->Json<Self>{
        Json(Self { code: code, data: Value::Null, msg: msg.as_ref().to_owned() })
    }
    pub fn new2<T: AsRef<str>>(code:u8,msg:T,data:Value)->Json<Self>{
        Json(Self { code: code, data: data, msg: msg.as_ref().to_owned() })
    }
}

async fn handle_list()-> Json<Message>{
    match uds::list(){
        Ok(result)=>{
            let obj:Value=serde_json::from_str(&result).unwrap();
            return Message::new2(
                0,
                "Success".to_owned(),
                obj,
            );
        }
        Err(err)=>{
            return Message::new(
                10,
                err,
            );
        }
    }
}

async fn handle_install(Json(req): Json<Request>)->Result<Json<Message>,StatusCode>{
    match uds::install(req.content){
        Ok(_)=>{
            return Ok(Message::new(
                0,
                "Success",
            ));
        }
        Err(err)=>{
            return Ok(Message::new(
                10,
                err,
            ));
        }
    }
}


async fn handle_install_pwa(Json(req): Json<Request>)->Result<Json<Message>,StatusCode>{
    match uds::install_pwa(req.content){
        Ok(_)=>{
            return Ok(Message::new(
                0,
                "Success",
            ));
        }
        Err(err)=>{
            return Ok(Message::new(
                10,
                err,
            ));
        }
    }
}

async fn handle_uninstall(Json(req): Json<Request>)->Result<Json<Message>,StatusCode>{
    match uds::uninstall(req.content){
        Ok(_)=>{
            return Ok(Message::new(
                0,
                "Success",
            ));
        }
        Err(err)=>{
            return Ok(Message::new(
                10,
                err,
            ));
        }
    }
}

async fn handle_proxy(
    Path((host,path)): Path<(String,String)>,
    query: Query<HashMap<String,String>>,
) -> Response {
    let client = Client::new();

    // println!("{} {}",host,path);

    // 构造目标 URL
    let url = format!("{}/{}", "http://127.0.0.1", path);
    let mut request=client.get(&url);
    
    for (k,v) in query.0.iter(){
        request=request.query(&(k,v));
    }

    request=request.header("Host", host);

    // 发送请求
    match request.send().await {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp.headers().clone(); // 复制原始响应头
            match resp.bytes().await {
                Ok(body) => {
                    let mut response = Response::builder().status(status);
                    let check_headers=["Content-Type","Content-Length","Content-Encoding"];
                    for &header in check_headers.iter(){
                        if let Some(value) = headers.get(header) {
                            response = response.header(header, value);
                        }
                    };
                    response.body(body.into()).unwrap()
                }
                Err(_) => StatusCode::BAD_GATEWAY.into_response(),
            }
        }
        Err(_) => StatusCode::BAD_GATEWAY.into_response(),
    }
}

pub async fn run(){
    let app = Router::new()
    .route("/list", get(handle_list))
    .route("/install",post(handle_install))
    .route("/install-pwa",post(handle_install_pwa))
    .route("/proxy/{host}/{*path}", get(handle_proxy))
    .route("/uninstall",post(handle_uninstall))
    .layer(
        ServiceBuilder::new()
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_str("Access-Control-Allow-Origin").unwrap(),
                #[cfg(feature ="test")]
                HeaderValue::from_static("*"),
                #[cfg(not(feature ="test"))]
                HeaderValue::from_static("http://ostore.localhost"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_str("Access-Control-Allow-Methods").unwrap(),
                HeaderValue::from_static("GET,POST"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                HeaderName::from_str("Access-Control-Allow-Headers").unwrap(),
                HeaderValue::from_static("Content-Type"),
            )),
    );


    #[cfg(feature ="test")]
    let addr="0.0.0.0:5431";
    #[cfg(not(feature ="test"))]
    let addr="127.0.0.1:5431";

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("server running on {}...",addr);
    axum::serve(listener, app).await.unwrap();
}