static DEFAULT_FETCHER: OnceCell<ArcSwap<Logger>> = OnceCell::new();

fn fetch() {
    let body = serde_json::String();

    //
    let srv_http_cli = G_SERVICE_HTTP_CLIENT.clone();

    launch_service(&srv_http_cli, || {
        //
    });

    srv_http_cli.http_post(
        "https://18.163.14.56:48964",
        vec!["Content-Type: application/json".to_owned()],
        body,
        |code, resp| {
            //
            //log::info!("hello http code: {}, resp: {}", code, resp);
        },
    )
}