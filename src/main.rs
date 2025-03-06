use std::{collections::VecDeque, env};

use serde_json::Value;

mod daemon;
mod uds;

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
            match uds::install(path){
                Ok(success)=>{
                    println!("{}",success);
                }
                Err(err)=>{
                    eprintln!("{}",err);
                }
            }
        },
        "install-pwa"=>{
            let url=if let Some(url)=args.pop_front(){
                url
            }else{
                eprintln!("no manifest url");
                return;
            };
            match uds::install_pwa(url){
                Ok(success)=>{
                    println!("{}",success);
                }
                Err(err)=>{
                    eprintln!("{}",err);
                }
            }
        }
        "list"=>{
            match uds::list(){
                Ok(success)=>{
                    let obj:Value=serde_json::from_str(&success).unwrap();
                    println!("{}",serde_json::to_string_pretty(&obj).unwrap());
                }
                Err(err)=>{
                    eprintln!("{}",err);
                }
            }
        }
        "daemon"=>{
            daemon::run();
        }
        _=>{
            eprintln!("bad command");
            return;
        }
    }
}
