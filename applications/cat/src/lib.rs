#![no_std]
#![feature(alloc)]
#[macro_use] extern crate terminal_print;
#[macro_use] extern crate log;

#[macro_use] extern crate alloc;
extern crate task;
extern crate getopts;
extern crate path;
extern crate fs_node;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::string::ToString;
use getopts::Options;
use core::ops::Deref;
use path::Path;
use fs_node::FileOrDir;


#[no_mangle]
pub fn main(args: Vec<String>) -> isize {
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    
    let matches = match opts.parse(&args) {
        Ok(m) => m,
        Err(_f) => {
            println!("{}", _f);
            print_usage(opts);
            return -1;
        }
    };
    
    if matches.free.is_empty() {
        return 0;
    }
    let taskref = match task::get_my_current_task() {
        Some(t) => t,
        None => {
            println!("failed to get current task");
            return -1;
        }
    };

    // grabs the current working directory pointer; this is scoped so that we drop the lock on the task as soon as we get the working directory pointer
    let curr_wr = {
        let locked_task = taskref.lock();
        let curr_env = locked_task.env.lock();
        Arc::clone(&curr_env.working_dir)
    };
    let path = Path::new(matches.free[0].to_string());
    
    // navigate to the filepath specified by first argument
    match path.get(&curr_wr) {
        Ok(file_dir_enum) => { 
            match file_dir_enum {
                FileOrDir::Dir(directory) => {
                    println!("'{}' is not a file, cannot call cat", directory.lock().get_name());
                },
                FileOrDir::File(file) => {
                    let file_size = file.lock().size();
                    let mut string_slice_as_bytes = vec![0; file_size];
                    
                    let num_bytes_read = match file.lock().read(&mut string_slice_as_bytes) {
                        Ok(num) => num,
                        Err(_) => {return -1},
                    };
                    let read_string = match String::from_utf8(string_slice_as_bytes) {
                        Ok(string_slice) => string_slice,
                        Err(_) => {return -1}, // consider printing error 
                    };
                    println!("{}", read_string);
                }
            }

        },
        Err(err) => {
            println!("get call failed in cat because: {}", err);
            return -1;
        }
    };
    return 0;
}

fn print_usage(opts: Options) {
    println!("{}", opts.usage(USAGE));
}


const USAGE: &'static str = "Usage: cd [ARGS]
Change directory";