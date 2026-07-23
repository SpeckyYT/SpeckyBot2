use itertools::Itertools;
use serenity::all::EditMessage;
use tokio::process::Command;

crate::command! {
    names: "update",
    category: "owner",
    run: |ctx, msg, _data| {
        let mut statuses: Statuses = vec![
            (Status::Queued(false), "Reset repository instance", &["git", "reset", "--hard"]),
            (Status::Queued(false), "Fetch repository", &["git", "fetch", "--all"]),
            (Status::Queued(false), "Pull repository", &["git", "pull", "origin"]),
            (Status::Queued(true), "Compile", &["cargo", "build", "--release"]),
            (Status::Queued(false), "Restart", &["pm2", "restart", "speckybot"]),
        ];

        let mut bot_msg = msg.channel_id.say(&ctx.http, draw_statuses(&statuses, None)).await?;

        for i in 0..statuses.len() {
            let can_fail = matches!(statuses[i].0, Status::Queued(true));
            statuses[i].0 = Status::Running;

            bot_msg.edit(&ctx.http, EditMessage::new().content(draw_statuses(&statuses, None))).await?;

            let mut output = Command::new(statuses[i].2[0])
                .args(&statuses[i].2[1..])
                .spawn()? // TODO idk, if the command doesn't exist, it should error
                .wait_with_output()
                .await?;

            let mut fail = None;
            statuses[i].0 = match (can_fail, output.status.success()) {
                (true, _)|(_, true) => Status::Success,
                (false, false) => {
                    output.stdout.extend(output.stderr);
                    fail = Some(String::from_utf8_lossy(&output.stdout).to_string());
                    Status::Failed
                },
            };

            bot_msg.edit(&ctx.http, EditMessage::new().content(draw_statuses(&statuses, fail.as_ref().map(|s| s.as_str())))).await?;

            if fail.is_some() { break }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum Status {
    Queued(bool), // error is acceptable or not
    Running,
    Failed,
    Success,
}

impl Status {
    pub fn as_emoji(&self) -> &str {
        match self {
            Self::Queued(_) => "♻️",
            Self::Running => "🔃",
            Self::Failed => "🚱",
            Self::Success => "✅",
        }
    }
}

type Statuses = Vec<(Status, &'static str, &'static [&'static str])>;

fn draw_statuses(statuses: &Statuses, output: Option<&str>) -> String {
    let mut statuses = statuses.iter().map(|(status, name, _)| format!("{} {name}", status.as_emoji())).join("\n");
    
    if let Some(output) = output {
        statuses.push_str(&format!("\n```\n{}\n```", &output[..1990_usize.saturating_sub(statuses.len())]));
    }

    statuses
}
