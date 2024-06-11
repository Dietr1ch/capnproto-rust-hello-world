// Copyright (c) 2013-2016 Sandstorm Development Group, Inc. and contributors
// Licensed under the MIT License:
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

pub mod hello_world_capnp {
    include!(concat!(env!("OUT_DIR"), "/schema/hello_world_capnp.rs"));
}

use crate::hello_world_capnp::hello_world;
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use std::net::SocketAddr;

use clap::Parser;
use futures::AsyncReadExt;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Server address
    #[clap(short, long, default_value = "127.0.0.1:4030")]
    address: SocketAddr,

    #[clap(short, long)]
    name: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    tokio::task::LocalSet::new()
        .run_until(async move {
            let stream = tokio::net::TcpStream::connect(&args.address).await?;
            stream.set_nodelay(true)?;
            let (reader, writer) =
                tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
            let rpc_network = Box::new(twoparty::VatNetwork::new(
                futures::io::BufReader::new(reader),
                futures::io::BufWriter::new(writer),
                rpc_twoparty_capnp::Side::Client,
                Default::default(),
            ));
            let mut rpc_system = RpcSystem::new(rpc_network, None);
            let hello_world: hello_world::Client =
                rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);

            tokio::task::spawn_local(rpc_system);

            let mut request = hello_world.say_hello_request();
            request.get().init_request().set_name(&args.name);

            let reply = request.send().promise.await?;

            println!(
                "received: {}",
                reply.get()?.get_reply()?.get_message()?.to_str()?
            );
            Ok(())
        })
        .await
}
