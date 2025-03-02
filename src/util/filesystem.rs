use std::{ env, path::PathBuf };

pub async fn get_filesystem_path(path: &str) -> PathBuf {
    let path = if cfg!(debug_assertions) {
        env::current_dir().unwrap().join(path)
    } else {
        env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(path)
    };

    if !path.exists() {
        tokio::fs::create_dir_all(&path).await.unwrap();
    }

    path
}
