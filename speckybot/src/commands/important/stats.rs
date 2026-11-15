use serenity::all::{CreateEmbedFooter, CreateMessage};
use sysinfo::{Disks, Networks, System};

use crate::{commands::COMMANDS_EXECUTED, util::embed::default_embed};

crate::command! {
    names: ["stats", "status", "st"],
    category: Important,
    run: |ctx, msg, _data| {
        let mut system = System::new_all()?;
        tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;
        system.refresh_cpu_usage();

        let cpu_usage = system.global_cpu_usage().clamp(0.0, 100.0);
        let cpu_count = system.cpus().len();
        let (cpu_model, cpu_frequency) = system.cpus()
            .first()
            .map(|cpu| (cpu.brand().to_owned(), format!("{} MHz", cpu.frequency())))
            .unwrap_or_else(|| ("Unavailable".to_owned(), "Unavailable".to_owned()));

        let used_memory = system.used_memory();
        let total_memory = system.total_memory();
        let process_memory = sysinfo::get_current_pid()
            .ok()
            .and_then(|pid| system.process(pid))
            .map(|process| process.memory());

        let disk_usage = Disks::new_with_refreshed_list()
            .ok()
            .and_then(|disks| {
                let (total, available) = disks.iter().fold((0_u64, 0_u64), |(total, available), disk| {
                    (
                        total.saturating_add(disk.total_space()),
                        available.saturating_add(disk.available_space()),
                    )
                });
                (total > 0).then_some((total.saturating_sub(available), total))
            });

        let network_totals = Networks::new_with_refreshed_list()
            .ok()
            .map(|networks| {
                networks.iter().fold((0_u64, 0_u64), |(received, transmitted), (_, network)| {
                    (
                        received.saturating_add(network.total_received()),
                        transmitted.saturating_add(network.total_transmitted()),
                    )
                })
            });

        let total_emotes = ctx.cache.guilds()
            .into_iter()
            .filter_map(|guild_id| ctx.cache.guild(guild_id))
            .map(|guild| guild.emojis.len())
            .sum::<usize>();

        let (bot_name, bot_avatar) = {
            let bot_user = ctx.cache.current_user();
            (bot_user.name.clone(), bot_user.avatar_url())
        };

        let api_ping_start = std::time::Instant::now();
        ctx.http.get_current_user().await?;
        let api_ping = api_ping_start.elapsed().as_millis();

        let memory_percent = percentage(used_memory, total_memory);
        let cpu_percent = cpu_usage.round() as u64;
        let memory_stats = format!(
            "RAM: {} [{}%] ({}/{})\nCPU: {} [{}%]\nProcess: {}\nStorage: {}\nProcesses: {}",
            progress_bar(used_memory, total_memory),
            memory_percent,
            format_bytes(used_memory),
            format_bytes(total_memory),
            progress_bar(cpu_percent, 100),
            cpu_percent,
            process_memory.map(format_bytes).unwrap_or_else(|| "Unavailable".to_owned()),
            disk_usage
                .map(|(used, total)| format!("{} [{}%]", progress_bar(used, total), percentage(used, total)))
                .unwrap_or_else(|| "Unavailable".to_owned()),
            system.processes().len(),
        );

        let network_stats = network_totals
            .map(|(received, transmitted)| {
                format!("Received: {}\nTransmitted: {}", format_bytes(received), format_bytes(transmitted))
            })
            .unwrap_or_else(|| "Unavailable".to_owned());

        let system_stats = [
            ("OS", System::name()),
            ("Version", System::os_version()),
            ("Kernel", System::kernel_version()),
        ]
        .into_iter()
        .map(|(label, value)| format!("{label}: {}", value.unwrap_or_else(|_| "Unavailable".to_owned())))
        .chain(std::iter::once(format!("Architecture: {}", std::env::consts::ARCH)))
        .collect::<Vec<_>>()
        .join("\n");

        let mut footer = CreateEmbedFooter::new(&bot_name);
        if let Some(avatar) = &bot_avatar {
            footer = footer.icon_url(avatar.clone());
        }

        let mut embed = default_embed(Some(ctx))
            .description("Here are some stats about the bot and this machine.")
            .field("API Ping", format!("{api_ping} ms"), true)
            .field("Usage", memory_stats, false)
            .field(
                "Machine Specs",
                format!("CPU Count: {cpu_count}\nCPU Model: {cpu_model}\nCPU Speed: {cpu_frequency}"),
                false,
            )
            .field("System Specs", system_stats, false)
            .field("Network Stats (since interface start)", network_stats, false)
            .field("Cached Users", ctx.cache.user_count().to_string(), true)
            .field("Total Emotes", total_emotes.to_string(), true)
            .field("Total Guilds", ctx.cache.guild_count().to_string(), true)
            .field(
                "Total Executed Commands",
                format!("{} Commands", COMMANDS_EXECUTED.load(std::sync::atomic::Ordering::Relaxed)),
                false,
            )
            .field(
                "Bot Uptime",
                crate::BOT_CONNECTED_AT
                    .get()
                    .map(|started_at| format_duration(started_at.elapsed()))
                    .unwrap_or_else(|| "Unavailable".to_owned()),
                true,
            )
            .field(
                "Process Uptime",
                crate::PROCESS_STARTED_AT
                    .get()
                    .map(|started_at| format_duration(started_at.elapsed()))
                    .unwrap_or_else(|| "Unavailable".to_owned()),
                true,
            )
            .footer(footer);

        if let Some(avatar) = bot_avatar {
            embed = embed.thumbnail(avatar);
        }

        msg.channel_id.send_message(&ctx.http, CreateMessage::new().embed(embed)).await?;
        Ok(())
    },
}

fn percentage(used: u64, total: u64) -> u64 {
    if total == 0 {
        0
    } else {
        ((used as f64 / total as f64) * 100.0).round().clamp(0.0, 100.0) as u64
    }
}

fn progress_bar(used: u64, total: u64) -> String {
    const PRECISION: usize = 20;

    let filled = ((percentage(used, total) as usize * PRECISION) + 50) / 100;
    "█".repeat(filled) + &"░".repeat(PRECISION - filled)
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit = 0;

    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    format!("{size:.2} {}", UNITS[unit])
}

fn format_duration(duration: std::time::Duration) -> String {
    let seconds = duration.as_secs();
    let units = [
        ("day", seconds / 86_400),
        ("hour", seconds / 3_600 % 24),
        ("minute", seconds / 60 % 60),
        ("second", seconds % 60),
    ];

    let parts = units
        .into_iter()
        .filter(|(_, value)| *value > 0)
        .map(|(unit, value)| format!("{value} {unit}{}", if value == 1 { "" } else { "s" }))
        .collect::<Vec<_>>();

    if parts.is_empty() {
        "0 seconds".to_owned()
    } else {
        parts.join(" ")
    }
}
