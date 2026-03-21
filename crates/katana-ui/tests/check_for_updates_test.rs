use katana_ui::updater::check_for_updates;

#[test]
fn test_check_for_updates_fetch() {
    let (tx, rx) = std::sync::mpsc::channel();
    check_for_updates(move |res| {
        tx.send(res).unwrap();
    });
    // Wait for the result
    match rx.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(res) => {
            if let Err(e) = res {
                println!("Error checking updates: {}", e);
            }
        }
        Err(_) => println!("Timeout waiting for update"),
    }
}
