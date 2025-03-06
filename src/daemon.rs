use std::{fs::File, io::{BufReader, Cursor, Read}};

use base64::{engine::general_purpose, Engine};
use rsa::{ pkcs8::DecodePublicKey, Pkcs1v15Sign, RsaPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_string, Value};
use sha2::{Digest, Sha256};
use tiny_http::{Header, Method, Request, Response, Server};

use crate::uds;

#[derive(Debug, Serialize, Deserialize)]
struct InstallRequest {
    path:String,
    sign:String
}

#[derive(Debug, Serialize, Deserialize)]
struct InstallPWARequest {
    url:String,
    sign:String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    code:u8,
    data:Value,
    msg:String
}

impl Message {
    pub fn new<T: AsRef<str>>(code:u8,msg:T)->Self{
        Self { code: code, data: Value::Null, msg: msg.as_ref().to_owned() }
    }
    pub fn to_res(&self)->Response<Cursor<Vec<u8>>>{
        Response::from_string(to_string(self).unwrap()).with_header(Header::from_bytes(b"Content-Type", b"application/json").unwrap())
    }
}

fn sha256_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];

    while let Ok(n) = reader.read(&mut buffer) {
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let hash = hasher.finalize();
    Ok(hash.to_vec())
}

fn sha256_str(content:&[u8])-> Vec<u8>{
    let mut hasher = Sha256::new();
    hasher.update(content);
    let hash=hasher.finalize();
    hash.to_vec()
}

pub trait RequestExtend {
    fn not_found(self);
    fn send(self,msg:Message);
}

impl RequestExtend for Request{
    fn not_found(self) {
        self.respond(Response::empty(404)).ok();
    }
    
    fn send(self,msg:Message) {
        self.respond(msg.to_res()).ok();
    }
}

const ADDR:&'static str="127.0.0.1:5431";

pub fn run(){
    // load pub key
    let public_key = RsaPublicKey::from_public_key_der(include_bytes!("../appscmd_public.der")).unwrap();


    let server=Server::http(ADDR).unwrap();
    println!("server running on {}...",ADDR);
    for mut request in server.incoming_requests() {
        let method=request.method();
        let url=request.url();
        match method{
            Method::Get=>{
                match url{
                    "/list"=>{
                        match uds::list(){
                            Ok(result)=>{
                                let obj:Value=serde_json::from_str(&result).unwrap();
                                request.send(Message{
                                    code:0,
                                    data:obj,
                                    msg:"Success".to_owned()
                                });
                                continue;
                            }
                            Err(err)=>{
                                request.send(Message::new(10,err));
                                continue;
                            }
                        }
                    }
                    _=>{}
                }
            }
            Method::Post=>{
                match url{
                    "/install"=>{
                        // read path and sign
                        let req:InstallRequest=match from_reader(request.as_reader()){
                            Ok(req)=>req,
                            _=>{
                                request.not_found();
                                continue;
                            }
                        };
                        let sign=match general_purpose::STANDARD.decode(req.sign){
                            Ok(sign)=>sign,
                            _=>{
                                request.not_found();
                                continue;
                            }
                        };
                        let file_hash=match sha256_file(&req.path){
                            Ok(hash)=>hash,
                            _=>{
                                request.send(Message::new(1,"Failed to read file"));
                                continue;
                            }
                        };
                        // let hashed_file_hash=sha256_str(&file_hash);
                        // verify
                        match public_key.verify(Pkcs1v15Sign::new::<Sha256>(), &file_hash, &sign){
                            Ok(_)=>{},
                            _=>{
                                request.send(Message::new(2,"Sign not match"));
                                continue;
                            }
                        }
                        // install
                        match uds::install(req.path){
                            Ok(_)=>{
                                request.send(Message::new(0,"Success"));
                                continue;
                            }
                            Err(err)=>{
                                request.send(Message::new(10,err));
                                continue;
                            }
                        }
                    }
                    "/install-pwa"=>{
                        // read url and sign
                        let req:InstallPWARequest=match from_reader(request.as_reader()){
                            Ok(req)=>req,
                            _=>{
                                request.not_found();
                                continue;
                            }
                        };
                        let sign=match general_purpose::STANDARD.decode(req.sign){
                            Ok(sign)=>sign,
                            _=>{
                                request.not_found();
                                continue;
                            }
                        };
                        let hashed_url=sha256_str(req.url.as_bytes());
                        //verify
                        match public_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed_url, &sign){
                            Ok(_)=>{},
                            Err(err)=>{
                                eprintln!("{:?}",err);
                                request.send(Message::new(2,"Sign not match"));
                                continue;
                            }
                        }
                        // install
                        match uds::install_pwa(req.url){
                            Ok(_)=>{
                                request.send(Message::new(0,"Success"));
                                continue;
                            }
                            Err(err)=>{
                                request.send(Message::new(10,err));
                                continue;
                            }
                        }
                    }
                    _=>{}
                }
            }
            _=>{}
        }
        request.not_found();
    }
}