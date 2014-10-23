// The MIT License (MIT)

// Copyright (c) 2014 Y. T. CHUNG

// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

//! This is a binary running in the local environment
//!
//! You have to provide all needed configuration attributes via command line parameters,
//! or you could specify a configuration file. The format of configuration file is defined
//! in mod `config`.
//!

#![feature(phase)]

extern crate getopts;
extern crate shadowsocks;
#[phase(plugin, link)]
extern crate log;

use getopts::{optopt, optflag, getopts, usage};

use std::os;

use shadowsocks::config::{Config, ServerConfig, ClientConfig, SingleServer, MultipleServer};
use shadowsocks::relay::{RelayLocal, Relay};
use shadowsocks::crypto::cipher::CIPHER_AES_256_CFB;

fn main() {
    let opts = [
        optflag("v", "version", "print version"),
        optflag("h", "help", "print this message"),
        optopt("c", "config", "specify config file", "config.json"),
        optopt("s", "server-addr", "server address", ""),
        optopt("b", "local-addr", "local address, listen only to this address if specified", ""),
        optopt("k", "password", "password", ""),
        optopt("p", "server-port", "server port", ""),
        optopt("l", "local-port", "local socks5 proxy port", ""),
        optopt("m", "encrypt-method", "entryption method", CIPHER_AES_256_CFB),
    ];

    let matches = getopts(os::args().tail(), opts).unwrap();

    if matches.opt_present("h") {
        println!("{}", usage(format!("Usage: {} [Options]", os::args()[0]).as_slice(),
                            opts));
        return;
    }

    if matches.opt_present("v") {
        println!("{}", shadowsocks::VERSION);
        return;
    }

    let mut config =
        if matches.opt_present("c") {
            Config::load_from_file(matches.opt_str("c")
                                            .unwrap().as_slice()).unwrap()
        } else {
            Config::new()
        };

    if matches.opt_present("s") && matches.opt_present("p") && matches.opt_present("k") && matches.opt_present("m") {
        let sc = ServerConfig {
            address: matches.opt_str("s").unwrap(),
            port: from_str(matches.opt_str("p").unwrap().as_slice()).expect("`port` should be an integer"),
            password: matches.opt_str("k").unwrap(),
            method: matches.opt_str("m").unwrap(),
            timeout: None,
        };
        match config.server {
            Some(ref mut server) => {
                match *server {
                    SingleServer(..) => {
                        *server = SingleServer(sc)
                    },
                    MultipleServer(ref mut server_list) => {
                        server_list.push(sc)
                    }
                }
            },
            None => { config.server = Some(SingleServer(sc)) },
        }
    } else if !matches.opt_present("s") && !matches.opt_present("b")
            && !matches.opt_present("k") && !matches.opt_present("m") {
        // Do nothing
    } else {
        fail!("`server`, `server_port`, `method` and `password` should be provided together");
    }

    if matches.opt_present("b") && matches.opt_present("l") {
        let local = ClientConfig {
            ip: from_str(matches.opt_str("b").unwrap().as_slice()).expect("`local` is not a valid IP address"),
            port: from_str(matches.opt_str("l").unwrap().as_slice()).expect("`local_port` should be an integer"),
        };
        config.local = Some(local)
    }

    info!("ShadowSocks {}", shadowsocks::VERSION);

    debug!("Config: {}", config)

    RelayLocal::new(config).run();
}