pub fn setup() {
    ccrs::logger::init_logger(|level, file, line, msg| {
        println!("[{:#?}] {}:{} {}", level, file, line, msg);
    });
}
