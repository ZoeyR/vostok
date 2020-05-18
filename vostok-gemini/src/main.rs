use std::{
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use async_native_tls::{Identity, TlsAcceptor, TlsStream};
use futures::{io::BufReader, prelude::*};
use smol::{Async, Task};
use vostok_core::{request, routes, Vostok};

use gem_request::GeminiRequest;
use gem_response::{GeminiResponse, StatusCode};

mod gem_request;
mod gem_response;

fn main() -> std::io::Result<()> {
    smol::run(async {
        let vostok = Vostok::<GeminiRequest, GeminiResponse>::build().route(routes!(index, foo));

        let vostok = Arc::new(vostok);
        let identity = Identity::from_pkcs12(include_bytes!("identity.pfx"), "password").unwrap();
        let tls = TlsAcceptor::from(native_tls::TlsAcceptor::new(identity).unwrap());

        let listener = Async::<TcpListener>::bind("127.0.0.1:1965")?;
        println!("Listening on {}", listener.get_ref().local_addr()?);
        println!("Now start a TCP client.");

        // Accept clients in a loop.
        loop {
            let (stream, _) = listener.accept().await?;
            let stream = tls.accept(stream).await.unwrap();

            println!(
                "Accepted client: {}",
                stream.get_ref().get_ref().peer_addr()?
            );
            // Spawn a task that echoes messages from the client back to it.
            let vostok = vostok.clone();
            Task::spawn(async move {
                process(vostok, stream).await;
            })
            .detach();
        }
    })
}

async fn process(
    vostok: Arc<Vostok<GeminiRequest, GeminiResponse>>,
    stream: TlsStream<Async<TcpStream>>,
) -> Option<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);

    let mut buf = String::new();
    reader.read_line(&mut buf).await.ok()?;
    let request = GeminiRequest::from_line(&buf)?;
    dbg!(&request);
    let response = vostok.handle(request).await.unwrap_or(GeminiResponse {
        status: StatusCode::PermanentFailure(1),
        meta: "Resource not found".to_string(),
        body: None,
    });

    writer.write_all(&response.status.bytes()).await.ok()?;
    writer.write_all(&[b' ']).await.ok()?;
    writer.write_all(response.meta.as_bytes()).await.ok()?;
    writer.write_all(b"\r\n").await.ok()?;
    if let Some(body) = response.body {
        writer.write_all(body.as_bytes()).await.ok()?;
    }

    Some(())
}

#[request("/")]
fn index() -> &'static str {
    "Welcome to my gem space!"
}

#[request("/foo")]
fn foo() -> &'static str {
    "Welcome to my foo space!"
}
