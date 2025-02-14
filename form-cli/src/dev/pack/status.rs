use clap::Args;
use colored::Colorize;
use form_pack::manager::{PackBuildStatus, PackResponse};
use reqwest::Client;

#[derive(Debug, Clone, Args)]
pub struct StatusCommand {
    #[clap(long="build-id", short='i')]
    build_id: String
}

impl StatusCommand {
    pub async fn handle_status(&self, provider: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        println!("Checking status for build ID: {}", self.build_id.bright_blue());
        
        let response = Client::new()
            .get(&format!("http://{provider}:{port}/{}/get_status", self.build_id))
            .send().await;

        let status = match response {
            Ok(resp) => {
                match resp.json::<PackResponse>().await {
                    Ok(status) => status,
                    Err(e) => {
                        println!("\nError parsing response from server: {}", e.to_string().bright_red());
                        println!("This could mean the server is not running or is misconfigured.");
                        return Ok(());
                    }
                }
            },
            Err(e) => {
                println!("\nError connecting to server: {}", e.to_string().bright_red());
                println!("Please check that:");
                println!("1. The provider {} is running and accessible", provider.bright_yellow());
                println!("2. The port {} is correct and open", port.to_string().bright_yellow());
                println!("3. Your network connection is stable");
                return Ok(());
            }
        };

        match status {
            PackResponse::Status(status) => {
                match status {
                    PackBuildStatus::Started(id) => {
                        println!(r#"
Build ID: {} is currently in progress.

Status: {}

Please be patient, as builds can take several minutes depending on:
- Size of build artifacts
- Number of dependencies to install
- System resources available

Check back in a few minutes for an update.
"#,
                        id.bright_blue(),
                        "STARTED".blue());
                    }
                    PackBuildStatus::Failed { build_id, reason } => {
                        println!(r#"
Build ID: {} has failed.

Status: {}
Reason: {}

Please check your Formfile and build configuration.
"#,
                        build_id.bright_blue(),
                        "FAILED".bright_red(),
                        reason.bright_yellow());
                    }
                    PackBuildStatus::Completed(instance) => {
                        println!(r#"
Build ID: {} has completed successfully!

Status: {}

You can now deploy this build by running:
{}
"#,
                        instance.build_id.bright_blue(),
                        "COMPLETED".bright_green(),
                        "form pack ship".bright_yellow());
                    }
                }
            }
            PackResponse::Success => {
                println!("\nBuild status check successful, but no status information is available.");
                println!("This usually means the build ID {} doesn't exist.", self.build_id.bright_yellow());
            }
            PackResponse::Failure => {
                println!("\nFailed to get build status for ID: {}", self.build_id.bright_yellow());
                println!("This could mean:");
                println!("1. The build ID doesn't exist");
                println!("2. The build status has expired from the queue");
                println!("3. There was an error processing the status request");
            }
        }

        Ok(())
    }
}

pub fn print_pack_status(status: PackResponse, build_id: String) {
    match status {
        PackResponse::Status(
            PackBuildStatus::Started(_id)
        ) => {
            println!(r#"
Your build has {} but has not yet {}.

Please be patient, as builds can take as long as {}, or possibly even
longer depending on the size and number build artifacts and/or number of arguments in your
{} or options selected on the GUI. This is particularly true if you included a significant number 
of system dependencies or application level dependencies to be installed, or included large 
system dependencies to be installed.

Check back in a couple more minutes to see an update of the status.

If your status is still `{}` after {} please consider taking one of the following
actions:

    1. Join our discord at {} and go to the {} channel and paste this response
    2. Submitting an {} on our project github at {} 
    3. Sending us a direct message on X at {}

One of our core contributors will be glad to help you out.
"#,
"Started".blue(),
"Completed".yellow(),
"5 minutes".bright_purple(),
"Formfile".blue(),
"Started".yellow(),
"10 minutes".bright_red(),
"discord.gg/formation".blue(),
"chewing-glass".bright_yellow(),
"Issue".yellow(),
"http://github.com/formthefog/formation.git".blue(),
"@formthefog".blue(),
);
        }
        PackResponse::Status(
            PackBuildStatus::Failed { build_id, reason }
        ) => {
            println!(r#"
We regret to inform you that your build with build id {} has {}.

The reason for the falure was: {}

If the reason for the failure is bewildering or does not make sense to you, please
consider taking one of the following actions:

    1. Join our discord at {} and go to the {} channel and paste this response
    2. Submitting an {} on our project github at {} 
    3. Sending us a direct message on X at {}

One of our core contributors will be glad to help you out.
"#,
build_id.bright_blue(),
"Failed".bright_red(),
reason.italic().bright_magenta(),
"discord.gg/formation".blue(),
"chewing-glass".bright_yellow(),
"Issue".yellow(),
"http://github.com/formthefog/formation.git".blue(),
"@formthefog".blue(),
);
        }
        PackResponse::Status(
            PackBuildStatus::Completed(instance)
        ) => {
            println!(r#"
We are overjoyed to inform you that your build with build id {} has {}.

You can now `{}` your build by running:

```
{}
```

If you run into any issues during the `{}` phase of deployment, please consider
taking one of the following actions:

    1. Join our discord at {} and go to the {} channel and paste this response
    2. Submitting an {} on our project github at {} 
    3. Sending us a direct message on X at {}

One of our core contributors will be glad to help you out.
"#,
instance.build_id.bright_blue(),
"Completed".bright_red(),
"ship".blue(),
"form pack [OPTIONS] ship".bright_yellow(),
"ship".blue(),
"discord.gg/formation".blue(),
"chewing-glass".bright_yellow(),
"Issue".yellow(),
"http://github.com/formthefog/formation.git".blue(),
"@formthefog".blue(),
);
        }
        PackResponse::Failure => {
            println!(r#"
Something went {} wrong. The response received was {:?} which is an invalid response 
to the `{}` command.

Please consider doing one of the following: 

    1. Join our discord at {} and go to the {} channel and paste this response
    2. Submitting an {} on our project github at {} 
    3. Sending us a direct message on X at {}

Someone from our core team will gladly help you out.
"#,
"terribly".bright_red().on_blue(),
status,
"form pack [OPTIONS] build".bright_yellow(),
"discord.gg/formation".blue(),
"chewing-glass".blue(),
"issue".bright_yellow(),
"http://github.com/formthefog/formation.git".blue(),
"@formthefog".blue(),
);
        }
        _ => {
            println!(r#"
Something went {} wrong. The response received was {:?} which is an invalid response 
to the `{}` command.

Please consider doing one of the following: 

    1. Join our discord at {} and go to the {} channel and paste this response
    2. Submitting an {} on our project github at {} 
    3. Sending us a direct message on X at {}

Someone from our core team will gladly help you out.
"#,
"terribly".bright_red().on_blue(),
status,
"form pack [OPTIONS] build".bright_yellow(),
"discord.gg/formation".blue(),
"chewing-glass".blue(),
"issue".bright_yellow(),
"http://github.com/formthefog/formation.git".blue(),
"@formthefog".blue(),
);
        }
    }
}
