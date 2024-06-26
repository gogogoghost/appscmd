use std::{collections::VecDeque, env, io::{Read, Write}, os::unix::net::UnixStream};

use serde_json::Value;

const SOCKET_PATH:&str="/data/local/tmp/apps-uds.sock";
// const SOCKET_PATH:&str="/tmp/nokia2.sock";

fn transfer(cmd:String,param:Option<String>)->Result<String,String>{
    let mut socket=UnixStream::connect(SOCKET_PATH).unwrap();
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

fn install(path:String){
    let res=transfer("install".to_owned(),Some(path));
    match res{
        Ok(success)=>{
            println!("{}",success);
        }
        Err(err)=>{
            eprintln!("{}",err);
        }
    }
}

fn install_pwa(url:String){
    let res=transfer("install-pwa".to_owned(),Some(url));
    match res{
        Ok(success)=>{
            println!("{}",success);
        }
        Err(err)=>{
            eprintln!("{}",err);
        }
    }
}

fn list(){
    let res=transfer("list".to_owned(),None);
    match res{
        Ok(success)=>{
            let obj:Value=serde_json::from_str(&success).unwrap();
            println!("{}",serde_json::to_string_pretty(&obj).unwrap());
        }
        Err(err)=>{
            eprintln!("{}",err);
        }
    }
}

fn main() {
    // 获取命令行参数的迭代器
    let args_raw:Vec<String>=env::args().collect();
    let mut args: VecDeque<String> = VecDeque::from(args_raw);
    args.pop_front().unwrap();
    let command=if let Some(command) = args.pop_front(){
        command
    }else{
        eprintln!("no command");
        return;
    };
    match command.as_str(){
        "install"=>{
            let path=if let Some(path)=args.pop_front(){
                path
            }else{
                eprintln!("no file path");
                return;
            };
            install(path);
        },
        "install-pwa"=>{
            let url=if let Some(url)=args.pop_front(){
                url
            }else{
                eprintln!("no manifest url");
                return;
            };
            install_pwa(url);
        }
        "list"=>{
            list();
        }
        _=>{
            eprintln!("bad command");
            return;
        }
    }
}
