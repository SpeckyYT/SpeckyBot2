use ilog::IntLog;
use serenity::all::CreateMessage;

use crate::commands::MATH_CATEGORY;

crate::command! {
    names: ["collatz", "coll"],
    usage: "[number]",
    category: MATH_CATEGORY,
    run: |ctx, msg, data| {
        let output = match data.cmd_content.parse::<u128>() {
            Ok(mut num) => {
                let mut string = String::with_capacity(2000);
                let mut first = true;

                loop {
                    let chars = num.log10() + 1;
                    if string.len() + 10 + chars + 1 > 2000 {
                        break
                    } else if !first {
                        string.push(' ');
                    }
                    string.push_str(&num.to_string());

                    if num == 1 { break }
                    first = false;

                    num = match num.is_multiple_of(2) {
                        true => num / 2,
                        false => num * 3 + 1,
                    }
                }

                string
            },
            Err(_) => "Insert number".to_string()
        };

        let format_output = format!("```js\n{output}\n```");

        Ok(msg.channel_id.send_message(&ctx.http, CreateMessage::new().content(format_output)).await.map(|_| ())?)
    }
}
