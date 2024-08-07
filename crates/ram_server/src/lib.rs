use std::{
    fs,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
};

pub fn run_server(version: impl ToString, port: u16, online_mode: bool) -> Option<Child> {
    let config = config::get();
    let version = version.to_string();
    let java_path = {
        let version = semver::Version::parse(&version).expect("Unable to parse version");
        match version.minor {
            ..=17 => &config.ram_server.java_8_path,
            18.. => &config.ram_server.java_17_path,
            #[allow(unreachable_patterns)]
            _ => &config.ram_server.java_11_path,
        }
    };
    run_setup();
    {
        let mut from_path = config.ram_server.server_jar_path.clone().into_os_string();
        from_path.push(format!("{version}.jar"));
        let mut to_path = config.ram_server.temp_fs_path.clone().into_os_string();
        to_path.push(format!("{version}.jar"));
        fs::copy(from_path, to_path).expect("Unable to copy server jar to temp dir");
    }
    {
        let mut eula_path = config.ram_server.temp_fs_path.clone().into_os_string();
        eula_path.push("eula.txt");
        fs::write(eula_path, b"eula=true").expect("Unable to write to eula file");
    }
    {
        let mut server_properties_path = config.ram_server.temp_fs_path.clone().into_os_string();
        server_properties_path.push("server.properties");
        fs::write(
            server_properties_path,
            format!("server-port={port}\nonline-mode={online_mode}"),
        )
        .expect("Unable to write to server properties file");
    }

    let mut res = Command::new(java_path)
        .current_dir(&config.ram_server.temp_fs_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .arg("-jar")
        .arg(format!("{version}.jar"))
        .arg("--nogui")
        .spawn()
        .expect("Could not start process");

    let done_str = "[Server thread/INFO]: Done";
    let stdout = res.stdout.take().expect("Could not take stdout");

    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();
    while let Some(Ok(line)) = lines.next() {
        println!("{line}");
        if line.contains(done_str) {
            return Some(res);
        }
    }

    res.kill().expect("Count not kill child process");
    None
}

fn run_setup() {
    let path = &config::get().ram_server.temp_fs_path;
    let _ = fs::remove_dir_all(path);
    fs::create_dir(path).expect("Unable to create temp dir");
}
