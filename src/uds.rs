use std::{io::{Read, Write}, os::unix::net::UnixStream};

use serde_json::Value;

const SOCKET_PATH:&str="/data/local/tmp/apps-uds.sock";
// const SOCKET_PATH:&str="/tmp/nokia2.sock";

fn transfer(cmd:String,param:Option<String>)->Result<String,String>{
    let mut socket=match UnixStream::connect(SOCKET_PATH){
        Ok(socket)=>socket,
        Err(_)=>{
            return Err("Unable to connect uds".to_owned())
        }
    };
    let mut obj = serde_json::map::Map::new();
    obj.insert("cmd".to_owned(), Value::String(cmd.to_owned()));
    obj.insert("param".to_owned(), match param {
        Some(param)=>{
            Value::String(param)
        },
        None=>{
            Value::Null
        }
    });
    socket.write_all(Value::Object(obj).to_string().as_bytes()).unwrap();
    socket.write(&[0x0d,0x0a]).unwrap();

    let mut buffer = Vec::new();
    let mut byte = [0; 1];
    loop {
        socket.read_exact(&mut byte).unwrap();
        buffer.push(byte[0]);
        if buffer.ends_with(&[b'\r', b'\n']) {
            break;
        }
    }
    let result=String::from_utf8(buffer).unwrap();
    let obj:Value=serde_json::from_str(result.as_str()).unwrap();
    match obj.get("error"){
        Some(error)=>{
            return Err(error.as_str().unwrap().to_owned());
        }
        _=>{}
    }
    match obj.get("success"){
        Some(success)=>{
            return Ok(success.as_str().unwrap().to_owned())
        }
        _=>{
            return Err("unexpect result".to_owned())
        }
    }
}

pub fn install(path:String)->Result<String,String>{
    transfer("install".to_owned(),Some(path))
}

pub fn install_pwa(url:String)->Result<String,String>{
    transfer("install-pwa".to_owned(),Some(url))
}

pub fn list()->Result<String,String>{
    transfer("list".to_owned(),None)
}

pub fn uninstall(manifest_url:String)->Result<String,String>{
    transfer("uninstall".to_owned(),Some(manifest_url))
}