pub mod core;
pub mod epic;

use luo9_sdk::Bot;
use luo9_sdk::Msg;
use luo9_sdk::bus::Bus;
use luo9_sdk::command::Command;
use luo9_sdk::command::PrefixMode;

use serde_json::json;
use std::ffi::CString;
use std::thread::sleep;
use std::time::Duration;

fn handle_task_event(json: &str) {
    let Ok(event) = serde_json::from_str::<serde_json::Value>(json) else {
        return;
    };
    let task_name = event["task_name"].as_str().unwrap_or("unknown");
    let payload = event["payload"].as_str().unwrap_or("");
    println!("[task] 定时任务触发: name={}, payload={}", task_name, payload);
    if let Ok(group_id) = payload.parse::<u64>() {        
        let msg = Msg::txt("今天的epic免费游戏提醒来啦！").build();
        Bot::send_group_msg(group_id, msg);
        epic_free_games(group_id);
    }
}

pub fn handle_group_msg(group_id: u64, user_id: u64, msg: &str) {
    match Command::parse(msg, "epic", PrefixMode::None) {
        Some(cmd) => {
            cmd.handle(|| {
                epic_free_games(group_id);
            }).on("提醒开启", |_| {
                task_start(group_id);
            }).on("提醒关闭", |_| {
                task_end(group_id);
            });
        },
        None => return,
    }
}

pub fn epic_free_games(group_id: u64) {
    let free_games = epic::get_epic_free_games().unwrap();
    let mut game_str = String::new();

    let mut msg = CString::new(format!("当前共有{}个免费游戏", free_games.len()).as_str()).unwrap();
    Bot::send_group_msg(group_id, msg);
    sleep(Duration::from_millis(100));

    for game in free_games {
        game_str.push_str(&format!("{:?}\n", game));
        msg = Msg::txt(format!("游戏名称: {}", game.title)).endl()
        .txt(format!("描述: {}", game.description)).endl()
        .txt(format!("开发商/发行商: {}", game.seller)).endl()
        .txt(format!("免费结束时间: {}", game.free_end_date)).endl()
        .txt(format!("商店链接: {}", game.store_url)).endl()
        .txt("预览图片：").image(game.preview_image)
        .build();

        Bot::send_group_msg(group_id, msg);
    }

}

pub fn task_start(group_id: u64) {
    let task_name = format!("epic_free_{}", group_id);
    let req = json!({
        "action": "schedule",
        "task_name": task_name,
        "cron": "0 0 8 * * *",
        "payload": group_id.to_string()
    });
    match Bus::topic("luo9_task_miso").publish(&req.to_string()) {
        Ok(_) => println!("已注册每日8点定时, group_id: {}", group_id),
        Err(e) => println!("注册定时失败: {:?}", e),
    }

    let msg = Msg::txt("开启每日epic免费游戏提醒！").build();
    Bot::send_group_msg(group_id, msg);
}

pub fn task_end(group_id: u64) {
    let task_name = format!("epic_free_{}", group_id);
    let req = json!({
        "action": "cancel",
        "task_name": task_name
    });
    match Bus::topic("luo9_task_miso").publish(&req.to_string()) {
        Ok(_) => println!("已取消每日定时, group_id: {}", group_id),
        Err(e) => println!("取消定时失败: {:?}", e),
    }

    let msg = Msg::txt("关闭每日epic免费游戏提醒").build();
    Bot::send_group_msg(group_id, msg);

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_group_msg_epic_reminder_enable() {
        handle_group_msg(123456, 789012, "epic提醒开启");
        
    }

    #[test]
    fn test_handle_group_msg_epic_reminder_disable() {
        handle_group_msg(123456, 789012, "epic提醒关闭");
    }

    #[test]
    fn test_none_command() {
        handle_group_msg(123456, 789012, "");
    }

    // #[test]
    fn _test_none_args() {
        handle_group_msg(123456, 789012, "epic");
        handle_group_msg(123456, 789012, "epic  ");
        handle_group_msg(123456, 789012, "  epic");
        handle_group_msg(123456, 789012, "  epic  ");
    }
}