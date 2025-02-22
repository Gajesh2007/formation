use std::time::Duration;
use colored::Colorize;
use daemonize::Daemonize;
use formnet::up;
use tokio::runtime::Runtime;

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();
    #[cfg(target_os = "linux")]
    let daemon = Daemonize::new()
        .pid_file("/run/formnet.pid")
        .chown_pid_file(true)
        .working_directory("/")
        .umask(0o027)
        .stdout(std::fs::File::create("/var/log/formnet.log").unwrap())
        .stderr(std::fs::File::create("/var/log/formnet.log").unwrap());


    #[cfg(target_os = "linux")]
    match daemon.start() {
        Ok(_) => {
            let rt = Runtime::new().expect("unable to launch tokio runtime");
            rt.block_on(async {
                if let Err(e) = up(
                    Some(Duration::from_secs(60)),
                    None,
                ).await {
                    println!("{}: {}", "Error trying to bring formnet up".yellow(), e.to_string().red());
                }
            })
        }
        Err(e) => {
            println!("{}: {}", "Error trying to daemonize formnet".yellow(), e.to_string().red());
        }
    }

    #[cfg(not(target_os = "linux"))]
    let rt = Runtime::new().expect("Unable to create tokio runtime");
    #[cfg(not(target_os = "linux"))]
    rt.block_on(async {
        if let Err(e) = up(
            Some(Duration::from_secs(60)),
            None,
        ).await {
            println!("{}: {}", "Error trying to bring formnet up".yellow(), e.to_string().red());
        }
    });
}
