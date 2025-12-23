use home::home_dir;

pub static GITIGNORE_URL: &str = "https://github.com/github/gitignore/archive/main.zip";

lazy_static! {
    pub static ref CONN_STR: String = format!(
        "mysql://hrcnrwusr:{}@hrpcns1rw.db.ie.tslans.net/security",
        "TODO"
    );
    pub static ref MASTER_ZIP_PATH: std::path::PathBuf = home_dir()
        .expect("unable to locate home_dir")
        .join("gitignore_main.zip");

    pub static ref EXTRACT_DIR: std::path::PathBuf = MASTER_ZIP_PATH.parent().unwrap().join("gitignore-cached");
}
