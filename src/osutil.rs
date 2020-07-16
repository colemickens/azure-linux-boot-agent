use tokio::io::AsyncWriteExt;

const ENTROPY_PATH: &str = "/sys/firmware/acpi/tables/OEM0";
const RANDOM_PATH: &str = "/dev/random"; // CONFIRM from MS PDF

const CDROM_PATH: &str = "/dev/sr0";

pub async fn seed_entropy() -> Result<(), std::io::Error> {
    let mut entropy_file = tokio::fs::OpenOptions::new()
        .read(true)
        .open(ENTROPY_PATH)
        .await?;

    let mut random_file = tokio::fs::OpenOptions::new()
        .write(true)
        .read(false)
        .open(RANDOM_PATH)
        .await?;

    tokio::io::copy(&mut entropy_file, &mut random_file).await?;

    Ok(())
}

pub async fn create_wheel_user(username: &str) -> Result<(), std::io::Error> {
    // create user
    let mut cmd = tokio::process::Command::new("useradd");
    let cmd = cmd.args(&["-m", &username]);
    trace!("{:?}", &cmd);
    let status = cmd.spawn()?.await.unwrap();
    if !status.success() {
        error!("hostname call failed");
        std::process::exit(-1); // TODO: error handling is all fucked up
    }

    // add user to wheel group
    let mut cmd = tokio::process::Command::new("gpasswd");
    let cmd = cmd.args(&["--add", &username, "wheel"]);
    trace!("{:?}", &cmd);
    let status = cmd.spawn()?.await.unwrap();
    if !status.success() {
        error!("hostname call failed");
        std::process::exit(-1); // TODO: error handling is all fucked up
    }

    Ok(())
}

pub async fn fixup_user_homedir_permissions(username: &str) -> Result<(), std::io::Error> {
    // useradd -m .... <- dont i have this already somewhere?
    let homedir = &format!("/home/{}", username);
    let mut cmd = tokio::process::Command::new("chown");
    let cmd = cmd.args(&["-R", &username, homedir]);
    trace!("{:?}", &cmd);
    let status = cmd.spawn()?.await.unwrap();
    if !status.success() {
        error!("hostname call failed");
        std::process::exit(-1); // TODO: error handling is all fucked up
    }

    let ssh_dir = &format!("/home/{}/.ssh", username);
    let mut cmd = tokio::process::Command::new("chmod");
    let cmd = cmd.args(&["0700", ssh_dir]);
    trace!("{:?}", cmd);
    let status = cmd.spawn()?.await.unwrap();
    if !status.success() {
        error!("hostname call failed");
        std::process::exit(-1); // TODO: error handling is all fucked up
    }

    let auth_keys = &format!("/home/{}/.ssh/authorized_keys", username);
    let mut cmd = tokio::process::Command::new("chmod");
    let cmd = cmd.args(&["0644", auth_keys]);
    trace!("{:?}", cmd);
    let status = cmd.spawn()?.await.unwrap();
    if !status.success() {
        error!("hostname call failed");
        std::process::exit(-1); // TODO: error handling is all fucked up
    }

    Ok(())
}

pub fn get_username_from_keypath(keypath: &str) -> Result<String, ()> {
    use regex::Regex;
    let re = Regex::new(r"^/home/([a-zA-Z][a-zA-Z0-9]*)/.ssh/authorized_keys$").unwrap();
    let captures = re.captures(keypath).unwrap();
    let username = captures.get(1).unwrap().as_str().to_string();
    Ok(username)
}

pub async fn create_users_with_keys(
    k: &crate::metadata::instance::compute::PublicKey,
) -> Result<(), std::io::Error> {
    // TODO: write a test with multiple SSH keys
    // looks like this will call useradd multiple times and idk what happens in that case

    // extract user name + create user
    let username = get_username_from_keypath(&k.path).unwrap();
    println!("creating wheel user: {}", &username);
    create_wheel_user(&username).await?;

    // write ssh keys out
    let key_path = std::path::Path::new(&k.path);
    let prefix = key_path.parent().unwrap();
    tokio::fs::create_dir_all(prefix)
        .await
        .expect("failed to create dirs");
    let mut key_file = tokio::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&k.path)
        .await
        .expect(&format!("failed to create/open {}", &k.path));
    println!("writing to file ({}) key: {}", &k.path, &k.key_data);
    key_file.write_all(k.key_data.as_bytes()).await?;

    // change ownership + permissions
    fixup_user_homedir_permissions(&username).await?;

    Ok(())
}

pub async fn read_ovf_env() -> Result<String, std::io::Error> {
    let dest_dir = tempfile::tempdir()?;
    let mut cmd = tokio::process::Command::new("7z");
    cmd.args(&["x", &CDROM_PATH, "ovf-env.xml"]);
    cmd.current_dir(&dest_dir);
    let status = cmd.spawn()?.await?;
    if !status.success() {
        panic!("read ovf_env failed"); // TODO: do this better
    }
    let tempfile = dest_dir.path().join("ovf-env.xml");
    tokio::fs::read_to_string(&tempfile).await
}
